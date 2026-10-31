//! Clips popup window lifecycle.
//!
//! Like `quick_capture.rs`, the window is created hidden at startup and only
//! ever shown/hidden from here — never built from the gesture handler, which
//! runs on the raw-input thread and hops to the main thread (building a
//! WebView2 window from inside the main event loop self-blocks for ~10s).
//!
//! Unlike the ink overlay and the voice indicator, this window *takes* focus:
//! it has a filter box and arrow-key navigation. The window that had focus
//! when the gesture fired is therefore remembered here, and focus is handed
//! back to it before the snippet is pasted (see `paste_clip`).

use tauri::{AppHandle, Emitter, Manager};
use tauri::{Runtime, WebviewWindow};

use crate::models::WindowRect;
use crate::storage;

pub const WINDOW_LABEL: &str = "clips-popup";

fn window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(WINDOW_LABEL)
}

/// Show the popup next to the mouse cursor, remembering which window had
/// focus so the paste can go back to it.
pub fn show(app: &AppHandle) {
    let Some(win) = window(app) else { return };
    if win.is_visible().unwrap_or(false) {
        return;
    }
    super::remember_paste_target(app);
    crate::window_place::place_at_cursor(app, &win);
    let _ = win.show();
    let _ = win.set_focus();
    let _ = app.emit_to(WINDOW_LABEL, "clips://activate", ());
}

/// Remember the current window rect and hide. Persisting here (not on every
/// Moved/Resized) keeps config writes off the hot path.
pub fn hide(app: &AppHandle) {
    let Some(win) = window(app) else { return };
    if !win.is_visible().unwrap_or(false) {
        return;
    }
    if let Some(rect) = current_rect(&win) {
        let mut cfg = storage::load();
        cfg.settings.clips_rect = Some(rect);
        if let Err(e) = storage::save(&cfg) {
            crate::diag!("clips: failed to persist window rect: {e}");
        }
    }
    let _ = win.hide();
}

fn current_rect<R: Runtime>(win: &WebviewWindow<R>) -> Option<WindowRect> {
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
