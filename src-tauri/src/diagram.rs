//! Diagram discovery and creation (T-0680, design: B-050 `020-T-0679`).
//!
//! A *diagram* is any note under `projects/<slug>/` whose frontmatter `type` is
//! one of [`KINDS`]. Where the file sits does not matter: the project-wide
//! `diagrams/` folder, the legacy `schedules/` and `mindmaps/` folders, and a
//! backlog item's own folder are all scanned. That is what lets the Diagrams
//! tab show Schedule, Mindmap and the newer kinds in one list.
//!
//! Only the head of each file is read (the frontmatter), never the body, so
//! listing a project with hundreds of notes stays cheap.
//!
//! Schedule and Mindmap keep reading, writing, renaming and deleting through
//! their own modules (`schedule.rs`, `mindmap.rs`: their undo history and trash
//! live in their own folders). The newer kinds go through the kind-independent
//! `read_diagram` / `write_diagram` / `rename_diagram` / `delete_diagram` here:
//! they work on a path, and the path is all the Diagrams tab needs to hand over.

use crate::vault_note::{
    frontmatter_value, mtime_secs, norm_path, rewrite_frontmatter, sanitize_filename,
    split_frontmatter, today, unique_note_path,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Every `type` the Diagrams tab lists. `schedule` and `mindmap` are the older
/// kinds; the rest arrive with the diagram series (B-050).
pub const KINDS: &[&str] = &[
    "schedule",
    "mindmap",
    "matrix2x2",
    "flow",
    "pfd",
    "algorithm",
    "usecase",
    "architecture",
    "ifdam",
];

/// Folder (under the vault root) holding one snapshot generation per diagram
/// note, so a rename or delete carries or drops it with the note.
pub const SNAPSHOT_DIR: &str = "diagram-snapshots";

/// The text of a new note of `kind`. The 2x2 matrix starts empty; the flow and
/// the PFD start with a small working example.
///
/// `pub(crate)` for the Docs tab's folder-local creation (`docs_create_diagram`),
/// which needs the same starting text without a vault project around it (T-1001).
pub(crate) fn skeleton(kind: &str, title: &str, range: &str, now: &str) -> Result<String, String> {
    Ok(match kind {
        "schedule" => crate::schedule::skeleton(title, range, now),
        "mindmap" => crate::mindmap::skeleton(title, now),
        // The axis and quadrant labels start empty (an empty one is not drawn);
        // the keys are there so a person or an agent sees what can be filled.
        "matrix2x2" => format!(
            "---\ntype: matrix2x2\ntitle: {title}\ncreated: {now}\nupdated: {now}\n\
x_axis:\nx_low:\nx_high:\ny_axis:\ny_low:\ny_high:\nq_tl:\nq_tr:\nq_bl:\nq_br:\n---\n\n\
## Items\n\n## Memo\n\n"
        ),
        // A small working flow rather than empty sections: on a blank
        // canvas there is nothing to double-click, and the three steps show
        // what a lane, a terminator and an arrow look like.
        "flow" => format!(
            "---\ntype: flow\ntitle: {title}\ncreated: {now}\nupdated: {now}\n---\n\n\
## Lanes\n\n- L-001 Lane 1\n\n\
## Steps\n\n- F-001 Start ^start lane:L-001\n- F-002 Step lane:L-001\n- F-003 End ^end lane:L-001\n\n\
## Edges\n\n- F-001 -> F-002\n- F-002 -> F-003\n\n\
## Memo\n\n"
        ),
        // One process, the deliverable it produces and the arrow between them:
        // the two symbols and the one connection a PFD is made of. The positions
        // are left to the layout.
        "pfd" => format!(
            "---\ntype: pfd\ntitle: {title}\ncreated: {now}\nupdated: {now}\n---\n\n\
## Nodes\n\n- P-001 Process\n- D-001 Deliverable\n\n\
## Edges\n\n- P-001 -> D-001\n\n\
## Memo\n\n"
        ),
        // A small working chart: the start and end terminals and one step
        // between them, so a blank canvas has something to build on. The
        // positions are left to the layout.
        "algorithm" => format!(
            "---\ntype: algorithm\ntitle: {title}\ncreated: {now}\nupdated: {now}\n---\n\n\
## Nodes\n\n- A-001 Start ^start\n- A-002 Step\n- A-003 End ^end\n\n\
## Edges\n\n- A-001 -> A-002\n- A-002 -> A-003\n\n\
## Memo\n\n"
        ),
        // One system in the middle, a person with two things to do (the lines
        // under a person are its speech bubble) and the line between them. The
        // positions are left to the ring layout.
        "usecase" => format!(
            "---\ntype: usecase\ntitle: {title}\ncreated: {now}\nupdated: {now}\n---\n\n\
## Nodes\n\n- U-001 System ^system\n- U-002 Person\n  Do something\n  Do something else\n\n\
## Edges\n\n- U-002 -- U-001\n\n\
## Memo\n\n"
        ),
        // Two frames with two blocks each, joined across the frame border, plus
        // a database and a cloud service outside. The positions are left to
        // the group layout.
        "architecture" => format!(
            "---\ntype: architecture\ntitle: {title}\ncreated: {now}\nupdated: {now}\n---\n\n\
## Frames\n\n- G-001 Client #blue\n- G-002 Server #green\n\n\
## Nodes\n\n- C-001 Screen frame:G-001\n- C-002 API frame:G-002\n- C-003 DB frame:G-002 ^db\n- C-004 Mailer ^cloud\n\n\
## Edges\n\n- C-001 -> C-002\n- C-002 <-> C-003 \"HTTPS\"\n- C-002 -> C-004\n\n\
## Memo\n\n"
        ),
        // The smallest working figure: a screen with one item in each of its
        // sections, the trigger on it, the process behind it and a data store the
        // process writes to. Items are continuation lines, one per line. The
        // positions are left to the layout.
        "ifdam" => format!(
            "---\ntype: ifdam\ntitle: {title}\ncreated: {now}\nupdated: {now}\n---\n\n\
## Nodes\n\n- V-001 Screen ^screen\n  show: Item to show\n  input: Item to enter\n  action: Button\n\
- V-002 Click the button ^trigger\n- V-003 Process\n- V-004 Data ^store\n\n\
## Edges\n\n- V-001 -> V-002\n- V-002 -> V-003\n- V-003 -> V-004\n\n\
## Memo\n\n"
        ),
        other => return Err(format!("unknown diagram type '{other}'")),
    })
}

/// A diagram note's full text plus the mtime that guards the next write.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramDoc {
    pub path: String,
    pub content: String,
    /// Unix seconds; pass back to [`write_diagram`] for conflict detection.
    pub mtime: u64,
}

