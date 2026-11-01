//! Task Markdown parsing/writing, vault file watching, and the
//! `_ai/index/tasks.json` machine-readable index.
//!
//! The vault is the source of truth (see `vault-template/CLAUDE.md`): every
//! task is one `tasks/<id> <title>.md` file with a YAML-ish frontmatter
//! block. Frontmatter here uses a fixed, known schema (see `Task` in
//! `models.rs`), so rather than pull in a general YAML crate we hand-roll a
//! small parser/writer for exactly that shape. This keeps the round-trip
//! guarantee simple to reason about: the body (everything after the closing
//! `---`) is captured verbatim and never touched unless the caller explicitly
//! supplies a replacement.

use crate::models::Task;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

fn tasks_dir(vault: &Path) -> PathBuf {
    vault.join("tasks")
}

/// Archived tasks are relocated here (a subfolder of `tasks/`) so the flat
/// `tasks/` listing stays uncluttered in Obsidian/Explorer. The `archived:
/// true` frontmatter flag remains the logical source of truth; the folder is
/// kept in sync with it on every write (see `relocate_for_archive_state`).
const ARCHIVE_SUBDIR: &str = "archive";

fn archive_dir(vault: &Path) -> PathBuf {
    tasks_dir(vault).join(ARCHIVE_SUBDIR)
}

fn index_file(vault: &Path) -> PathBuf {
    vault.join("_ai").join("index").join("tasks.json")
}

