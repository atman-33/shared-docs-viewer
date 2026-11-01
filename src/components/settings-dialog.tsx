import { useEffect, useState } from "react";
import { Check, Loader2 } from "lucide-react";
import { api } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { DiagnosticLogPanel } from "@/components/diagnostic-log-panel";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { LOCALES, useT } from "@/lib/i18n";
import {
  isThemePreference,
  readThemePreference,
  setThemePreference,
  THEME_PREFERENCES,
  type ThemePreference,
} from "@/lib/theme";
import type { Settings, UpdateInfo } from "@/types";

const TIDY_DEFAULTS: Settings["tidy"] = {
  enabled: false,
  assignee: "claude-code",
  model: "",
  anchor: null,
  interval_hours: 24,
  stale_days: 7,
  exclude_dirs: ["_wip"],
  last_run: null,
  last_session_id: null,
};

const LANGUAGES: { id: string; label: string }[] = [
  { id: "en", label: "English" },
  { id: "ja", label: "日本語" },
];

const DEFAULTS: Settings = {
  // Managed from the Repos tab itself, not from this dialog (T-0304).
  vscode_cmd: "code",
  terminal_cmd: "wt -d {path}",
  agent_cmd: "wt -d {path} pwsh -NoExit -Command claude",
  opencode_cmd: "wt -d {path} pwsh -NoExit -Command opencode",
  use_herdr: true,
  herdr_cmd: "herdr",
  autostart: false,
  check_updates: true,
  check_template_updates: true,
  auto_apply_template_updates: true,
  check_memory_setup: true,
  memory_claude_code: true,
  memory_opencode: true,
  secretary_enabled: false,
  ink_enabled: true,
  ink_dir: "",
  vault_path: null,
  worktree_root: "C:/repos/.worktrees",
  // Managed from the Tasks tab itself, not from this dialog (T-0300).
  terminal_embed: false,
  quick_capture_enabled: true,
  quick_capture_shortcut: "Ctrl+Alt+N",
  quick_capture_rect: null,
  ink_preview_rect: null,
  task_editor_rect: null,
  task_editor_maximized: false,
  // Managed from the Voice tab itself, not from this dialog (T-0277).
  voice_enabled: true,
  voice_hotkey: "Ctrl+Shift+Space",
  voice_model: "small",
  voice_language: "auto",
  voice_indicator_placement: "caret",
  voice_indicator_position: null,
  voice_system_audio: false,
  clips_enabled: true,
  clips_gesture: "ctrl-double",
  clips_rect: null,
  language: "en",
  ui_locale: "en",
  // Managed from the tab bar itself (right-click / drag), not from this dialog (T-0684).
  tab_order: [],
  hidden_tabs: [],
  // Managed from the Diagrams tab itself (T-0686).
  diagram_hidden_panels: [],
  diagram_collapsed_kinds: [],
  diagram_list_sort: "updated",
  response_language_inject: true,
  custom_prompt: "",
  prompt_copy_multiline: true,
  claude_desktop_mode: "code",
  // Managed from the Inbox tab itself, not from this dialog (T-0300).
  tidy: TIDY_DEFAULTS,
  // Managed from the Diagrams tab itself, not from this dialog (T-0289, T-0685).
  diagram_assignee: "claude-code",
  diagram_model: "",
  diagram_confirm: false,
  schedule_export_dir: "",
  // Managed from the Voice tab's meeting panel, not from this dialog (T-0318).
  meeting_struct_interval_secs: 120,
  meeting_struct_assignee: "claude-code",
  meeting_struct_model: "",
  // Managed from the Voice tab's meeting panel, not from this dialog (T-0338).
  voice_meetings_dir: "voice/meetings",
  recurring: [],
  // Managed from the Docs tab itself, not from this dialog (T-0259).
  docs_roots: [],
  notices_read: [],
  docs_plantuml_server: "",
  docs_shortcuts: [],
  docs_list_pane: false,
  docs_allow_remote_images: false,
  docs_show_hidden: false,
};

interface Props {
  open: boolean;
  settings: Settings;
  onClose: () => void;
  /** Persists the settings; rejects (with a message) on failure so the
   * dialog can report it and stay open instead of closing on a save that
   * silently didn't happen (T-0064). */
  onSave: (settings: Settings) => Promise<void>;
}

