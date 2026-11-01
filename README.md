# shared-docs-viewer

A small Windows desktop app for reading a team's shared folders — Markdown
documents and diagram notes, nothing else. One tab, no vault, no plugins.
Windows 10/11 only. Third-party origins are recorded in [NOTICE.md](NOTICE.md).

## What it is

- Register shared folders (a network drive, a Drive mount) and read them.
  Nothing is ever written into the folder except the diagram notes you edit.
- Markdown renders as documents; a Markdown file whose frontmatter `type`
  names a diagram kind renders as that diagram.
- Diagrams are editable by hand in the app. There is no in-app AI runner —
  the toolbar's copy button hands a prompt (plus the bundled skill's path) to
  an agent session you run yourself.
- Settings live in `~/.shared-docs-viewer/config.json`, and there is no
  vault to configure.

## Status

Viewer shell (T-1001): browse, preview, diagram view/create/edit, copy prompt.
Shared-folder support with UNC hardening plus diagram rename/delete (T-1002).
Team distribution build with self-update, no signing, no MSI (T-1003).

## Development

Requires Rust and Node.js 20+:

```powershell
npm install
npm run tauri:dev            # run with hot reload
npm run build                # typecheck + frontend build
npx tauri build --no-bundle  # release build -> src-tauri/target/release/shared-docs-viewer.exe
```

Tests: `npm test` (frontend). The Rust unit tests currently do not run on
this machine's toolchain setup (`cargo check` passes); see T-1001's log.

## Repository layout

```
src/            # React frontend (React 19, Tailwind v4, shadcn/ui)
src-tauri/      # Rust backend (Tauri 2)
src-tauri/resources/  # bundled diagram-edit skill + notation rules
vault-template/templates/project/  # scaffold for viewer-managed projects
```

## License

MIT.
