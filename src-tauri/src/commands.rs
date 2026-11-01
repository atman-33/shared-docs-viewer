use crate::diagram_edit;
use crate::docs::{self, DocsEntry, DocsRootStatus};
use crate::mindmap;
use crate::models::{
    Config, DocsRoot, DocsShortcut, MindmapDoc, MindmapFile, ScheduleDoc, ScheduleFile, Task,
    VaultProject,
};
use crate::schedule;
use crate::tasks::{self, UpdateTaskInput};
use crate::vault_note;
use crate::vault_project;
use crate::{actions, diagram, storage, update};
use serde::Serialize;
use std::path::PathBuf;

use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
pub fn get_config() -> Config {
    storage::load()
}

/// Persists `config` and returns the configuration that took effect.
#[tauri::command]
pub fn save_config(app: tauri::AppHandle, config: Config) -> Result<Config, String> {
    let before = storage::load().settings;
    storage::save(&config)?;
    // Register/unregister start-with-Windows when the setting is toggled.
    if config.settings.autostart != before.autostart {
        crate::apply_autostart(&app, config.settings.autostart);
    }
    Ok(config)
}

/// Off the main thread: revealing a file goes through the shell, which can
/// stall on a slow or network path (a WSL share, a mapped drive).
#[tauri::command]
pub async fn open_explorer(path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || actions::open_explorer(&path.replace('/', "\\")))
        .await
        .map_err(|e| e.to_string())?
}

/// Reveals the main window, which starts hidden so the saved zoom can be
/// applied before the first paint (T-0491). The frontend calls it once the
/// zoom is in place; `lib.rs` has a timer for when it never does.
#[tauri::command]
pub fn reveal_main_window(app: tauri::AppHandle) {
    crate::reveal_main_window(&app);
}

#[tauri::command]
pub fn app_version() -> String {
    update::current_version().to_string()
}

/// The app's own diagnostic log, most recent lines last. A release build has
/// no console, so this command (and the log file it mirrors) is the only way
/// a user can see what the app recorded — see `diag.rs`.
#[tauri::command]
pub fn diagnostic_log(limit: Option<usize>) -> Vec<crate::diag::DiagEntry> {
    crate::diag::entries(limit)
}

/// Where the log file is and how large it has grown, so the panel can offer
/// to open the folder and show the user what they would be attaching to a
/// bug report.
#[tauri::command]
pub fn diagnostic_log_info() -> crate::diag::DiagLogInfo {
    crate::diag::info()
}

/// The longest frontend error the log will accept. A stack trace from a
/// bundled build is a handful of frames; anything past this is a runaway
/// message, and the log is capped by rotation rather than by retention.
const MAX_FRONTEND_ERROR: usize = 4000;

/// Records a crash the webview reported (an uncaught error, a rejected
/// promise, or a React render that an error boundary caught).
///
/// A packaged build has no console, so before this existed a frontend
/// exception left no trace at all: the window went blank and there was
/// nothing to read afterwards (T-0254). What arrives here is the error's own
/// message and stack — app behaviour, never note or task content — and it is
/// truncated so one runaway string cannot rotate the log out from under the
/// lines around it.
#[tauri::command]
pub fn log_frontend_error(context: String, message: String, stack: Option<String>) {
    let mut text = format!("frontend error [{context}]: {message}");
    if let Some(stack) = stack.filter(|s| !s.trim().is_empty()) {
        text.push('\n');
        text.push_str(&stack);
    }
    if text.chars().count() > MAX_FRONTEND_ERROR {
        text = text.chars().take(MAX_FRONTEND_ERROR).collect::<String>() + " …(truncated)";
    }
    crate::diag!("{text}");
}

#[derive(Serialize)]
pub struct UpdateInfo {
    pub tag: String,
    pub url: String,
}

/// Returns Some only when a strictly newer release exists.
#[tauri::command]
pub async fn check_update() -> Option<UpdateInfo> {
    tauri::async_runtime::spawn_blocking(|| {
        let (tag, url) = update::check_latest()?;
        if update::is_newer(&tag, update::current_version()) {
            Some(UpdateInfo { tag, url })
        } else {
            None
        }
    })
    .await
    .ok()
    .flatten()
}