/// Normalizes a path to forward slashes for cross-separator comparison
/// (`Task.file` is always stored with `/`, while `Path::join` yields `\` on
/// Windows).
fn norm_path(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

// ---------------------------------------------------------------------
// frontmatter parsing / rendering
// ---------------------------------------------------------------------

/// Splits `---\n<frontmatter>\n---\n<body>` into (frontmatter text, body).
/// The body is the exact remainder of the file, byte-for-byte.
fn split_frontmatter(content: &str) -> Result<(String, String), String> {
    let mut lines = content.split_inclusive('\n');
    let first = lines.next().unwrap_or("");
    if first.trim_end_matches(['\r', '\n']) != "---" {
        return Err("file does not start with a frontmatter block".into());
    }
    let mut consumed = first.len();
    let mut front = String::new();
    let mut closed = false;
    for line in lines {
        consumed += line.len();
        if line.trim_end_matches(['\r', '\n']) == "---" {
            closed = true;
            break;
        }
        front.push_str(line);
    }
    if !closed {
        return Err("frontmatter block is not closed".into());
    }
    let body = content[consumed..].to_string();
    Ok((front, body))
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if s.len() >= 2 {
        let bytes = s.as_bytes();
        if (bytes[0] == b'"' && bytes[s.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[s.len() - 1] == b'\'')
        {
            return s[1..s.len() - 1].to_string();
        }
    }
    s.to_string()
}

struct RawFrontmatter {
    map: HashMap<String, String>,
    tags: Vec<String>,
    depends_on: Vec<String>,
}

/// Frontmatter keys whose value is a list (inline `[a, b]` or a `- a` block).
const LIST_KEYS: [&str; 2] = ["tags", "depends_on"];

fn parse_frontmatter(front: &str) -> RawFrontmatter {
    let mut map = HashMap::new();
    let mut lists: HashMap<&'static str, Vec<String>> = HashMap::new();
    let mut block_key: Option<&'static str> = None;
    for raw_line in front.lines() {
        let line = raw_line.trim_end();
        if let Some(key) = block_key {
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix("- ") {
                lists.entry(key).or_default().push(unquote(rest.trim()));
                continue;
            } else if trimmed.is_empty() {
                continue;
            } else {
                block_key = None; // fall through to normal key parsing
            }
        }
        let Some(idx) = line.find(':') else { continue };
        let key = line[..idx].trim().to_string();
        let val = line[idx + 1..].trim();
        if let Some(list_key) = LIST_KEYS.iter().find(|k| **k == key) {
            if val.is_empty() {
                block_key = Some(list_key);
                lists.insert(list_key, Vec::new());
            } else if let Some(inner) = val.strip_prefix('[').and_then(|v| v.strip_suffix(']')) {
                let items = inner
                    .split(',')
                    .map(|s| unquote(s.trim()))
                    .filter(|s| !s.is_empty())
                    .collect();
                lists.insert(list_key, items);
            } else {
                lists.insert(list_key, vec![unquote(val)]);
            }
        } else {
            map.insert(key, unquote(val));
        }
    }
    RawFrontmatter {
        map,
        tags: lists.remove("tags").unwrap_or_default(),
        depends_on: lists.remove("depends_on").unwrap_or_default(),
    }
}

/// Quotes a scalar only if it contains characters that would otherwise
/// confuse a YAML/Obsidian reader (colon, hash, brackets, leading/trailing
/// whitespace, or is empty).
fn yaml_scalar(s: &str) -> String {
    let needs_quote = s.is_empty()
        || s.contains(':')
        || s.contains('#')
        || s.contains('[')
        || s.contains(']')
        || s.contains(',')
        || s != s.trim();
    if needs_quote {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

fn render_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        "[]".to_string()
    } else {
        format!(
            "[{}]",
            tags.iter()
                .map(|t| yaml_scalar(t))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// Renders an `order` float without a trailing `.0` for whole numbers, so
/// hand-edited files stay tidy (`order: 3`, not `order: 3.0`).
fn render_order(order: f64) -> String {
    if order.fract() == 0.0 && order.abs() < 1e15 {
        format!("{}", order as i64)
    } else {
        format!("{order}")
    }
}

fn render_frontmatter(t: &Task) -> String {
    let order_line = t
        .order
        .map(|o| format!("order: {}\n", render_order(o)))
        .unwrap_or_default();
    // Emitted only when set, like `order`, so files never archived stay
    // byte-identical on round-trip.
    let archived_line = if t.archived { "archived: true\n" } else { "" };
    // Same policy for the per-task launch flags: only emitted when enabled, so
    // files that never opt in stay byte-identical on round-trip.
    let confirm_line = if t.confirm { "confirm: true\n" } else { "" };
    let worktree_line = if t.worktree { "worktree: true\n" } else { "" };
    // Same policy for the blocked trio: a task that was never blocked carries
    // none of the three lines. The note/since lines are tied to the flag so a
    // cleared block leaves no stale leftovers behind.
    let blocked_lines = if t.blocked {
        let mut s = String::from("blocked: true\n");
        if !t.blocked_note.is_empty() {
            s.push_str(&format!("blocked_note: {}\n", yaml_scalar(&t.blocked_note)));
        }
        if !t.blocked_since.is_empty() {
            s.push_str(&format!("blocked_since: {}\n", t.blocked_since));
        }
        s
    } else {
        String::new()
    };
    // Same policy: only tasks that specify a model carry the line.
    let model_line = if t.model.is_empty() {
        String::new()
    } else {
        format!("model: {}\n", yaml_scalar(&t.model))
    };
    // Same policy again: a task that belongs to no backlog item carries no
    // `backlog:` line at all, so every task file written before T-0253 stays
    // byte-identical.
    let backlog_line = if t.backlog.is_empty() {
        String::new()
    } else {
        format!("backlog: {}\n", yaml_scalar(&t.backlog))
    };
    // Same policy once more: only tasks with a predecessor carry `depends_on:`.
    let depends_line = if t.depends_on.is_empty() {
        String::new()
    } else {
        format!("depends_on: {}\n", render_tags(&t.depends_on))
    };
    // Same policy once more: `completed` is only written while it carries a
    // time, so every task file written before T-0727 stays byte-identical on
    // round-trip. It sits right after `updated`, the stamp it was taken with.
    let completed_line = if t.completed.is_empty() {
        String::new()
    } else {
        format!("completed: {}\n", t.completed)
    };
    format!(
        "---\nid: {}\ntitle: {}\nstatus: {}\nassignee: {}\nproject: {}\n{}priority: {}\n{}{}due: {}\ntags: {}\n{}{}{}{}{}created: {}\nupdated: {}\n{}---\n",
        t.id,
        yaml_scalar(&t.title),
        t.status,
        t.assignee,
        yaml_scalar(&t.project),
        backlog_line,
        t.priority,
        model_line,
        order_line,
        t.due,
        render_tags(&t.tags),
        archived_line,
        confirm_line,
        worktree_line,
        blocked_lines,
        depends_line,
        t.created,
        t.updated,
        completed_line,
    )
}

fn parse_task_file(path: &Path) -> Result<Task, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let (front, body) = split_frontmatter(&content)?;
    let raw = parse_frontmatter(&front);
    let get = |k: &str| raw.map.get(k).cloned().unwrap_or_default();
    Ok(Task {
        id: get("id"),
        title: get("title"),
        status: {
            let v = get("status");
            if v.is_empty() {
                "inbox".into()
            } else {
                v
            }
        },
        assignee: {
            let v = get("assignee");
            if v.is_empty() {
                "me".into()
            } else {
                v
            }
        },
        project: get("project"),
        backlog: get("backlog"),
        priority: {
            let v = get("priority");
            if v.is_empty() {
                "medium".into()
            } else {
                v
            }
        },
        model: get("model"),
        order: raw.map.get("order").and_then(|v| v.parse::<f64>().ok()),
        due: get("due"),
        tags: raw.tags,
        archived: raw
            .map
            .get("archived")
            .map(|v| v == "true")
            .unwrap_or(false),
        confirm: raw.map.get("confirm").map(|v| v == "true").unwrap_or(false),
        worktree: raw
            .map
            .get("worktree")
            .map(|v| v == "true")
            .unwrap_or(false),
        blocked: raw.map.get("blocked").map(|v| v == "true").unwrap_or(false),
        blocked_note: get("blocked_note"),
        blocked_since: get("blocked_since"),
        depends_on: raw.depends_on,
        created: get("created"),
        updated: get("updated"),
        completed: get("completed"),
        file: path.to_string_lossy().replace('\\', "/"),
        body,
    })
}

/// Writes a task back to disk, rewriting only the frontmatter block. The
/// body is whatever is currently held on `task.body` — callers that did not
/// intend to change it must have preserved it from a prior parse, so the
/// file round-trips byte-for-byte outside the frontmatter block.
fn write_task_file(task: &Task) -> Result<(), String> {
    let content = format!("{}{}", render_frontmatter(task), task.body);
    fs::write(&task.file, content).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------
// scanning / id assignment
// ---------------------------------------------------------------------

pub fn scan_tasks(vault: &Path) -> Result<Vec<Task>, String> {
    let mut out = Vec::new();
    // `tasks/` holds active tasks; `tasks/archive/` holds archived ones. Both
    // are scanned so archived tasks still reserve their id (see `next_id`),
    // stay in the index, and remain findable for unarchiving. Reading `tasks/`
    // non-recursively skips the `archive/` subdirectory itself (it has no `.md`
    // extension), so the two passes never double-count.
    scan_dir_into(&tasks_dir(vault), &mut out)?;
    scan_dir_into(&archive_dir(vault), &mut out)?;
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

/// Parses every `*.md` task file directly inside `dir` (non-recursive) and
/// appends them to `out`. Missing directories and unparsable files are skipped
/// rather than failing the whole scan.
fn scan_dir_into(dir: &Path, out: &mut Vec<Task>) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some("_index.md") {
            continue;
        }
        match parse_task_file(&path) {
            Ok(task) => out.push(task),
            Err(_) => continue, // skip unparsable/non-task files rather than fail the whole scan
        }
    }
    Ok(())
}

fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

/// `YYYY-MM-DD HH:MM` in local time, stamped when a task's `status` changes
/// to `done` (T-0727). Local rather than UTC because the owner reads it
/// against their own day. Windows answers this directly; elsewhere the date
/// comes from [`civil_from_days`] and the clock from the epoch's remainder —
/// the same UTC approximation [`today`] already accepts on those targets,
/// which this app does not ship to.
#[cfg(windows)]
fn now_datetime() -> String {
    // SAFETY: GetLocalTime reads the clock and returns by value; it takes no
    // pointer from us and cannot fail.
    let st = unsafe { windows::Win32::System::SystemInformation::GetLocalTime() };
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute
    )
}

#[cfg(not(windows))]
fn now_datetime() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let rem = (secs % 86_400) as u32;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}",
        rem / 3600,
        (rem % 3600) / 60
    )
}

