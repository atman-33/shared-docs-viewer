# shared-docs-viewer

A small Windows desktop app for reading a team's shared folders — Markdown
documents and diagram notes, nothing else. One tab, no vault, no plugins.

Forked from [workhub](https://github.com/atman-33/workhub) (its docs/diagrams
rendering), stripped down to viewing. Windows 10/11 only.

## What it is

- Register shared folders (a network drive, a Drive mount) and read them.
  Nothing is ever written into the folder except the diagram notes you edit.
- Markdown renders as documents; a Markdown file whose frontmatter `type`
  names a diagram kind renders as that diagram.
- Diagrams are editable by hand in the app. There is no in-app AI runner —
  the toolbar's copy button hands a prompt (plus the bundled skill's path) to
  an agent session you run yourself.
- Settings live in `~/.shared-docs-viewer/config.json` — separate from
  workhub's, and there is no vault to configure.

## Status

Viewer shell (T-1001): browse, preview, diagram view/create/edit, copy prompt.
Not yet here: diagram rename/delete, shared-folder hardening, packaged
distribution. See the roadmap in the project vault.

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
docs/           # workhub-era docs (being replaced; do not trust yet)
```

## License

MIT.
