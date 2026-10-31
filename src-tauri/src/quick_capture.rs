//! Quick capture: a global hotkey (default Ctrl+Alt+N) opens a small
//! always-on-top window next to the cursor that turns the clipboard into an
//! inbox task
//! (title + description form; the frontend calls the existing `create_task`).
//!
//! The window is created hidden at startup and reused — never built from the
//! hotkey handler: on Windows that handler runs synchronously inside WndProc
//! (the main event loop), and building a WebView2 window there self-blocks
//! for ~10s because the build waits on the very message pump it is holding
//! (measured in the kakisute app; see its src-tauri/src/windows.rs).

use tauri::{AppHandle, Manager, WebviewWindow};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::models::WindowRect;
use crate::storage;

pub const WINDOW_LABEL: &str = "quick-capture";
/// Tried in order after the configured shortcut when registration fails
/// (e.g. another app already holds the key).
const FALLBACK_SHORTCUTS: &[&str] = &["Ctrl+Shift+N"];

/// The shortcut actually registered (may be a fallback, so the configured
/// string can't be used to identify it in the handler).
#[derive(Default)]
pub struct QuickCaptureState(pub std::sync::Mutex<Option<Shortcut>>);

/// Ordered registration candidates: the preferred key first, then fallbacks,
/// deduplicated. Pure so the ordering is unit-testable.
fn candidates<'a>(preferred: &'a str, fallbacks: &[&'a str]) -> Vec<&'a str> {
    let mut list = vec![preferred];
    for f in fallbacks {
        if !list.contains(f) {
            list.push(f);
        }
    }
    list
}

fn window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(WINDOW_LABEL)
}

/// Remember the current window rect and hide. Persisting here (not on every
/// Moved/Resized) keeps config writes off the hot path.
pub fn hide(app: &AppHandle) {
    let Some(win) = window(app) else { return };
    if let Some(rect) = current_rect(&win) {
        let mut cfg = storage::load();
        cfg.settings.quick_capture_rect = Some(rect);
        if let Err(e) = storage::save(&cfg) {
            crate::diag!("quick-capture: failed to persist window rect: {e}");
        }
    }
    let _ = win.hide();
}

fn current_rect(win: &WebviewWindow) -> Option<WindowRect> {
    let (pos, size, scale) = (
        win.outer_position().ok()?,
        win.inner_size().ok()?,
        win.scale_factor().ok()?,
    );
    // A minimized window reports coordinates like (-32000, -32000); don't
    // remember those or the next open would be off-screen.
    if pos.x <= -30000 || pos.y <= -30000 || size.width == 0 {
        return None;
    }
    let pos = pos.to_logical::<f64>(scale);
    let size = size.to_logical::<f64>(scale);
    Some(WindowRect {
        x: pos.x,
        y: pos.y,
        width: size.width,
        height: size.height,
    })
}

/// (Re)register the global hotkey from the current settings. Tries the
/// configured key first, then fallbacks, and reports what was actually
/// registered. Safe to call again after a settings change.
pub fn apply_shortcut(app: &AppHandle) {
    let settings = storage::load().settings;
    let state = app.state::<QuickCaptureState>();

    // Drop the previous registration (if any) before re-registering.
    if let Some(prev) = state.0.lock().unwrap().take() {
        let _ = app.global_shortcut().unregister(prev);
    }
    if !settings.quick_capture_enabled {
        return;
    }

    let preferred = settings.quick_capture_shortcut.as_str();
    for candidate in candidates(preferred, FALLBACK_SHORTCUTS) {
        let shortcut: Shortcut = match candidate.parse() {
            Ok(s) => s,
            Err(e) => {
                crate::diag!("quick-capture: invalid shortcut {candidate}: {e}");
                continue;
            }
        };
        match app.global_shortcut().register(shortcut) {
            Ok(()) => {
                if candidate == preferred {
                    crate::diag!("quick-capture: hotkey {candidate} registered");
                } else {
                    crate::diag!(
                        "quick-capture: {preferred} is taken, registered {candidate} instead"
                    );
                }
                *state.0.lock().unwrap() = Some(shortcut);
                return;
            }
            Err(e) => crate::diag!("quick-capture: failed to register {candidate}: {e}"),
        }
    }
    crate::diag!("quick-capture: could not register any hotkey ({preferred})");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidates_prefers_configured_key_then_fallbacks() {
        assert_eq!(
            candidates("Ctrl+Alt+N", &["Ctrl+Shift+N"]),
            vec!["Ctrl+Alt+N", "Ctrl+Shift+N"]
        );
    }

    #[test]
    fn candidates_dedupes_when_configured_equals_fallback() {
        assert_eq!(
            candidates("Ctrl+Shift+N", &["Ctrl+Shift+N"]),
            vec!["Ctrl+Shift+N"]
        );
    }

    #[test]
    fn settings_defaults() {
        let s = crate::models::Settings::default();
        assert!(s.quick_capture_enabled);
        assert_eq!(s.quick_capture_shortcut, "Ctrl+Alt+N");
        assert!(s.quick_capture_rect.is_none());
    }

    #[test]
    fn settings_deserialize_missing_fields_uses_defaults() {
        // Older config.json files predate the quick-capture keys; they must
        // deserialize with the defaults (config compatibility contract).
        let s: crate::models::Settings = serde_json::from_str("{}").unwrap();
        assert!(s.quick_capture_enabled);
        assert_eq!(s.quick_capture_shortcut, "Ctrl+Alt+N");
        assert!(s.quick_capture_rect.is_none());
    }
}
