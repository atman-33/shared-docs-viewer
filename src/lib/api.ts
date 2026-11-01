import { invoke } from "@tauri-apps/api/core";
import type {
  Config,
  DiagEntry,
  DiagLogInfo,
  DiagramDoc,
  DocsEntry,
  DocsOpenPathResolution,
  DocsRootStatus,
  DocsShortcut,
  DocsViewerPayload,
  MindmapDoc,
  MindmapFile,
  ScheduleDoc,
  ScheduleFile,
  Task,
  UpdateInfo,
  UpdateTaskInput,
  VaultProject,
  WikiLinkResolution,
} from "@/types";

export const api = {
  getConfig: () => invoke<Config>("get_config"),
  /// Returns the config that took effect — it differs from what was sent
  /// when the vault changed and its own settings were adopted (T-0206).
  saveConfig: (config: Config) => invoke<Config>("save_config", { config }),
  /// Saves a few settings without touching the rest. The patch is merged into
  /// a fresh read of the config, never into a copy a view loaded earlier —
  /// every save writes the whole struct, so a stale copy would revert whatever
  /// another tab changed meanwhile (T-0281).
  patchSettings: async (patch: Partial<Config["settings"]>) => {
    const cfg = await api.getConfig();
    return api.saveConfig({ ...cfg, settings: { ...cfg.settings, ...patch } });
  },
  // ---- vault tidy (T-0050) ----
  openExplorer: (path: string) => invoke<void>("open_explorer", { path }),
  appVersion: () => invoke<string>("app_version"),
  // ---- global keyboard listener health (ink / clips gestures) ----
  diagnosticLog: (limit?: number) => invoke<DiagEntry[]>("diagnostic_log", { limit }),
  diagnosticLogInfo: () => invoke<DiagLogInfo>("diagnostic_log_info"),
  /** Records a webview crash into the diagnostic log. A packaged build has no
   *  console, so without this a frontend exception left nothing to read after
   *  the window went blank (T-0254). Never rejects: a failed report must not
   *  become a second error on top of the one being reported. */
  logFrontendError: (context: string, message: string, stack?: string) =>
    invoke<void>("log_frontend_error", { context, message, stack }).catch(() => {}),
  checkUpdate: () => invoke<UpdateInfo | null>("check_update"),
  applyUpdate: (url: string) => invoke<void>("apply_update", { url }),
  restartApp: () => invoke<void>("restart_app"),
  listTasks: (vaultPath: string) => invoke<Task[]>("list_tasks", { vaultPath }),
  updateTask: (vaultPath: string, input: UpdateTaskInput) =>
    invoke<Task>("update_task", { vaultPath, input }),
  listVaultProjects: (vaultPath: string, includeArchived: boolean) =>
    invoke<VaultProject[]>("list_vault_projects", { vaultPath, includeArchived }),
  /** The backlog items of one project, for the task editor's item picker.
   * Empty rather than an error when the project has no `backlog/` folder. */
  setVaultProjectOrder: (
    vaultPath: string,
    slug: string,
    pinned: boolean,
    order: number | null,
  ) => invoke<void>("set_vault_project_order", { vaultPath, slug, pinned, order }),
  // ---- schedule notes (projects/<slug>/schedules/*.md) ----
  /** The project's real folder under `projects/` — which may carry a
   * `NNNN-` sort prefix the slug itself never includes — or `null` when no
   * folder answers to the slug. Never build `projects/<slug>/` by string
   * concatenation for a path used in file I/O; go through this (T-0379). */
  resolveProjectDir: (vaultPath: string, slug: string) =>
    invoke<string | null>("resolve_project_dir", { vaultPath, slug }),
  /** Creates `projects/NNNN-<slug>/` from the bundled scaffold, filling its
   * placeholders (`name` falls back to the slug when empty). The `NNNN-`
   * sort number is assigned automatically (T-0178, numbered folders T-0278). */
  createVaultProject: (vaultPath: string, slug: string, name: string) =>
    invoke<void>("create_vault_project", { vaultPath, slug, name }),
  /** The folder name `createVaultProject` will give a new project with this
   * slug — `NNNN-<slug>` — so the create dialog can preview it before the
   * folder exists. Rejects the same slugs `createVaultProject` would
   * (T-0278). */
  nextProjectFolder: (vaultPath: string, slug: string) =>
    invoke<string>("next_project_folder", { vaultPath, slug }),
  /** `project` narrows to one project slug; pass "" for every project. */
  listSchedules: (vaultPath: string, project = "") =>
    invoke<ScheduleFile[]>("list_schedules", { vaultPath, project }),
  readSchedule: (path: string) => invoke<ScheduleDoc>("read_schedule", { path }),
  /** Returns the new mtime. Rejects when the file changed on disk since it was
   * read; pass `expectedMtime: 0` to write unconditionally. */
  writeSchedule: (path: string, content: string, expectedMtime: number) =>
    invoke<number>("write_schedule", { path, content, expectedMtime }),
  createSchedule: (vaultPath: string, project: string, title: string, range: string) =>
    invoke<ScheduleFile>("create_schedule", { vaultPath, project, title, range }),
  /** Renames a note: frontmatter `title` and file name move together. Returns
   * the note at its new path, which the caller must reselect. */
  renameSchedule: (vaultPath: string, path: string, title: string) =>
    invoke<ScheduleFile>("rename_schedule", { vaultPath, path, title }),
  /** Moves the note into `_ai/state/schedule-trash/` rather than deleting it,
   * and returns where it went. */
  deleteSchedule: (vaultPath: string, path: string) =>
    invoke<string>("delete_schedule", { vaultPath, path }),
  /** `guard`, when given, must name the vault and project the destination
   * was resolved into — the vault's own default `attachments/` folder. It is
   * omitted for a user-configured export directory (`schedule_export_dir`),
   * which is not under any project and needs no such check. Passing it lets
   * the backend refuse the write rather than inventing the project folder,
   * even if the frontend's own resolution were somehow stale (T-0379). */
  exportScheduleHtml: (
    outPath: string,
    html: string,
    guard?: { vaultPath: string; project: string },
  ) =>
    invoke<void>("export_schedule_html", {
      outPath,
      html,
      vaultPath: guard?.vaultPath,
      project: guard?.project,
    }),
  copyDiagramEditPrompt: (path: string, instruction: string, confirm: boolean) =>
    invoke<void>("copy_diagram_edit_prompt", { path, instruction, confirm }),
  docsRoots: () => invoke<DocsRootStatus[]>("docs_roots"),
  addDocsRoot: (path: string, name: string) =>
    invoke<DocsRootStatus[]>("add_docs_root", { path, name }),
  removeDocsRoot: (id: string) => invoke<DocsRootStatus[]>("remove_docs_root", { id }),
  /** Name and folder are written together — they are edited in one dialog. */
  updateDocsRoot: (id: string, name: string, path: string) =>
    invoke<DocsRootStatus[]>("update_docs_root", { id, name, path }),
  /** Opens a non-Markdown file in the OS default app. */
  docsOpenExternal: (path: string) => invoke<void>("docs_open_external", { path }),
  /** Shows a file or folder in Explorer, selected in its parent. */
  docsReveal: (path: string) => invoke<void>("docs_reveal", { path }),
  /** Creates a diagram note inside a registered document root (T-1001).
   * `dir` must be a folder inside a registered root. Returns the new path. */
  docsCreateDiagram: (dir: string, kind: string, title: string) =>
    invoke<string>("docs_create_diagram", { dir, kind, title }),
  /** Renames a diagram note inside a registered root (T-1002): frontmatter
   * `title` and file name move together. Returns the new path. Rejects when
   * the file is not a diagram note. */
  docsRenameDiagram: (path: string, title: string) =>
    invoke<string>("docs_rename_diagram", { path, title }),
  /** Moves a diagram note inside a registered root to the OS trash (T-1002).
   * Rejects when the file is not a diagram note. */
  docsDeleteDiagram: (path: string) => invoke<void>("docs_delete_diagram", { path }),
  /** One directory only — the tree calls again when a folder is opened. */
  docsListDir: (path: string) => invoke<DocsEntry[]>("docs_list_dir", { path }),
  docsReadFile: (path: string) => invoke<string>("docs_read_file", { path }),
  /** An embedded image as a `data:` URI. */
  docsReadAsset: (path: string) => invoke<string>("docs_read_asset", { path }),
  /** Resolves a pasted absolute path against this machine's roots (T-0362):
   * a direct hit when the mounts agree, longest-tail candidates when they
   * do not. Rejects when nothing under the registered roots matches. */
  docsResolveOpenPath: (pasted: string, currentRoot?: string) =>
    invoke<DocsOpenPathResolution>("docs_resolve_open_path", { pasted, currentRoot }),
  /** Resolves an Obsidian `[[wikilink]]` written in `from` (T-0726): one path
   * when the name is unique, candidates when it is ambiguous, neither when it
   * is missing. `target` is the link's file part — alias and heading split off. */
  docsResolveWikiLink: (from: string, target: string) =>
    invoke<WikiLinkResolution>("docs_resolve_wikilink", { from, target }),
  /** The PlantUML server diagrams are rendered on; "" when rendering is off. */
  docsShortcuts: () => invoke<DocsShortcut[]>("docs_shortcuts"),
  /** Replaces the whole list: adding, removing and reordering are one write. */
  setDocsShortcuts: (shortcuts: DocsShortcut[]) =>
    invoke<void>("set_docs_shortcuts", { shortcuts }),
  docsListPane: () => invoke<boolean>("docs_list_pane"),
  setDocsListPane: (enabled: boolean) => invoke<void>("set_docs_list_pane", { enabled }),
  /** Whether the Docs tab loads images from `https:` URLs (T-0329). */
  docsAllowRemoteImages: () => invoke<boolean>("docs_allow_remote_images"),
  setDocsAllowRemoteImages: (enabled: boolean) =>
    invoke<void>("set_docs_allow_remote_images", { enabled }),
  docsShowHidden: () => invoke<boolean>("docs_show_hidden"),
  setDocsShowHidden: (enabled: boolean) => invoke<void>("set_docs_show_hidden", { enabled }),
  docsPlantumlServer: () => invoke<string>("docs_plantuml_server"),
  setDocsPlantumlServer: (server: string) =>
    invoke<void>("set_docs_plantuml_server", { server }),
  /** A PlantUML diagram as SVG markup. Fails, sending nothing, when no server is set. */
  docsRenderPlantuml: (source: string) => invoke<string>("docs_render_plantuml", { source }),
  /** Pops a document or one figure out into a window of its own (T-0279). */
  openDocsViewer: (payload: DocsViewerPayload) => invoke<void>("open_docs_viewer", { payload }),
  /** Read by a viewer window: what it was opened to show. */
  docsViewerPayload: () => invoke<DocsViewerPayload | null>("docs_viewer_payload"),

  // ---- diagrams (any note under projects/ whose `type` is a diagram kind) ----
  /** `project` narrows to one project slug; pass "" for every project. */
  readDiagram: (path: string) => invoke<DiagramDoc>("read_diagram", { path }),
  /** Returns the new mtime. Rejects when the file changed on disk since it was
   * read; pass `expectedMtime: 0` to write unconditionally. */
  writeDiagram: (path: string, content: string, expectedMtime: number) =>
    invoke<number>("write_diagram", { path, content, expectedMtime }),
  /** Renames a note: frontmatter `title` and file name move together. Returns
   * the note at its new path, which the caller must reselect. */
  exportDiagramFile: (
    outPath: string,
    content: string,
    guard: { vaultPath: string; project: string },
  ) =>
    invoke<void>("export_diagram_file", {
      outPath,
      content,
      vaultPath: guard.vaultPath,
      project: guard.project,
    }),
  /** `base64Data` is the payload of a `data:image/png;base64,...` URL. */
  exportDiagramPng: (
    outPath: string,
    base64Data: string,
    guard: { vaultPath: string; project: string },
  ) =>
    invoke<void>("export_diagram_png", {
      outPath,
      base64Data,
      vaultPath: guard.vaultPath,
      project: guard.project,
    }),
  // ---- mindmap notes (projects/<slug>/mindmaps/*.md) ----
  /** `project` narrows to one project slug; pass "" for every project. */
  listMindmaps: (vaultPath: string, project = "") =>
    invoke<MindmapFile[]>("list_mindmaps", { vaultPath, project }),
  readMindmap: (path: string) => invoke<MindmapDoc>("read_mindmap", { path }),
  /** Returns the new mtime. Rejects when the file changed on disk since it was
   * read; pass `expectedMtime: 0` to write unconditionally. */
  writeMindmap: (path: string, content: string, expectedMtime: number) =>
    invoke<number>("write_mindmap", { path, content, expectedMtime }),
  createMindmap: (vaultPath: string, project: string, title: string) =>
    invoke<MindmapFile>("create_mindmap", { vaultPath, project, title }),
  /** Renames a note: frontmatter `title` and file name move together. Returns
   * the note at its new path, which the caller must reselect. */
  renameMindmap: (vaultPath: string, path: string, title: string) =>
    invoke<MindmapFile>("rename_mindmap", { vaultPath, path, title }),
  /** Moves the note into `_ai/state/mindmap-trash/` rather than deleting it,
   * and returns where it went. */
  deleteMindmap: (vaultPath: string, path: string) =>
    invoke<string>("delete_mindmap", { vaultPath, path }),
  /** `guard` names the vault and project the destination was resolved into
   * — a mindmap export always lands under the vault's own default
   * `attachments/` folder, so this is refused rather than inventing the
   * project folder if the slug does not resolve (T-0379). */
  exportMindmapFile: (
    outPath: string,
    content: string,
    guard: { vaultPath: string; project: string },
  ) =>
    invoke<void>("export_mindmap_file", {
      outPath,
      content,
      vaultPath: guard.vaultPath,
      project: guard.project,
    }),
  /** `base64Data` is the payload of a `data:image/png;base64,...` URL. See
   * `exportMindmapFile` for `guard` (T-0379). */
  exportMindmapPng: (
    outPath: string,
    base64Data: string,
    guard: { vaultPath: string; project: string },
  ) =>
    invoke<void>("export_mindmap_png", {
      outPath,
      base64Data,
      vaultPath: guard.vaultPath,
      project: guard.project,
    }),
  // ---- music player (vault-backed) ----
  revealMainWindow: () => invoke<void>("reveal_main_window"),

  // ---- voice input: transcript history ----
};

export function timeAgo(unixSecs: number): string {
  const secs = Math.max(0, Math.floor(Date.now() / 1000) - unixSecs);
  if (secs < 60) return "just now";
  if (secs < 3600) return `${Math.floor(secs / 60)}m ago`;
  if (secs < 86400) return `${Math.floor(secs / 3600)}h ago`;
  return `${Math.floor(secs / 86400)}d ago`;
}

export function nowUnix(): number {
  return Math.floor(Date.now() / 1000);
}