export function SettingsDialog({ open, settings, onClose, onSave }: Props) {
  const t = useT();
  const [draft, setDraft] = useState<Settings>(settings);
  const [version, setVersion] = useState("");
  const [update, setUpdate] = useState<UpdateInfo | null>(null);
  const [phase, setPhase] = useState<
    "idle" | "checking" | "uptodate" | "available" | "downloading" | "ready" | "failed"
  >("idle");
  const [error, setError] = useState("");
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState("");
  const [theme, setTheme] = useState<ThemePreference>(readThemePreference);

  useEffect(() => {
    if (open) {
      setDraft(settings);
      setUpdate(null);
      setPhase("idle");
      setError("");
      setSaveError("");
      void api.appVersion().then(setVersion);
    }
  }, [open, settings]);

  const check = async () => {
    setPhase("checking");
    setError("");
    const info = await api.checkUpdate();
    if (info) {
      setUpdate(info);
      setPhase("available");
    } else {
      setUpdate(null);
      setPhase("uptodate");
    }
  };

  const save = async () => {
    setSaveError("");
    setSaving(true);
    try {
      await onSave(draft);
      onClose();
    } catch (e) {
      setSaveError(String(e));
    } finally {
      setSaving(false);
    }
  };

  const install = async () => {
    if (!update) return;
    setPhase("downloading");
    try {
      await api.applyUpdate(update.url);
      setPhase("ready");
    } catch (e) {
      setError(String(e));
      setPhase("failed");
    }
  };

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent draggable className="flex max-h-[90vh] flex-col gap-4 sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>{t("settings.title")}</DialogTitle>
          <DialogDescription>{t("settings.description")}</DialogDescription>
        </DialogHeader>
        {/* The viewer opens a shared folder, not a vault (T-1001): there is no
            vault picker here. Document folders are registered on the docs
            tab itself. */}
        <Tabs defaultValue="general" className="flex flex-col gap-3">
          <TabsList>
            <TabsTrigger value="general">{t("settings.tab.general")}</TabsTrigger>
            <TabsTrigger value="agents">{t("settings.tab.agents")}</TabsTrigger>
          </TabsList>
          {/* Fixed-height scroll area so the tab bar stays put when switching
              tabs, regardless of how much content each tab holds. The bottom
              padding is part of the scrollable content, so the last section
              does not sit flush against the edge at the end of the scroll. */}
          <div className="-mx-6 h-[min(65vh,520px)] overflow-y-auto px-6 pb-4">
            <TabsContent value="general" className="mt-0 space-y-3">
              <div className="space-y-2 rounded-md border p-3">
                <p className="text-sm font-medium">{t("settings.general.theme.title")}</p>
                <Select
                  value={theme}
                  onValueChange={(v) => {
                    if (!isThemePreference(v)) return;
                    setTheme(v);
                    setThemePreference(v);
                  }}
                >
                  <SelectTrigger size="sm">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {THEME_PREFERENCES.map((p) => (
                      <SelectItem key={p} value={p}>
                        {t(`settings.general.theme.${p}`)}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  {t("settings.general.theme.description")}
                </p>
              </div>
              {/* Every group on this tab is a titled bordered section, so no
                  checkbox sits loose next to a framed one. */}
              <div className="space-y-2 rounded-md border p-3">
                <p className="flex items-center gap-2 text-sm font-medium">
                  {t("settings.general.language.title")}
                </p>
                <Select
                  value={draft.ui_locale}
                  onValueChange={(v) => setDraft({ ...draft, ui_locale: v })}
                >
                  <SelectTrigger size="sm">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {LOCALES.map((l) => (
                      <SelectItem key={l.id} value={l.id}>
                        {l.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  {t("settings.general.language.description")}
                </p>
              </div>
              <div className="space-y-2 rounded-md border p-3">
                <p className="text-sm font-medium">{t("settings.general.startup.title")}</p>
                <label className="flex items-center gap-2 text-sm">
                  <Checkbox
                    checked={draft.autostart}
                    onCheckedChange={(v) => setDraft({ ...draft, autostart: v === true })}
                  />
                  {t("settings.general.startup.autostart")}
                </label>
                <p className="pl-6 text-xs text-muted-foreground">
                  {t("settings.general.startup.autostartDescription")}
                </p>
              </div>
              <div className="space-y-2 rounded-md border p-3">
                <p className="text-sm font-medium">{t("settings.general.startupChecks.title")}</p>
                <label className="flex items-center gap-2 text-sm">
                  <Checkbox
                    checked={draft.check_updates}
                    onCheckedChange={(v) => setDraft({ ...draft, check_updates: v === true })}
                  />
                  {t("settings.general.startupChecks.checkUpdates")}
                </label>
              </div>
              {/* The viewer runs no helpers (T-1001): no quick-capture, no ink,
                  no listener to restart. Diagnostics stay for the log. */}
              <DiagnosticLogPanel />
              <div className="space-y-2 rounded-md border p-3">
                <div className="flex items-center justify-between gap-3">
                  <div>
                    <p className="text-sm font-medium">{t("settings.general.appUpdate.title")}</p>
                    <p className="text-xs text-muted-foreground">
                      {t("settings.general.appUpdate.currentVersion", { version })}
                    </p>
                  </div>
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    onClick={check}
                    disabled={phase === "checking" || phase === "downloading"}
                  >
                    {phase === "checking" && <Loader2 className="mr-1.5 size-3.5 animate-spin" />}
                    {phase === "checking"
                      ? t("settings.general.appUpdate.checking")
                      : t("settings.general.appUpdate.check")}
                  </Button>
                </div>
                {phase === "ready" && (
                  <div className="flex items-center justify-between gap-3 rounded-md bg-muted p-2">
                    <span className="flex items-center gap-1.5 text-xs">
                      <Check className="size-3.5 text-green-500" />
                      {t("settings.general.appUpdate.installed")}
                    </span>
                    <Button
                      type="button"
                      size="sm"
                      variant="secondary"
                      onClick={() => api.restartApp()}
                    >
                      {t("banner.update.restartNow")}
                    </Button>
                  </div>
                )}
                {update && phase !== "ready" && (
                  <div className="flex items-center justify-between gap-3 rounded-md bg-muted p-2">
                    <span className="text-xs">
                      {t("settings.general.appUpdate.newVersion", { tag: update.tag })}
                    </span>
                    <Button
                      type="button"
                      size="sm"
                      onClick={install}
                      disabled={phase === "downloading"}
                    >
                      {phase === "downloading" && (
                        <Loader2 className="mr-1.5 size-3.5 animate-spin" />
                      )}
                      {phase === "downloading"
                        ? t("settings.general.appUpdate.downloading")
                        : t("settings.general.appUpdate.downloadInstall")}
                    </Button>
                  </div>
                )}
                {phase === "uptodate" && (
                  <p className="text-xs text-muted-foreground">
                    {t("settings.general.appUpdate.upToDate")}
                  </p>
                )}
                {phase === "failed" && (
                  <p className="text-xs text-destructive">
                    {t("settings.general.appUpdate.failed", { error })}
                  </p>
                )}
              </div>
            </TabsContent>
            {/* The viewer runs no agent itself (T-1001): no command templates,
                no worktree root, no desktop send-off. What stays shapes the
                copied prompt's language. */}
            <TabsContent value="agents" className="mt-0 space-y-3">
              <div className="space-y-1.5 pt-1">
                <label className="flex items-center gap-2 text-xs font-medium text-muted-foreground">
                  {t("settings.agents.language.label")}
                </label>
                <p className="text-[10px] leading-tight text-muted-foreground/70">
                  {t("settings.agents.language.description")}
                </p>
                <Select
                  value={draft.language}
                  onValueChange={(v) => setDraft({ ...draft, language: v })}
                >
                  <SelectTrigger size="sm">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {LANGUAGES.map((l) => (
                      <SelectItem key={l.id} value={l.id}>
                        {l.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </TabsContent>
          </div>
        </Tabs>
        {saveError && (
          <p className="text-xs text-destructive">
            {t("settings.footer.saveFailed", { error: saveError })}
          </p>
        )}
        <DialogFooter>
          {/* Recurring rules are content the user authored (edited from the
              Tasks tab, not here), not a knob with a sensible default — a reset
              of the command templates must not delete them. */}
          <Button variant="ghost" onClick={() => setDraft({ ...DEFAULTS, recurring: draft.recurring })}>
            {t("settings.footer.resetToDefaults")}
          </Button>
          <Button onClick={() => void save()} disabled={saving}>
            {saving && <Loader2 className="mr-1.5 size-3.5 animate-spin" />}
            {saving ? t("settings.footer.saving") : t("common.save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
