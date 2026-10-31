import { useCallback, useEffect, useRef, useState } from "react";
import { BookOpen, Minus, Plus, RotateCcw, Settings as SettingsIcon } from "lucide-react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { DocsView } from "@/components/docs/docs-view";
import { ErrorBoundary } from "@/components/error-boundary";
import { SettingsDialog } from "@/components/settings-dialog";
import { Button } from "@/components/ui/button";
import { Hint } from "@/components/ui/hint";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { TooltipProvider } from "@/components/ui/tooltip";
import { api } from "@/lib/api";
import {
  APP_ZOOM,
  APP_ZOOM_KEY,
  isTauri,
  matchZoomKey,
  normalizeAppZoom,
  parseAppZoom,
} from "@/lib/app-zoom";
import { applyLocale, setWindowTitle, useLocale, useT } from "@/lib/i18n";
import { cn } from "@/lib/utils";
import type { Settings } from "@/types";

/**
 * The viewer's shell (T-1001): one tab, docs. There is no tab strip, no
 * vault setup and no plugin/template/memory banners — the viewer opens a
 * shared folder, not a vault.
 */
function ZoomControl() {
  const t = useT();
  const [zoom, setZoomState] = useState(() => {
    try {
      return parseAppZoom(localStorage.getItem(APP_ZOOM_KEY));
    } catch {
      return APP_ZOOM.initial;
    }
  });
  const zoomRef = useRef(zoom);
  zoomRef.current = zoom;

  const apply = useCallback((next: number) => {
    const value = normalizeAppZoom(next);
    setZoomState(value);
    try {
      localStorage.setItem(APP_ZOOM_KEY, String(value));
    } catch {
      // Persistence is a nicety; never break zooming over storage.
    }
    if (isTauri()) void getCurrentWebview().setZoom(value).catch(console.error);
  }, []);

  useEffect(() => {
    if (isTauri()) {
      getCurrentWebview()
        .setZoom(zoomRef.current)
        .catch(console.error)
        .finally(() => void api.revealMainWindow().catch(console.error));
    }
    const onKey = (e: KeyboardEvent) => {
      if (!(e.ctrlKey || e.metaKey) || e.altKey) return;
      const action = matchZoomKey(e);
      if (!action) return;
      e.preventDefault();
      if (action === "in") apply(zoomRef.current + APP_ZOOM.step);
      else if (action === "out") apply(zoomRef.current - APP_ZOOM.step);
      else apply(APP_ZOOM.initial);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [apply]);

  const percent = Math.round(zoom * 100);
  return (
    <Popover>
      <PopoverTrigger asChild>
        <button
          type="button"
          aria-label={t("nav.zoomAria", { percent })}
          className="hidden rounded px-1.5 py-1 text-[11px] tabular-nums text-muted-foreground hover:text-foreground md:inline"
        >
          {percent}%
        </button>
      </PopoverTrigger>
      <PopoverContent align="end" className="w-56 p-3">
        <div className="flex items-center gap-1">
          <Button
            size="icon-xs"
            variant="ghost"
            aria-label={t("nav.zoomOut")}
            disabled={zoom <= APP_ZOOM.min}
            onClick={() => apply(zoom - APP_ZOOM.step)}
          >
            <Minus />
          </Button>
          <input
            type="range"
            min={APP_ZOOM.min * 100}
            max={APP_ZOOM.max * 100}
            step={APP_ZOOM.step * 100}
            value={percent}
            onChange={(e) => apply(Number(e.target.value) / 100)}
            className="min-w-0 flex-1 accent-primary"
            aria-label={t("nav.zoomSliderAria")}
          />
          <Button
            size="icon-xs"
            variant="ghost"
            aria-label={t("nav.zoomIn")}
            disabled={zoom >= APP_ZOOM.max}
            onClick={() => apply(zoom + APP_ZOOM.step)}
          >
            <Plus />
          </Button>
        </div>
        <div className="mt-1 flex items-center justify-between">
          <span className="text-xs tabular-nums text-muted-foreground">{percent}%</span>
          <Button size="xs" variant="ghost" onClick={() => apply(APP_ZOOM.initial)}>
            <RotateCcw />
            {t("nav.zoomReset")}
          </Button>
        </div>
        <p className="mt-1 text-[11px] text-muted-foreground">Ctrl+= / Ctrl+- / Ctrl+0</p>
      </PopoverContent>
    </Popover>
  );
}

export default function App() {
  const t = useT();
  const [showSettings, setShowSettings] = useState(false);
  const [settings, setSettings] = useState<Settings | null>(null);
  const uiLocale = settings?.ui_locale;
  useEffect(() => {
    if (uiLocale !== undefined) applyLocale(uiLocale);
  }, [uiLocale]);
  const locale = useLocale();
  useEffect(() => {
    setWindowTitle("app.windowTitle");
  }, [locale]);
  const [version, setVersion] = useState("");
  // Bumped after settings save; views reload their config when it changes.
  const [, setConfigVersion] = useState(0);

  useEffect(() => {
    void (async () => {
      const cfg = await api.getConfig();
      setSettings(cfg.settings);
      setVersion(await api.appVersion());
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const saveSettings = useCallback(async (next: Settings) => {
    const cfg = await api.getConfig();
    const saved = await api.saveConfig({ ...cfg, settings: next });
    setSettings(saved.settings);
    setConfigVersion((v) => v + 1);
  }, []);

  const openSettings = useCallback(async () => {
    try {
      const cfg = await api.getConfig();
      setSettings(cfg.settings);
    } catch {
      // Fall back to the copy already held rather than not opening at all.
    }
    setShowSettings(true);
  }, []);

  return (
    <TooltipProvider delayDuration={300}>
      <div className="flex h-full flex-col overflow-hidden">
        <nav className="flex items-center gap-1 border-b bg-muted/30 px-3 py-1.5">
          <span className="flex items-center gap-1.5 text-xs font-medium text-foreground">
            <BookOpen className="size-3.5 text-muted-foreground" />
            {t("nav.docs")}
          </span>
          <div className="flex shrink-0 items-center gap-1">
            <ZoomControl />
            <span
              className={cn(
                "hidden text-[11px] text-muted-foreground md:inline",
                !version && "invisible",
              )}
            >
              v{version || "…"}
            </span>
            <Hint label={t("nav.settingsAria")}>
              <Button
                size="icon"
                variant="ghost"
                className="size-7"
                aria-label={t("nav.settingsAria")}
                onClick={() => void openSettings()}
              >
                <SettingsIcon className="size-4" />
              </Button>
            </Hint>
          </div>
        </nav>
        <div className="min-h-0 flex-1">
          <div className="h-full">
            <ErrorBoundary label="docs">
              <DocsView />
            </ErrorBoundary>
          </div>
        </div>
        {settings && (
          <SettingsDialog
            open={showSettings}
            settings={settings}
            onClose={() => setShowSettings(false)}
            onSave={saveSettings}
          />
        )}
      </div>
    </TooltipProvider>
  );
}
