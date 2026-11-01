//! Headless agent edits of a diagram note (T-0091, T-0188, unified in T-0685).
//!
//! One runner for every kind of diagram (schedule, mindmap, matrix2x2, flow,
//! pfd, algorithm, usecase, architecture, ifdam): the run takes a path and nothing else,
//! because the `diagram-edit` skill reads the note's frontmatter `type` and
//! follows that kind's rule file.
//! A new kind therefore needs no change here.
//!
//! Same execution shape as `tidy.rs` - spawn the agent CLI with the prompt on
//! stdin, capture the output, report progress through a Tauri event - but with
//! a different safety story, because this run rewrites a file the user is
//! looking at and dragging things around in:
//!
//! - a snapshot is taken **before** the agent starts, so the UI can offer a
//!   one-generation undo (`diagram::restore_snapshot`);
//! - `running` doubles as an editing lock: the frontend goes read-only while a
//!   run is live, so an app write and an agent write cannot interleave;
//! - the file watcher reloads the note when the agent finishes
//!   (`diagrams-changed`), so nothing here has to push the new content back.
//!
//! The agent does the editing through the `diagram-edit` skill, which owns the
//! shared procedure (ids are never renumbered, `## Memo` is never touched). This
//! module only tells it which file and what to do.

use crate::actions::is_ja;
use std::path::{Path, PathBuf};

fn mode_line(confirm: bool, language: &str) -> &'static str {
    match (confirm, is_ja(language)) {
        (true, true) => "ファイルには書き込まないでください。変更する行をそのまま diff で報告して、そこで止まってください。",
        (true, false) => "Do NOT write the file. Report the exact lines you would change, as a diff, and stop.",
        (false, true) => "変更をファイルに適用してください。",
        (false, false) => "Apply the change to the file.",
    }
}

/// Where the bundled `diagram-edit` skill and its notation rules live (T-1001).
///
/// The viewer ships no plugin, so the copy prompt points the agent at files
/// instead: `resources/skills/diagram-edit/SKILL.md` plus
/// `resources/skill-rules/{diagrams,mindmaps,schedules}.md`. In a packaged
/// build they ride in the resource dir; in dev they sit in the repo next to
/// `src-tauri/`, found by walking up from the exe. Returns the skill file and
/// the rules directory.
pub fn skill_files(app: &tauri::AppHandle) -> Option<(PathBuf, PathBuf)> {
    use tauri::Manager;
    let under = |base: &Path| {
        let skill = base.join("skills").join("diagram-edit").join("SKILL.md");
        let rules = base.join("skill-rules");
        (skill.is_file()
            && ["diagrams.md", "mindmaps.md", "schedules.md"]
                .iter()
                .all(|f| rules.join(f).is_file()))
        .then_some((skill, rules))
    };
    let dir = app.path().resource_dir().ok()?;
    if let Some(found) = under(&dir) {
        return Some(found);
    }
    // Dev (`cargo tauri dev`): resources are not staged into the resource
    // dir, so look next to the source tree instead.
    let exe = std::env::current_exe().ok()?;
    let mut cur = exe.as_path();
    while let Some(parent) = cur.parent() {
        if let Some(found) = under(&parent.join("resources")) {
            return Some(found);
        }
        cur = parent;
    }
    None
}

/// The same prompt for pasting into an interactive session (Claude Desktop and
/// the like) by hand. `instruction` is whatever the panel holds: when it is
/// empty the owner will give instructions in the following messages, so the
/// prompt says to wait instead of editing on its own.
///
/// `skill` names the bundled skill file and its rules directory (see
/// [`skill_files`]): the viewer ships no plugin, so the prompt carries where
/// to read them. `None` keeps the bare skill name, for contexts where the
/// files are not on disk (tests).
pub fn build_copy_prompt(
    path: &str,
    instruction: &str,
    confirm: bool,
    language: &str,
    skill: Option<(&str, &str)>,
) -> String {
    let instruction = instruction.trim();
    let skill_block = skill
        .map(|(file, rules)| {
            if is_ja(language) {
                format!(
                    "\nスキルファイル: {file}\n記法ルール: {rules}\n\
先にスキルファイルを読んでください。ノートの `type` に対応するルールファイルはそこに書いてあります。\n"
                )
            } else {
                format!(
                    "\nSkill file: {file}\nNotation rules: {rules}\n\
Read the skill file first — it names the rule file for the note's `type`.\n"
                )
            }
        })
        .unwrap_or_default();
    if is_ja(language) {
        let request = if instruction.is_empty() {
            "指示はこのあとのメッセージで伝えます。それまでは何も編集しないでください。".to_string()
        } else {
            format!("指示:\n{instruction}")
        };
        return format!(
            "`diagram-edit` スキルを使って、この図解ノートを編集してください。\n\n\
図解ファイル: {path}\n{skill_block}\n\
{request}\n\n\
{mode}\n\n\
変更のたびに、変更した内容（または変更する内容）を、触った要素の id を挙げて 1 文で要約してください。他のファイルは変更しないでください。\n",
            mode = mode_line(confirm, language)
        );
    }
    let request = if instruction.is_empty() {
        "I will give you the instructions in the following messages. \
Do not edit anything until I do."
            .to_string()
    } else {
        format!("Instruction:\n{instruction}")
    };
    format!(
        "Use the `diagram-edit` skill to edit this diagram note.\n\n\
Diagram file: {path}\n{skill_block}\n\
{request}\n\n\
{mode}\n\n\
After each change, summarize in a sentence what changed (or would change), \
naming the element ids you touched. Do not modify any other file.\n",
        mode = mode_line(confirm, language)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_prompt_names_the_bundled_skill_files_when_given() {
        let skill = ("C:/app/skills/diagram-edit/SKILL.md", "C:/app/skill-rules");
        let en = build_copy_prompt("C:/v/a.md", "do it", false, "en", Some(skill));
        assert!(en.contains("Skill file: C:/app/skills/diagram-edit/SKILL.md"));
        assert!(en.contains("Notation rules: C:/app/skill-rules"));

        let ja = build_copy_prompt("C:/v/a.md", "do it", false, "ja", Some(skill));
        assert!(ja.contains("スキルファイル: C:/app/skills/diagram-edit/SKILL.md"));
        assert!(ja.contains("記法ルール: C:/app/skill-rules"));

        let bare = build_copy_prompt("C:/v/a.md", "do it", false, "en", None);
        assert!(!bare.contains("Skill file:"));
    }
}