pub fn read_diagram(path: &Path) -> Result<DiagramDoc, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    Ok(DiagramDoc {
        path: norm_path(path),
        content,
        mtime: mtime_secs(path),
    })
}

/// Minimal structural check before a write. Deliberately shallow: each kind's
/// grammar is validated on the frontend, and rejecting a file here for a
/// notation slip would block the user from saving their way out of it. What
/// this *does* catch is a caller about to write something that is not a
/// diagram note at all (a truncated string, an empty buffer from a failed
/// render), which would silently destroy the file.
fn validate(content: &str) -> Result<(), String> {
    let Some((front, _)) = split_frontmatter(content) else {
        return Err("diagram content must start with a frontmatter block".into());
    };
    let kind = frontmatter_value(&front, "type");
    if !KINDS.contains(&kind.as_str()) {
        return Err(format!(
            "diagram content has no valid `type` (found '{kind}')"
        ));
    }
    Ok(())
}

/// Writes the file only when its on-disk mtime still matches `expected_mtime`,
/// so a concurrent Obsidian/agent edit is reported instead of overwritten.
/// Pass `0` to skip the check. Returns the new mtime so the caller can keep
/// guarding subsequent writes without a re-read.
pub fn write_diagram(path: &Path, content: &str, expected_mtime: u64) -> Result<u64, String> {
    validate(content)?;
    if expected_mtime != 0 {
        // A guarded write is an edit of a note that was read. If the note has
        // gone (renamed, moved, deleted), writing would quietly recreate it
        // under the old name.
        if !path.exists() {
            return Err("the diagram file is gone — it was renamed, moved or deleted".into());
        }
        if mtime_secs(path) != expected_mtime {
            return Err(
                "the diagram file changed on disk since it was loaded — reload before saving"
                    .into(),
            );
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(mtime_secs(path))
}

/// The vault-free half of [`rename_diagram`]: validates the note, rewrites its
/// frontmatter and moves the file. Returns the note's new path.
///
/// The viewer's Docs tab renames through this (T-1002): there is no vault, so
/// no snapshot to move and no project listing entry to describe.
pub fn rename_shared_diagram(path: &Path, new_title: &str) -> Result<String, String> {
    Ok(norm_path(&rename_diagram_file(path, new_title)?))
}

/// The frontmatter `type` of the diagram at `path`, or an error when the file
/// is not a diagram note.
pub fn kind_of(path: &Path) -> Result<String, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let Some((front, _)) = split_frontmatter(&content) else {
        return Err("this file is not a diagram note (no frontmatter block)".into());
    };
    let kind = frontmatter_value(&front, "type");
    if !KINDS.contains(&kind.as_str()) {
        return Err(format!(
            "this file is not a diagram note (type '{kind}' is not one of {})",
            KINDS.join(", ")
        ));
    }
    Ok(kind)
}

/// Moves a diagram note inside a shared folder to the OS trash (T-1002).
/// Unlike [`delete_diagram`] there is no vault trash to receive it, so the
/// platform's own trash is what keeps a mis-click reversible.
pub fn trash_shared_diagram(path: &Path) -> Result<(), String> {
    // Only a diagram note goes through here — anything else is refused before
    // anything is moved.
    kind_of(path)?;
    if !path.is_file() {
        return Err("this diagram no longer exists".into());
    }
    trash::delete(path).map_err(|e| e.to_string())
}

fn rename_diagram_file(path: &Path, new_title: &str) -> Result<PathBuf, String> {
    let title = new_title.trim();
    if title.is_empty() {
        return Err("a diagram name is required".into());
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let Some((front, body)) = split_frontmatter(&content) else {
        return Err("this file is not a diagram note (no frontmatter block)".into());
    };
    let kind = frontmatter_value(&front, "type");
    if !KINDS.contains(&kind.as_str()) {
        return Err("this file is not a diagram note (no valid `type`)".into());
    }
    let dir = path
        .parent()
        .ok_or_else(|| "the diagram path has no parent folder".to_string())?;

    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let digits: String = stem.chars().take_while(|c| c.is_ascii_digit()).collect();
    let name = if !digits.is_empty() && stem[digits.len()..].starts_with('-') {
        format!("{digits}-{}", sanitize_filename(title, &kind))
    } else {
        title.to_string()
    };
    let target = unique_note_path(dir, &name, &kind, Some(path));

    let now = today();
    let front_out = rewrite_frontmatter(&front, &[("title", title), ("updated", &now)]);
    fs::write(path, format!("---\n{front_out}---\n{body}")).map_err(|e| e.to_string())?;
    // Compared exactly, not case-insensitively: renaming "ideas" to "Ideas" is
    // a real rename, even though the two names collide on Windows.
    if norm_path(&target) != norm_path(path) {
        fs::rename(path, &target).map_err(|e| e.to_string())?;
    }
    Ok(target)
}

// ---------------------------------------------------------------------
// snapshots (undo for AI edits)
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_vault(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("viewer-diagram-{name}-{nanos}"));
        fs::create_dir_all(dir.join("projects/0010-demo/backlog/B-001-item")).unwrap();
        fs::create_dir_all(dir.join("projects/0020-other")).unwrap();
        dir
    }

    fn skeleton_file(vault: &Path, kind: &str, title: &str) -> PathBuf {
        let content = skeleton(kind, title, "", "2026-01-01").unwrap();
        let path = vault.join("notes").join(format!("{title}.md"));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn matrix_skeleton_carries_empty_label_keys_and_an_items_section() {
        let vault = temp_vault("matrix-skeleton");
        let file = skeleton_file(&vault, "matrix2x2", "Priorities");
        let doc = read_diagram(&file).unwrap();
        let (front, body) = split_frontmatter(&doc.content).unwrap();
        assert_eq!(frontmatter_value(&front, "type"), "matrix2x2");
        assert_eq!(frontmatter_value(&front, "title"), "Priorities");
        for key in [
            "x_axis", "x_low", "x_high", "y_axis", "y_low", "y_high", "q_tl", "q_tr", "q_bl",
            "q_br",
        ] {
            assert!(front.contains(&format!("{key}:")), "missing {key}");
            assert_eq!(frontmatter_value(&front, key), "");
        }
        assert!(body.contains("## Items"));
        assert!(body.contains("## Memo"));
        // Written back as-is, it is still a valid diagram.
        assert!(write_diagram(&file, &doc.content, doc.mtime).is_ok());
        fs::remove_dir_all(&vault).ok();
    }

    #[test]
    fn flow_skeleton_is_a_small_working_flow() {
        let vault = temp_vault("flow-skeleton");
        let file = skeleton_file(&vault, "flow", "Order");
        let doc = read_diagram(&file).unwrap();
        let (front, body) = split_frontmatter(&doc.content).unwrap();
        assert_eq!(frontmatter_value(&front, "type"), "flow");
        assert_eq!(frontmatter_value(&front, "title"), "Order");
        // Every managed section is there, in the order the editor writes them.
        let at = |h: &str| body.find(h).unwrap_or_else(|| panic!("missing {h}"));
        assert!(at("## Lanes") < at("## Steps"));
        assert!(at("## Steps") < at("## Edges"));
        assert!(at("## Edges") < at("## Memo"));
        assert!(body.contains("- L-001 Lane 1"));
        assert!(body.contains("- F-001 Start ^start lane:L-001"));
        assert!(body.contains("- F-003 End ^end lane:L-001"));
        assert!(body.contains("- F-002 -> F-003"));
        // Written back as-is, it is still a valid diagram.
        assert!(write_diagram(&file, &doc.content, doc.mtime).is_ok());
        fs::remove_dir_all(&vault).ok();
    }

    #[test]
    fn pfd_skeleton_is_a_process_a_deliverable_and_an_arrow() {
        let vault = temp_vault("pfd-skeleton");
        let file = skeleton_file(&vault, "pfd", "Dev");
        let doc = read_diagram(&file).unwrap();
        let (front, body) = split_frontmatter(&doc.content).unwrap();
        assert_eq!(frontmatter_value(&front, "type"), "pfd");
        assert_eq!(frontmatter_value(&front, "title"), "Dev");
        // Every managed section is there, in the order the editor writes them.
        let at = |h: &str| body.find(h).unwrap_or_else(|| panic!("missing {h}"));
        assert!(at("## Nodes") < at("## Edges"));
        assert!(at("## Edges") < at("## Memo"));
        assert!(body.contains("- P-001 Process\n"));
        assert!(body.contains("- D-001 Deliverable\n"));
        assert!(body.contains("- P-001 -> D-001\n"));
        // Written back as-is, it is still a valid diagram.
        assert!(write_diagram(&file, &doc.content, doc.mtime).is_ok());
        fs::remove_dir_all(&vault).ok();
    }

    #[test]
    fn algorithm_skeleton_is_a_start_a_step_an_end_and_two_arrows() {
        let vault = temp_vault("algorithm-skeleton");
        let file = skeleton_file(&vault, "algorithm", "Order");
        assert!(file.ends_with("notes/Order.md"));
        assert_eq!(kind_of(&file).unwrap(), "algorithm");
        let doc = read_diagram(&file).unwrap();
        let (front, body) = split_frontmatter(&doc.content).unwrap();
        assert_eq!(frontmatter_value(&front, "type"), "algorithm");
        assert_eq!(frontmatter_value(&front, "title"), "Order");
        let at = |h: &str| body.find(h).unwrap_or_else(|| panic!("missing {h}"));
        assert!(at("## Nodes") < at("## Edges"));
        assert!(at("## Edges") < at("## Memo"));
        assert!(body.contains("- A-001 Start ^start\n"));
        assert!(body.contains("- A-003 End ^end\n"));
        assert!(body.contains("- A-001 -> A-002\n"));
        assert!(body.contains("- A-002 -> A-003\n"));
        // Written back as-is, it is still a diagram.
        assert!(write_diagram(&file, &doc.content, doc.mtime).is_ok());
        fs::remove_dir_all(&vault).ok();
    }

    #[test]
    fn usecase_skeleton_is_a_system_a_person_with_two_actions_and_a_line() {
        let vault = temp_vault("usecase-skeleton");
        let file = skeleton_file(&vault, "usecase", "Booking");
        assert!(file.ends_with("notes/Booking.md"));
        assert_eq!(kind_of(&file).unwrap(), "usecase");
        let doc = read_diagram(&file).unwrap();
        let (front, body) = split_frontmatter(&doc.content).unwrap();
        assert_eq!(frontmatter_value(&front, "type"), "usecase");
        assert_eq!(frontmatter_value(&front, "title"), "Booking");
        let at = |h: &str| body.find(h).unwrap_or_else(|| panic!("missing {h}"));
        assert!(at("## Nodes") < at("## Edges"));
        assert!(at("## Edges") < at("## Memo"));
        assert!(body.contains("- U-001 System ^system\n"));
        // A person has no mark; its indented lines are what it does.
        assert!(body.contains("- U-002 Person\n  Do something\n  Do something else\n"));
        assert!(body.contains("- U-002 -- U-001\n"));
        // Written back as-is, it is still a diagram.
        assert!(write_diagram(&file, &doc.content, doc.mtime).is_ok());
        fs::remove_dir_all(&vault).ok();
    }

    #[test]
    fn architecture_skeleton_is_two_frames_four_blocks_and_three_arrows() {
        let vault = temp_vault("architecture-skeleton");
        let file = skeleton_file(&vault, "architecture", "Booking");
        assert!(file.ends_with("notes/Booking.md"));
        assert_eq!(kind_of(&file).unwrap(), "architecture");
        let doc = read_diagram(&file).unwrap();
        let (front, body) = split_frontmatter(&doc.content).unwrap();
        assert_eq!(frontmatter_value(&front, "type"), "architecture");
        assert_eq!(frontmatter_value(&front, "title"), "Booking");
        let at = |h: &str| body.find(h).unwrap_or_else(|| panic!("missing {h}"));
        assert!(at("## Frames") < at("## Nodes"));
        assert!(at("## Nodes") < at("## Edges"));
        assert!(at("## Edges") < at("## Memo"));
        assert!(body.contains("- G-001 Client #blue\n"));
        assert!(body.contains("- G-002 Server #green\n"));
        assert!(body.contains("- C-001 Screen frame:G-001\n"));
        assert!(body.contains("- C-003 DB frame:G-002 ^db\n"));
        assert!(body.contains("- C-001 -> C-002\n"));
        assert!(body.contains("- C-002 <-> C-003 \"HTTPS\"\n"));
        // Written back as-is, it is still a diagram.
        assert!(write_diagram(&file, &doc.content, doc.mtime).is_ok());
        fs::remove_dir_all(&vault).ok();
    }

    #[test]
    fn ifdam_skeleton_is_a_screen_with_its_sections_a_trigger_a_process_and_a_store() {
        let vault = temp_vault("ifdam-skeleton");
        let file = skeleton_file(&vault, "ifdam", "Order");
        assert!(file.ends_with("notes/Order.md"));
        assert_eq!(kind_of(&file).unwrap(), "ifdam");
        let doc = read_diagram(&file).unwrap();
        let (front, body) = split_frontmatter(&doc.content).unwrap();
        assert_eq!(frontmatter_value(&front, "type"), "ifdam");
        assert_eq!(frontmatter_value(&front, "title"), "Order");
        let at = |h: &str| body.find(h).unwrap_or_else(|| panic!("missing {h}"));
        assert!(at("## Nodes") < at("## Edges"));
        assert!(at("## Edges") < at("## Memo"));
        // The screen's items follow its line, one per line, in the order of the sections.
        assert!(body.contains(
            "- V-001 Screen ^screen\n  show: Item to show\n  input: Item to enter\n  action: Button\n- V-002 Click the button ^trigger\n"
        ));
        assert!(body.contains("- V-003 Process\n"));
        assert!(body.contains("- V-004 Data ^store\n"));
        assert!(body.contains("- V-001 -> V-002\n"));
        assert!(body.contains("- V-002 -> V-003\n"));
        assert!(body.contains("- V-003 -> V-004\n"));
        // Written back as-is, it is still a diagram.
        assert!(write_diagram(&file, &doc.content, doc.mtime).is_ok());
        fs::remove_dir_all(&vault).ok();
    }

    #[test]
    fn write_refuses_a_stale_mtime_and_a_non_diagram() {
        let vault = temp_vault("write");
        let file = skeleton_file(&vault, "matrix2x2", "Grid");
        let path = &file;
        let doc = read_diagram(path).unwrap();

        let changed = doc.content.replace("## Items", "## Items\n\n- M-001 a");
        let mtime = write_diagram(path, &changed, doc.mtime).unwrap();
        assert!(fs::read_to_string(path).unwrap().contains("M-001"));
        // A guard from before another writer touched the file is refused.
        assert!(write_diagram(path, &changed, mtime + 5).is_err());
        // 0 skips the check.
        assert!(write_diagram(path, &changed, 0).is_ok());

        // A guarded write to a note that has vanished does not recreate it.
        let gone = path.with_file_name("gone.md");
        assert!(write_diagram(&gone, &changed, mtime).is_err());
        assert!(!gone.exists());

        assert!(write_diagram(path, "no frontmatter", 0).is_err());
        assert!(write_diagram(path, "---\ntype: note\n---\n", 0).is_err());
        assert!(write_diagram(path, "---\ntitle: x\n---\n", 0).is_err());
        fs::remove_dir_all(&vault).ok();
    }

    /// The Docs tab's vault-free rename/delete path (T-1002): no vault, no
    /// project — a plain folder stands in for the shared folder.
    #[test]
    fn shared_rename_moves_title_and_name_and_refuses_non_diagrams() {
        let dir = std::env::temp_dir().join(format!("shared-rename-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("Old.md");
        fs::write(
            &path,
            "---\ntype: mindmap\ntitle: Old\nupdated: 2026-10-31\n---\n\n## Nodes\n\n- N-001 root\n",
        )
        .unwrap();

        let moved = rename_shared_diagram(&path, "New name").unwrap();
        assert!(moved.ends_with("New name.md"), "{moved}");
        assert!(!path.exists());
        let (front, _) = split_frontmatter(&fs::read_to_string(&moved).unwrap()).unwrap();
        assert_eq!(frontmatter_value(&front, "title"), "New name");

        // Plain Markdown is not a diagram: neither command touches it.
        let plain = dir.join("plain.md");
        fs::write(&plain, "# just prose\n").unwrap();
        assert!(rename_shared_diagram(&plain, "Other").is_err());
        assert!(trash_shared_diagram(&plain).is_err());
        assert!(plain.is_file());
        fs::remove_dir_all(&dir).ok();
    }
}