/// Howard Hinnant's `civil_from_days`: days-since-epoch -> (year, month, day).
/// Avoids pulling in a full date/time crate for a single "today" stamp.
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d)
}

// ---------------------------------------------------------------------
// create / update
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaskInput {
    pub id: String,
    pub title: Option<String>,
    pub status: Option<String>,
    pub assignee: Option<String>,
    pub project: Option<String>,
    pub backlog: Option<String>,
    pub priority: Option<String>,
    pub model: Option<String>,
    pub order: Option<f64>,
    pub due: Option<String>,
    pub tags: Option<Vec<String>>,
    pub archived: Option<bool>,
    pub confirm: Option<bool>,
    pub worktree: Option<bool>,
    pub blocked: Option<bool>,
    pub blocked_note: Option<String>,
    pub blocked_since: Option<String>,
    pub depends_on: Option<Vec<String>>,
    pub body: Option<String>,
}

fn find_task_by_id(vault: &Path, id: &str) -> Result<Task, String> {
    let prefix = format!("{id} ");
    // An archived task lives under `tasks/archive/`, so both directories must
    // be searched (e.g. to unarchive it or edit it while archived).
    for dir in [tasks_dir(vault), archive_dir(vault)] {
        if !dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if name.starts_with(&prefix) && name.ends_with(".md") {
                return parse_task_file(&path);
            }
        }
    }
    Err(format!("task {id} not found"))
}

