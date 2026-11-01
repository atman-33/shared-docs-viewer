# Changelog

## 0.2.0 (unreleased)

- **First team distribution build** (T-1003). Release workflow and self-update now point at `atman-33/shared-docs-viewer` and ship `shared-docs-viewer.exe` + `shared-docs-viewer-windows-x86_64.zip`. No signing and no MSI; install from the GitHub Release zip.

## 0.171.0 (2026-10-30)

- **Diagram files now render as diagrams in the Docs tab** (T-1000, stacked on T-0998). A Markdown file whose frontmatter `type` names a diagram kind opens in the same editor view the Projects Backlog and Documents panes embed — frozen read-only with the side panel closed and no prompt-copy button, so the tab stays read-only and every other file reads exactly as before. The "AI edit running" hint stays hidden, since no agent holds the file.

## 0.170.0 (2026-10-30)

- **The Diagrams tab no longer edits with AI in the app** (T-0998). The ✨ edit button, the ⚙ agent settings and the side panel are gone: editing moved to the terminal / desktop session (Claude Code, OpenCode). Every editor toolbar — in the Diagrams tab and in the projects Backlog (and Documents) panes alike — carries the 📋 button at its right end, copying the `diagram-edit` prompt for pasting there. On-screen edits are saved before the copy, so the prompt is one click away wherever a diagram is shown. The backend runner and the `diagram-edit` skill stay as they are.
