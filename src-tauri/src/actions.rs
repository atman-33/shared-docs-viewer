use std::process::Command;

// std has no "append raw args" helper that splits a template string, so do a
// minimal quote-aware split and pass each token as an arg.

/// Whether prompts are written in Japanese. Anything but `ja` is English,
/// so a prompt never ends up in a language the builders have no text for.
pub fn is_ja(code: &str) -> bool {
    code == "ja"
}

/// Quote-aware split: `a "b c" d` -> ["a", "b c", "d"].
#[cfg(test)]
fn split_command_line(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for ch in line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            c if c.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

// ---------------------------------------------------------------------
// Claude Desktop deep links (T-0095)
// ---------------------------------------------------------------------

/// What "show in Explorer" does with a path.
#[derive(Debug, PartialEq, Eq)]
enum ExplorerTarget {
    /// Open the folder itself.
    OpenFolder,
    /// Open the parent folder with the item selected.
    RevealItem,
}

/// A file is revealed, anything else is opened. `explorer <file>` does not
/// show the file: it launches it with its associated app, which turns every
/// "Show in Explorer" handed a file into "Open". A missing path falls through
/// to `explorer` unchanged, so nothing that worked before starts failing.
fn explorer_target(path: &std::path::Path) -> ExplorerTarget {
    if path.is_file() {
        ExplorerTarget::RevealItem
    } else {
        ExplorerTarget::OpenFolder
    }
}

/// Shows a path in Explorer: a folder is opened, a file is selected in its
/// parent folder (the opener plugin passes `/select` properly).
pub fn open_explorer(path: &str) -> Result<(), String> {
    match explorer_target(std::path::Path::new(path)) {
        ExplorerTarget::RevealItem => {
            tauri_plugin_opener::reveal_item_in_dir(path).map_err(|e| e.to_string())
        }
        ExplorerTarget::OpenFolder => Command::new("explorer")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explorer_reveals_files_and_opens_everything_else() {
        let dir = std::env::temp_dir().join(format!("explorer-target-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("capture.png");
        std::fs::write(&file, b"x").unwrap();

        assert_eq!(explorer_target(&file), ExplorerTarget::RevealItem);
        assert_eq!(explorer_target(&dir), ExplorerTarget::OpenFolder);
        assert_eq!(
            explorer_target(&dir.join("missing.html")),
            ExplorerTarget::OpenFolder
        );

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn splits_quoted_arguments() {
        assert_eq!(
            split_command_line(r#"wt -d "C:/My Projects/app" pwsh"#),
            vec!["wt", "-d", "C:/My Projects/app", "pwsh"]
        );
    }
}