#[tauri::command]
pub async fn apply_update(url: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || update::apply_update(&url))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn restart_app(app: tauri::AppHandle) {
    app.restart();
}

// ---------------------------------------------------------------------
// tasks (vault-backed)
// ---------------------------------------------------------------------

#[tauri::command]
pub async fn list_tasks(vault_path: String) -> Result<Vec<Task>, String> {
    tauri::async_runtime::spawn_blocking(move || tasks::scan_and_index(&PathBuf::from(vault_path)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn update_task(vault_path: String, input: UpdateTaskInput) -> Result<Task, String> {
    tauri::async_runtime::spawn_blocking(move || {
        tasks::update_task(&PathBuf::from(vault_path), input)
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---- schedule notes (T-0088..T-0091) -------------------------------------

/// Creates a vault project (`projects/<slug>/`) from the bundled scaffold, so
/// the schedule picker's project list can grow from inside the app (T-0178).
#[tauri::command]
pub async fn create_vault_project(
    vault_path: String,
    slug: String,
    name: String,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        vault_note::create_project(&PathBuf::from(vault_path), &slug, &name)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// The folder name `create_vault_project` will give a new project with this
/// slug — `NNNN-<slug>`, with the sort number the vault would allocate right
/// now — so the create dialog can show it before the folder exists (T-0278).
#[tauri::command]
pub async fn next_project_folder(vault_path: String, slug: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        vault_note::next_project_folder(&PathBuf::from(vault_path), &slug)
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---- vault projects (T-0190) --------------------------------------------

/// Resolves a project slug to its real folder under `projects/` — which may
/// carry a `NNNN-` sort prefix the slug itself never includes — or `None`
/// when no folder answers to it. The frontend must go through this rather
/// than string-concatenating `projects/<slug>/`, since that guess is wrong
/// for every numbered project and silently creates a second, empty one
/// (T-0379).
#[tauri::command]
pub async fn resolve_project_dir(
    vault_path: String,
    slug: String,
) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let vault = PathBuf::from(vault_path);
        let dir = vault_note::resolve_project_dir(&vault_note::projects_dir(&vault), &slug)?;
        Ok(dir.map(|p| vault_note::norm_path(&p)))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Every project folder under the vault's `projects/`, with what it holds and
/// where it departs from the documented layout. `include_archived` adds the
/// folders under `archive/projects/`.
#[tauri::command]
pub async fn list_vault_projects(
    vault_path: String,
    include_archived: bool,
) -> Result<Vec<VaultProject>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        vault_project::list_projects(&PathBuf::from(vault_path), include_archived)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Pins a project and records its manual sort position in `_index.md`
/// frontmatter. `order: null` clears the position, putting the project back
/// at the end of its group (T-0231).
#[tauri::command]
pub async fn set_vault_project_order(
    vault_path: String,
    slug: String,
    pinned: bool,
    order: Option<f64>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        vault_project::set_project_order(&PathBuf::from(vault_path), &slug, pinned, order)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Lists schedule notes, optionally narrowed to one project slug (pass an
/// empty string for "all projects").
#[tauri::command]
pub async fn list_schedules(
    vault_path: String,
    project: String,
) -> Result<Vec<ScheduleFile>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let project = (!project.trim().is_empty()).then_some(project);
        schedule::list_schedules(&PathBuf::from(vault_path), project.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn read_schedule(path: String) -> Result<ScheduleDoc, String> {
    tauri::async_runtime::spawn_blocking(move || schedule::read_schedule(&PathBuf::from(path)))
        .await
        .map_err(|e| e.to_string())?
}

/// Writes a schedule note, refusing when the file changed on disk since the
/// caller read it. Returns the new mtime for the next guarded write. Pass
/// `expected_mtime: 0` to write unconditionally.
#[tauri::command]
pub async fn write_schedule(
    path: String,
    content: String,
    expected_mtime: u64,
) -> Result<u64, String> {
    tauri::async_runtime::spawn_blocking(move || {
        schedule::write_schedule(&PathBuf::from(path), &content, expected_mtime)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn create_schedule(
    vault_path: String,
    project: String,
    title: String,
    range: String,
) -> Result<ScheduleFile, String> {
    tauri::async_runtime::spawn_blocking(move || {
        schedule::create_schedule(&PathBuf::from(vault_path), &project, &title, &range)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Renames a schedule note: its frontmatter `title` and its file name move
/// together, and its snapshot follows the new path. Returns the note at its new
/// location, so the caller can reselect it (T-0157).
#[tauri::command]
pub async fn rename_schedule(
    vault_path: String,
    path: String,
    title: String,
) -> Result<ScheduleFile, String> {
    tauri::async_runtime::spawn_blocking(move || {
        schedule::rename_schedule(&PathBuf::from(vault_path), &PathBuf::from(path), &title)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Writes a frontend-generated, self-contained HTML export to disk (T-0090).
///
/// `vault_path`/`project` are set only when `out_path` is the vault's own
/// default destination (`<vault>/projects/<slug>/attachments/...`) — never a
/// user-configured export directory — so the write can be refused rather than
/// inventing the project folder when the slug does not resolve (T-0379).
#[tauri::command]
pub async fn export_schedule_html(
    out_path: String,
    html: String,
    vault_path: Option<String>,
    project: Option<String>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        schedule::export_html(
            vault_path.as_deref().map(PathBuf::from).as_deref(),
            project.as_deref(),
            &PathBuf::from(out_path),
            &html,
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Copies the diagram-edit prompt for `path` to the clipboard, for pasting into
/// an interactive agent session. `instruction` may be empty.
///
/// The viewer ships no plugin, so the prompt names the bundled skill file and
/// its rules directory for the agent to read (T-1001).
#[tauri::command]
pub fn copy_diagram_edit_prompt(
    app: tauri::AppHandle,
    path: String,
    instruction: String,
    confirm: bool,
) -> Result<(), String> {
    let skill = diagram_edit::skill_files(&app).map(|(file, rules)| {
        (
            file.to_string_lossy().replace('\\', "/"),
            rules.to_string_lossy().replace('\\', "/"),
        )
    });
    let skill_ref = skill.as_ref().map(|(f, r)| (f.as_str(), r.as_str()));
    let prompt = diagram_edit::build_copy_prompt(
        &path.replace('\\', "/"),
        &instruction,
        confirm,
        &storage::load().settings.language,
        skill_ref,
    );
    app.clipboard()
        .write_text(prompt)
        .map_err(|e| format!("failed to copy prompt: {e}"))
}

// ---- Docs tab: read-only browsing of shared Markdown (T-0259) -----------
//
// The roots these commands are allowed to reach are read from the config
// here, never taken from the caller — see `docs::resolve_within_roots`. There
// is no write command in this section on purpose: the whole point of the tab
// is that nothing of ours ever lands in a folder the team shares.

/// A folder path as the Docs settings store it: trimmed, forward slashes,
/// and no trailing separator, so the same folder typed two ways compares
/// equal.
fn normalize_docs_path(path: &str) -> String {
    let normalized = path.trim().replace('\\', "/");
    let trimmed = normalized.trim_end_matches('/');
    // A bare drive root ("G:/") keeps its slash - "G:" alone is not a folder.
    if trimmed.ends_with(':') {
        format!("{trimmed}/")
    } else {
        trimmed.to_string()
    }
}

/// The registered roots, each resolved against this machine.
#[tauri::command]
pub fn docs_roots() -> Vec<DocsRootStatus> {
    docs::root_statuses(&storage::load().settings)
}

/// Registers a folder as a document root and returns the new list. The path
/// is stored as given (vault-scoped, shared with the team); a machine whose
/// mount differs overrides it with `set_docs_root_local_path`.
#[tauri::command]
pub fn add_docs_root(path: String, name: String) -> Result<Vec<DocsRootStatus>, String> {
    let path = normalize_docs_path(&path);
    if path.is_empty() {
        return Err("no folder given".into());
    }
    let mut cfg = storage::load();
    if cfg
        .settings
        .docs_roots
        .iter()
        .any(|r| r.path.eq_ignore_ascii_case(&path))
    {
        return Err(format!("{path} is already registered"));
    }
    let id = docs::next_root_id(&cfg.settings.docs_roots);
    cfg.settings.docs_roots.push(DocsRoot {
        id,
        name: name.trim().to_string(),
        path,
    });
    storage::save(&cfg)?;
    Ok(docs::root_statuses(&cfg.settings))
}

/// Forgets a root. Only the registration goes — nothing on the share is
/// touched, here or anywhere else in this module.
#[tauri::command]
pub fn remove_docs_root(id: String) -> Result<Vec<DocsRootStatus>, String> {
    let mut cfg = storage::load();
    cfg.settings.docs_roots.retain(|r| r.id != id);
    storage::save(&cfg)?;
    Ok(docs::root_statuses(&cfg.settings))
}

/// Updates a root's name and folder together.
///
/// They were two commands behind two buttons, which made editing a root feel
/// like two unrelated operations. One dialog writes both.
#[tauri::command]
pub fn update_docs_root(
    id: String,
    name: String,
    path: String,
) -> Result<Vec<DocsRootStatus>, String> {
    let mut cfg = storage::load();
    let path = normalize_docs_path(&path);
    if path.is_empty() {
        return Err("the shared folder path cannot be empty".into());
    }
    if cfg
        .settings
        .docs_roots
        .iter()
        .any(|r| r.id != id && r.path.eq_ignore_ascii_case(&path))
    {
        return Err(format!("{path} is already registered"));
    }
    let Some(root) = cfg.settings.docs_roots.iter_mut().find(|r| r.id == id) else {
        return Err(format!("no such document root: {id}"));
    };
    root.name = name.trim().to_string();
    root.path = path;
    storage::save(&cfg)?;
    Ok(docs::root_statuses(&cfg.settings))
}

/// Opens a file inside a registered root with whatever the OS associates with
/// it. Markdown is rendered in the tab itself; this is for everything else a
/// team share holds - PDFs, spreadsheets, images.
#[tauri::command]
pub async fn docs_open_external(path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        docs::guarded_open_external(&storage::load().settings, &path)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Shows a file or folder inside a registered root in the OS file manager,
/// selected in its parent folder.
#[tauri::command]
pub async fn docs_reveal(path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        docs::guarded_reveal(&storage::load().settings, &path)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Lists one directory inside a registered root. Never recurses — the tree
/// calls again when a folder is opened, so a streamed share is only ever
/// touched where the user is actually looking.
#[tauri::command]
pub async fn docs_list_dir(path: String) -> Result<Vec<DocsEntry>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        docs::guarded_list_dir(&storage::load().settings, &path)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn docs_read_file(path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        docs::guarded_read_doc(&storage::load().settings, &path)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Creates a diagram note inside a registered document root (T-1001).
///
/// The Docs tab's own creation path: `diagram::create_diagram` needs a vault
/// project, which the viewer does not have. `dir` must be a folder inside a
/// registered root (the same containment guard as the reads); the note gets
/// the kind's starting text and a free `<title>.md` name. Returns the new
/// note's path. Renaming and deleting stay out of the shell (T-1002).
#[tauri::command]
pub async fn docs_create_diagram(
    dir: String,
    kind: String,
    title: String,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if !diagram::KINDS.contains(&kind.as_str()) {
            return Err(format!("unknown diagram type '{kind}'"));
        }
        let settings = storage::load().settings;
        let folder = docs::resolve_within_roots(&dir, &docs::allowed_roots(&settings))?;
        if !folder.is_dir() {
            return Err(format!("{dir} is not a folder"));
        }
        let title = match title.trim() {
            "" => kind.clone(),
            t => t.to_string(),
        };
        let content = diagram::skeleton(&kind, &title, "", &vault_note::today())
            .map_err(|e| e.to_string())?;
        let path = vault_note::unique_note_path(&folder, &title, &kind, None);
        std::fs::write(&path, &content).map_err(|e| e.to_string())?;
        Ok(vault_note::norm_path(&path))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Renames a diagram note inside a registered document root (T-1002): its
/// frontmatter `title` and its file name move together. Returns the note's
/// new path, which the caller must reselect.
///
/// The vault-bound `rename_diagram` is unusable here — the viewer has no
/// vault — so this guards the path against the roots and renames through the
/// vault-free half. Anything that is not a diagram note is refused.
#[tauri::command]
pub async fn docs_rename_diagram(path: String, title: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = storage::load().settings;
        let real = docs::resolve_within_roots(&path, &docs::allowed_roots(&settings))?;
        diagram::rename_shared_diagram(&real, &title)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Moves a diagram note inside a registered document root to the OS trash
/// (T-1002). There is no vault trash to receive it, so the platform's own
/// trash is what keeps a mis-click reversible. Anything that is not a diagram
/// note is refused.
#[tauri::command]
pub async fn docs_delete_diagram(path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = storage::load().settings;
        let real = docs::resolve_within_roots(&path, &docs::allowed_roots(&settings))?;
        diagram::trash_shared_diagram(&real)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Reads an image a document embeds and returns it as a `data:` URI.
#[tauri::command]
pub async fn docs_read_asset(path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        docs::guarded_read_asset(&storage::load().settings, &path)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Resolves a pasted absolute path against this machine's roots (T-0362).
///
/// A teammate's path names a file on *their* machine; the mounts here may
/// disagree, so the backend answers with a direct hit or longest-tail
/// candidates rather than the frontend guessing at strings.
#[tauri::command]
pub async fn docs_resolve_open_path(
    pasted: String,
    current_root: Option<String>,
) -> Result<docs::OpenPathResolution, String> {
    tauri::async_runtime::spawn_blocking(move || {
        docs::guarded_resolve_open_path(
            &storage::load().settings,
            &pasted,
            current_root.as_deref().unwrap_or(""),
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Resolves an Obsidian `[[wikilink]]` written in the document `from` (T-0726).
///
/// `target` is the link's file part: the frontend splits off the `|alias` and
/// the `#heading` before calling. A bare name is looked up across the vault by
/// file name (Obsidian's rule); a `folder/name` narrows it to a path. Anything
/// the backend cannot pin to exactly one file comes back as an empty or an
/// ambiguous resolution — never a guess — and the frontend points at Obsidian
/// instead. Every returned path already passed the containment guard.
#[tauri::command]
pub async fn docs_resolve_wikilink(
    from: String,
    target: String,
) -> Result<docs::WikiLinkResolution, String> {
    tauri::async_runtime::spawn_blocking(move || {
        docs::guarded_resolve_wikilink(&storage::load().settings, &from, &target)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// The PlantUML server the Docs tab renders diagrams with; "" when rendering
/// is off (the default).
/// The starred folders and files, in the order the Shortcuts section shows
/// them.
#[tauri::command]
pub fn docs_shortcuts() -> Vec<DocsShortcut> {
    storage::load().settings.docs_shortcuts
}

/// Replaces the whole shortcut list. Adding, removing and reordering are one
/// command because the list is the order: a per-entry command would have to
/// carry a position anyway, and two of them could disagree about it.
#[tauri::command]
pub fn set_docs_shortcuts(shortcuts: Vec<DocsShortcut>) -> Result<(), String> {
    let mut cfg = storage::load();
    cfg.settings.docs_shortcuts = shortcuts;
    storage::save(&cfg)
}

/// Whether the sidebar is split into a folder tree and a file list (T-0276).
#[tauri::command]
pub fn docs_list_pane() -> bool {
    storage::load().settings.docs_list_pane
}

#[tauri::command]
pub fn set_docs_list_pane(enabled: bool) -> Result<(), String> {
    let mut cfg = storage::load();
    cfg.settings.docs_list_pane = enabled;
    storage::save(&cfg)
}

/// Whether the Docs tab loads images from `https:` URLs (T-0329).
#[tauri::command]
pub fn docs_allow_remote_images() -> bool {
    storage::load().settings.docs_allow_remote_images
}

#[tauri::command]
pub fn set_docs_allow_remote_images(enabled: bool) -> Result<(), String> {
    let mut cfg = storage::load();
    cfg.settings.docs_allow_remote_images = enabled;
    storage::save(&cfg)
}

/// Whether the Docs tab lists dot-folders and dot-files (T-0394).
#[tauri::command]
pub fn docs_show_hidden() -> bool {
    storage::load().settings.docs_show_hidden
}

#[tauri::command]
pub fn set_docs_show_hidden(enabled: bool) -> Result<(), String> {
    let mut cfg = storage::load();
    cfg.settings.docs_show_hidden = enabled;
    storage::save(&cfg)
}

#[tauri::command]
pub fn docs_plantuml_server() -> String {
    storage::load().settings.docs_plantuml_server
}

/// Sets the PlantUML server. An empty string turns rendering off.
#[tauri::command]
pub fn set_docs_plantuml_server(server: String) -> Result<(), String> {
    let server = server.trim().trim_end_matches('/').to_string();
    if !server.is_empty() && !server.starts_with("http://") && !server.starts_with("https://") {
        return Err("the PlantUML server must be an http:// or https:// URL".into());
    }
    let mut cfg = storage::load();
    cfg.settings.docs_plantuml_server = server;
    storage::save(&cfg)
}

/// Renders a PlantUML diagram on the configured server and returns the SVG.
/// Fails without contacting anything when no server is set.
#[tauri::command]
pub async fn docs_render_plantuml(source: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::plantuml::render(&storage::load().settings.docs_plantuml_server, &source)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Opens a Docs viewer window on `payload` (a document or one figure).
///
/// `async` on purpose: building a window from a synchronous command deadlocks
/// on Windows, and unlike the pre-built helper windows this one is built on
/// every call.
#[tauri::command]
pub async fn open_docs_viewer(
    app: tauri::AppHandle,
    payload: serde_json::Value,
) -> Result<(), String> {
    crate::docs_viewer::open(&app, payload)
}

/// What the calling Docs viewer window was opened to show.
#[tauri::command]
pub fn docs_viewer_payload(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
) -> Option<serde_json::Value> {
    crate::docs_viewer::payload(&app, window.label())
}

// ---- diagrams (T-0680) ----------------------------------------------------

/// Reads a diagram note of any kind: the whole text plus the mtime that guards
/// the next [`write_diagram`].
///
/// Reads are confined to the registered document roots (T-1002): the Docs tab
/// passes shared-folder paths here, and an unguarded read would reach any
/// path on the machine.
#[tauri::command]
pub async fn read_diagram(path: String) -> Result<crate::diagram::DiagramDoc, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = storage::load().settings;
        let real = docs::resolve_within_roots(&path, &docs::allowed_roots(&settings))?;
        crate::diagram::read_diagram(&real)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Writes a diagram note, refusing when the file changed on disk since the
/// caller read it (`expected_mtime` 0 skips the check). Returns the new mtime.
///
/// Same containment guard as [`read_diagram`]: a write outside the registered
/// roots is refused, and a failure (read-only share, offline folder) surfaces
/// as an error the editor shows rather than a crash (T-1002).
#[tauri::command]
pub async fn write_diagram(
    path: String,
    content: String,
    expected_mtime: u64,
) -> Result<u64, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let settings = storage::load().settings;
        let real = docs::resolve_within_roots(&path, &docs::allowed_roots(&settings))?;
        crate::diagram::write_diagram(&real, &content, expected_mtime)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Writes a frontend-generated HTML or SVG export of any diagram kind. Same
/// destination rules as [`export_mindmap_file`] (T-0379).
#[tauri::command]
pub async fn export_diagram_file(
    out_path: String,
    content: String,
    vault_path: Option<String>,
    project: Option<String>,
) -> Result<(), String> {
    export_mindmap_file(out_path, content, vault_path, project).await
}

/// Writes a frontend-rendered PNG (base64) of any diagram kind. Same
/// destination rules as [`export_mindmap_file`] (T-0379).
#[tauri::command]
pub async fn export_diagram_png(
    out_path: String,
    base64_data: String,
    vault_path: Option<String>,
    project: Option<String>,
) -> Result<(), String> {
    export_mindmap_png(out_path, base64_data, vault_path, project).await
}

// ---- mindmap notes (T-0188) ---------------------------------------------

/// Lists mindmap notes, optionally narrowed to one project slug (pass an empty
/// string for "all projects").
#[tauri::command]
pub async fn list_mindmaps(
    vault_path: String,
    project: String,
) -> Result<Vec<MindmapFile>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let project = (!project.trim().is_empty()).then_some(project);
        mindmap::list_mindmaps(&PathBuf::from(vault_path), project.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn read_mindmap(path: String) -> Result<MindmapDoc, String> {
    tauri::async_runtime::spawn_blocking(move || mindmap::read_mindmap(&PathBuf::from(path)))
        .await
        .map_err(|e| e.to_string())?
}

/// Writes a mindmap note, refusing when the file changed on disk since the
/// caller read it. Returns the new mtime for the next guarded write. Pass
/// `expected_mtime: 0` to write unconditionally.
#[tauri::command]
pub async fn write_mindmap(
    path: String,
    content: String,
    expected_mtime: u64,
) -> Result<u64, String> {
    tauri::async_runtime::spawn_blocking(move || {
        mindmap::write_mindmap(&PathBuf::from(path), &content, expected_mtime)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn create_mindmap(
    vault_path: String,
    project: String,
    title: String,
) -> Result<MindmapFile, String> {
    tauri::async_runtime::spawn_blocking(move || {
        mindmap::create_mindmap(&PathBuf::from(vault_path), &project, &title)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Renames a mindmap note: its frontmatter `title` and its file name move
/// together, and its snapshot follows the new path.
#[tauri::command]
pub async fn rename_mindmap(
    vault_path: String,
    path: String,
    title: String,
) -> Result<MindmapFile, String> {
    tauri::async_runtime::spawn_blocking(move || {
        mindmap::rename_mindmap(&PathBuf::from(vault_path), &PathBuf::from(path), &title)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Moves a schedule note into `_ai/state/schedule-trash/`. Returns where it
/// went, so the UI can tell the user where to find it.
#[tauri::command]
pub async fn delete_schedule(vault_path: String, path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        schedule::delete_schedule(&PathBuf::from(vault_path), &PathBuf::from(path))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Moves a mindmap note into `_ai/state/mindmap-trash/`. Returns where it
/// went, so the UI can tell the user where to find it.
#[tauri::command]
pub async fn delete_mindmap(vault_path: String, path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        mindmap::delete_mindmap(&PathBuf::from(vault_path), &PathBuf::from(path))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Writes a frontend-generated, self-contained HTML or SVG export to disk.
///
/// `vault_path`/`project` are set only when `out_path` is the vault's own
/// default destination (`<vault>/projects/<slug>/attachments/...`), so the
/// write can be refused rather than inventing the project folder when the
/// slug does not resolve (T-0379).
#[tauri::command]
pub async fn export_mindmap_file(
    out_path: String,
    content: String,
    vault_path: Option<String>,
    project: Option<String>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        mindmap::export_file(
            vault_path.as_deref().map(PathBuf::from).as_deref(),
            project.as_deref(),
            &PathBuf::from(out_path),
            &content,
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Writes a frontend-rendered PNG (base64, as `canvas.toDataURL` produces it).
///
/// `vault_path`/`project` are set only when `out_path` is the vault's own
/// default destination — see [`export_mindmap_file`] (T-0379).
#[tauri::command]
pub async fn export_mindmap_png(
    out_path: String,
    base64_data: String,
    vault_path: Option<String>,
    project: Option<String>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        mindmap::export_binary(
            vault_path.as_deref().map(PathBuf::from).as_deref(),
            project.as_deref(),
            &PathBuf::from(out_path),
            &base64_data,
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::normalize_docs_path;

    /// Root registration has to survive network shares (T-1002): a UNC path
    /// picked from the folder dialog must normalize to the same `//server/…`
    /// form the guard and the availability check compare against.
    #[test]
    fn normalize_keeps_unc_and_drive_roots_usable() {
        assert_eq!(
            normalize_docs_path(r"\\server\share\docs\"),
            "//server/share/docs"
        );
        assert_eq!(normalize_docs_path("//server/share/"), "//server/share");
        assert_eq!(normalize_docs_path(r"G:\docs\"), "G:/docs");
        // A bare drive root keeps its slash — "G:" alone is not a folder.
        assert_eq!(normalize_docs_path("G:/"), "G:/");
        assert_eq!(normalize_docs_path("  C:/a  "), "C:/a");
        assert_eq!(normalize_docs_path("   "), "");
    }
}
