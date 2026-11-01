/**
 * English UI strings — the reference dictionary (T-0409).
 *
 * Every key the UI uses is defined here; `ja.ts` is typed against these keys,
 * and anything it leaves out falls back to the English text. Keys are flat and
 * dotted by area (`nav.tasks`, `settings.general.title`) so a string is found
 * by searching for its key. Interpolate with `{name}` placeholders.
 *
 * The areas translated after T-0409 keep their strings in their own pair of
 * files (`<area>.en.ts` / `<area>.ja.ts`) and are spread in below, so one
 * area can grow without touching another's lines. New areas follow that
 * pattern; the keys still share one flat namespace.
 */
import { projectsEn } from "./projects.en";
import { miscEn } from "./misc.en";
import { scheduleEn } from "./schedule.en";
import { mindmapEn } from "./mindmap.en";
import { diagramEn } from "./diagram.en";
import { docsEn } from "./docs.en";

export const en = {
  "app.windowTitle": "shared-docs-viewer",

  "common.cancel": "Cancel",
  "common.save": "Save",
  "common.delete": "Delete",
  "common.close": "Close",

  "common.add": "Add",
  "common.remove": "Remove",
  "common.copy": "Copy",
  "common.copied": "Copied",

  "common.retry": "Retry",
  "common.refresh": "Refresh",

  "common.open": "Open",
  "common.today": "Today",
  "common.create": "Create",

  // Nav

  "nav.docs": "Docs",

  "nav.zoomAria": "App zoom {percent} percent, open zoom controls",
  "nav.zoomOut": "Zoom out",
  "nav.zoomIn": "Zoom in",
  "nav.zoomSliderAria": "App zoom",
  "nav.zoomReset": "Reset",
  "nav.settingsAria": "Settings",

  // Banners
  "banner.update.available": "New version {tag} is available (current v{version})",
  "banner.update.action": "Update & restart",
  "banner.update.downloading": "downloading {tag}…",
  "banner.update.installed": "✓ Update installed",
  "banner.update.restartNow": "Restart now",
  "banner.update.failed": "update failed: {error}",

  "task.status.todo": "Todo",
  "task.status.doing": "Doing",
  "task.status.review": "Review",
  "task.status.done": "Done",
  "task.status.inbox": "Inbox",

  "settings.title": "Settings",
  "settings.description": "How the app behaves, and the language agents reply in. Settings a single tab owns live in that tab.",

  "settings.tab.general": "General",
  "settings.tab.agents": "Agents",

  "settings.general.theme.title": "Theme",
  "settings.general.theme.light": "Light",
  "settings.general.theme.dark": "Dark",
  "settings.general.theme.system": "Follow system",
  "settings.general.theme.description":
    "Color theme of the app. Applies right away on this machine, in every window; it is not part of the vault.",
  "settings.general.language.title": "Display language",
  "settings.general.language.description":
    "Language the app's screens are shown in, including the schedule calendar and its HTML export. Screens not yet translated stay in English. Display only — nothing written to your notes is translated.",
  "settings.general.startup.title": "Startup",
  "settings.general.startup.autostart": "Start shared-docs-viewer when I sign in to Windows",
  "settings.general.startup.autostartDescription": "Starts minimized. Open it from the taskbar when you need it.",
  "settings.general.startupChecks.title": "Startup checks",
  "settings.general.startupChecks.checkUpdates": "Check for app updates",

  "settings.general.appUpdate.title": "App update",
  "settings.general.appUpdate.currentVersion": "Current version: v{version}",
  "settings.general.appUpdate.checking": "Checking…",
  "settings.general.appUpdate.check": "Check for updates",
  "settings.general.appUpdate.installed": "Update installed",
  "settings.general.appUpdate.newVersion": "New version {tag} is available",
  "settings.general.appUpdate.downloading": "Downloading…",
  "settings.general.appUpdate.downloadInstall": "Download & install",
  "settings.general.appUpdate.upToDate": "You are up to date.",
  "settings.general.appUpdate.failed": "Update failed: {error}",

  "settings.agents.language.label": "Language",
  "settings.agents.language.description": "Language AI agents reply to you in.",

  "settings.footer.saveFailed": "Save failed: {error}",
  "settings.footer.resetToDefaults": "Reset to defaults",
  "settings.footer.saving": "Saving…",

  // Areas kept in their own files (T-0423).
  ...projectsEn,
  ...miscEn,
  ...scheduleEn,
  ...mindmapEn,
  ...diagramEn,
  ...docsEn,
} as const;

export type MessageKey = keyof typeof en;