/// Trims, drops blanks and duplicates, keeping the caller's order.
fn normalize_deps(deps: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for d in deps {
        let d = d.trim().to_string();
        if !d.is_empty() && !out.contains(&d) {
            out.push(d);
        }
    }
    out
}

/// Whether giving `id` the predecessors `deps` would close a loop: true when
/// `id` is one of `deps`, or is reachable by following `depends_on` from them.
pub fn would_create_cycle(id: &str, deps: &[String], all: &[Task]) -> bool {
    let mut stack: Vec<&str> = deps.iter().map(|d| d.as_str()).collect();
    let mut seen: Vec<&str> = Vec::new();
    while let Some(cur) = stack.pop() {
        if cur == id {
            return true;
        }
        if seen.contains(&cur) {
            continue;
        }
        seen.push(cur);
        if let Some(t) = all.iter().find(|t| t.id == cur) {
            stack.extend(t.depends_on.iter().map(|d| d.as_str()));
        }
    }
    false
}

fn check_no_cycle(id: &str, deps: &[String], all: &[Task]) -> Result<(), String> {
    if would_create_cycle(id, deps, all) {
        Err(format!(
            "{id} cannot depend on {}: that would make a dependency cycle",
            deps.join(", ")
        ))
    } else {
        Ok(())
    }
}

pub fn update_task(vault: &Path, input: UpdateTaskInput) -> Result<Task, String> {
    let mut task = find_task_by_id(vault, &input.id)?;
    if let Some(v) = input.title {
        task.title = v;
    }
    if let Some(v) = input.status {
        let was_done = task.status == "done";
        let now_done = v == "done";
        task.status = v;
        // The completion stamp belongs to the transition, not the state: it
        // is taken when `done` is entered, cleared when it is left, and never
        // touched otherwise — so saving a task that is already `done` keeps
        // its existing value, and an old `done` task without one keeps none.
        if now_done && !was_done {
            if task.completed.is_empty() {
                task.completed = now_datetime();
            }
        } else if !now_done && was_done {
            task.completed.clear();
        }
    }
    if let Some(v) = input.assignee {
        task.assignee = v;
    }
    if let Some(v) = input.project {
        task.project = v;
    }
    if let Some(v) = input.backlog {
        task.backlog = v;
    }
    if let Some(v) = input.priority {
        task.priority = v;
    }
    if let Some(v) = input.model {
        task.model = v;
    }
    if let Some(v) = input.order {
        task.order = Some(v);
    }
    if let Some(v) = input.due {
        task.due = v;
    }
    if let Some(v) = input.tags {
        task.tags = v;
    }
    if let Some(v) = input.archived {
        task.archived = v;
    }
    if let Some(v) = input.confirm {
        task.confirm = v;
    }
    if let Some(v) = input.worktree {
        task.worktree = v;
    }
    if let Some(v) = input.blocked_note {
        task.blocked_note = v;
    }
    if let Some(v) = input.blocked_since {
        task.blocked_since = v;
    }
    // Applied after the two detail fields so toggling the flag always wins:
    // unblocking clears the details, and blocking stamps today when the caller
    // didn't supply a date (the common case — a badge click or a bare toggle).
    if let Some(v) = input.blocked {
        task.blocked = v;
        if v {
            if task.blocked_since.is_empty() {
                task.blocked_since = today();
            }
        } else {
            task.blocked_note.clear();
            task.blocked_since.clear();
        }
    }
    if let Some(v) = input.depends_on {
        let deps = normalize_deps(v);
        if !deps.is_empty() {
            let all = scan_tasks(vault)?;
            check_no_cycle(&task.id, &deps, &all)?;
        }
        task.depends_on = deps;
    }
    if let Some(v) = input.body {
        task.body = v;
    }
    task.updated = today();
    // Keep the file's folder in sync with its archived state before writing,
    // so archived tasks move into `tasks/archive/` and unarchived ones move
    // back to `tasks/`.
    relocate_for_archive_state(vault, &mut task)?;
    write_task_file(&task)?;
    refresh_index(vault);
    Ok(task)
}

