# shared-docs-viewer

shared-docs-viewer — a small Windows desktop app (Tauri 2 + React 19 +
Tailwind v4 + shadcn/ui) for reading a team's shared folders: Markdown
documents and diagram notes, nothing else. One tab, no vault, no plugins.
Third-party origins are recorded in [NOTICE.md](NOTICE.md).

## Commands

```powershell
npm install                # frontend dependencies (first time)
npm run tauri:dev          # run the app with hot reload
npm run build              # typecheck + frontend build
npx tauri build --no-bundle # release build -> src-tauri/target/release/shared-docs-viewer.exe

# in src-tauri/
cargo test                 # unit tests
cargo fmt                  # format (CI enforces --check)
cargo clippy -- -D warnings # lint (CI enforces)
```

## Architecture

Backend (`src-tauri/src/`), docs-first:

| Module | Responsibility |
|---|---|
| `lib.rs` / `main.rs` | Tauri builder, command registration |
| `commands.rs` | `#[tauri::command]` layer exposed to the frontend |
| `models.rs` | domain types (`Task`, `VaultProject`, `Settings`, `Config`, ...) |
| `docs.rs` | shared-folder roots, directory listing, file/asset reads |
| `docs_viewer.rs` | popped-out docs viewer windows |
| `diagram.rs` | diagram notes: read/write/rename/delete within the registered roots |
| `diagram_edit.rs` | copy-prompt builder for the bundled `diagram-edit` skill |
| `mindmap.rs` / `schedule.rs` | mindmap and schedule diagram kinds (notes + export) |
| `vault_note.rs` / `vault_project.rs` | note frontmatter helpers and project folder layout |
| `tasks.rs` | task-note parsing for the schedule/linked-task display |
| `plantuml.rs` | PlantUML server rendering |
| `update.rs` | self-update against GitHub Releases |
| `storage.rs` | JSON persistence (`~/.shared-docs-viewer/config.json`) |

Frontend (`src/`): `lib/api.ts` wraps `invoke()` with types matching the
Rust structs. Blocking work runs via `tauri::async_runtime::spawn_blocking`
in commands — never on the UI thread.

Only diagram notes are ever written back, and only inside a registered
shared folder. Everything else is read-only.

## Workflow

- Do all development on a branch cut from `main`, named
   `<type>/<task-id>-<short-name>` — `<type>` is `feature`, `fix`, `docs` or
   `chore` (never `feat`), the task id lowercase, e.g.
   `feature/t-1020-icon-refresh`. Open a PR into `main` when done. Don't
   commit directly to `main`.
- Any change that alters app behavior bumps `version` in
  `src-tauri/Cargo.toml` (semver) and adds a `CHANGELOG.md` entry in the same
  PR.

## Rules

- Conventional Commits (`feat:`, `fix:`, `chore:`, `docs:`, `refactor:`).
- Before committing: `cargo fmt`, `cargo clippy -- -D warnings`,
  `cargo test`, and `npm run build` (typecheck) must pass.
- Shell out to the `git` CLI for git operations; do not introduce libgit2.

<important>
- Never write to a shared folder except through the diagram-note commands;
  Markdown/text notes stay read-only.
- Keep `crate-type` in `src-tauri/Cargo.toml` as plain rlib (no cdylib) —
  cdylib breaks windows-gnu debug builds ("export ordinal too large").
</important>