/// Ensures `task.file` sits in the directory matching its archived state
/// (`tasks/archive/` when archived, `tasks/` otherwise), moving the file on
/// disk and updating `task.file` when they disagree. The basename is
/// preserved. A no-op when the file is already in the right place, so it is
/// safe to call on every update.
fn relocate_for_archive_state(vault: &Path, task: &mut Task) -> Result<(), String> {
    let current = PathBuf::from(&task.file);
    let Some(file_name) = current.file_name().map(|n| n.to_owned()) else {
        return Ok(());
    };
    let target_dir = if task.archived {
        archive_dir(vault)
    } else {
        tasks_dir(vault)
    };
    let target = target_dir.join(&file_name);
    if norm_path(&target) == norm_path(&current) {
        return Ok(());
    }
    fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
    if current.exists() {
        fs::rename(&current, &target).map_err(|e| e.to_string())?;
    }
    task.file = norm_path(&target);
    Ok(())
}

// ---------------------------------------------------------------------
// index
// ---------------------------------------------------------------------

#[derive(Serialize)]
struct IndexEntry<'a> {
    id: &'a str,
    title: &'a str,
    status: &'a str,
    assignee: &'a str,
    project: &'a str,
    backlog: &'a str,
    priority: &'a str,
    model: &'a str,
    order: Option<f64>,
    due: &'a str,
    tags: &'a [String],
    archived: bool,
    confirm: bool,
    worktree: bool,
    blocked: bool,
    blocked_note: &'a str,
    blocked_since: &'a str,
    depends_on: &'a [String],
    created: &'a str,
    updated: &'a str,
    completed: &'a str,
    file: String,
}

pub fn regenerate_index(vault: &Path) -> Result<(), String> {
    let tasks = scan_tasks(vault)?;
    let vault_prefix = vault.to_string_lossy().replace('\\', "/");
    let entries: Vec<IndexEntry> = tasks
        .iter()
        .map(|t| IndexEntry {
            id: &t.id,
            title: &t.title,
            status: &t.status,
            assignee: &t.assignee,
            project: &t.project,
            backlog: &t.backlog,
            priority: &t.priority,
            model: &t.model,
            order: t.order,
            due: &t.due,
            tags: &t.tags,
            archived: t.archived,
            confirm: t.confirm,
            worktree: t.worktree,
            blocked: t.blocked,
            blocked_note: &t.blocked_note,
            blocked_since: &t.blocked_since,
            depends_on: &t.depends_on,
            created: &t.created,
            updated: &t.updated,
            completed: &t.completed,
            file: t
                .file
                .strip_prefix(&vault_prefix)
                .unwrap_or(&t.file)
                .trim_start_matches('/')
                .to_string(),
        })
        .collect();
    let path = index_file(vault);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let body = serde_json::to_string_pretty(&entries).map_err(|e| e.to_string())?;
    fs::write(&path, body).map_err(|e| e.to_string())
}

/// Rewrites the index after a task file has already been written. The index is
/// derived data, so a failure here (typically Windows refusing to truncate
/// `tasks.json` while another process has it memory-mapped, os error 1224) is
/// logged and swallowed: the update itself succeeded, and returning the error
/// made a bulk archive stop after its first task (T-0917).
fn refresh_index(vault: &Path) {
    if let Err(e) = regenerate_index(vault) {
        crate::diag!("tasks: index rewrite failed after a task update: {e}");
    }
}

/// Convenience for the `list_tasks` command: scan and keep the index fresh
/// in the same call, since a scan already has everything the index needs.
pub fn scan_and_index(vault: &Path) -> Result<Vec<Task>, String> {
    let tasks = scan_tasks(vault)?;
    let _ = regenerate_index(vault);
    Ok(tasks)
}

// ---------------------------------------------------------------------
// vault init + template sync (vault-template/, embedded in the binary)
// The project scaffold template is embedded into the binary at compile time
// (`include_dir!`), so a packaged build has no dev-machine path dependency.

use include_dir::{include_dir, Dir};

static VAULT_TEMPLATE: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../vault-template");

/// Location of the project scaffold inside the embedded template.
pub(crate) const PROJECT_TEMPLATE_DIR: &str = "templates/project";

/// The embedded `templates/project/` subtree, for scaffolding a new vault
/// project from the Schedule view (T-0178). `None` only in a build where the
/// template folder was removed — the caller turns that into an error.
pub(crate) fn project_template() -> Option<&'static Dir<'static>> {
    VAULT_TEMPLATE.get_dir(PROJECT_TEMPLATE_DIR)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A task that belongs to no backlog item writes no `backlog:` line at
    /// all, so every task file written before T-0253 stays byte-identical; one
    /// that does belong to an item round-trips through the frontmatter.

    #[test]
    fn depends_on_block_list_is_read() {
        let raw =
            parse_frontmatter("id: T-0002\ntags: []\ndepends_on:\n  - T-0001\n  - \"T-0003\"\n");
        assert_eq!(raw.depends_on, vec!["T-0001", "T-0003"]);
        assert!(raw.tags.is_empty());
    }
}
