mod actions;
mod b64;
mod commands;
mod diag;
mod diagram;
mod diagram_edit;
mod docs;
mod docs_viewer;
mod mindmap;
mod models;
mod plantuml;
mod schedule;
mod storage;
mod tasks;
mod update;
mod vault_note;
mod vault_project;
mod window_place;

use tauri::Manager;

/// One line saying what the app started with. Deliberately a snapshot of the
/// *settings*, not of the machine: which features are on, and which vault is
/// configured, are the first two questions any bug report raises.
fn log_startup_settings(cfg: &models::Config) {
    let s = &cfg.settings;
    crate::diag!(
        "config: vault={} autostart={} ink={} clips={} quick-capture={} voice={} (model={}, indicator={})",
        s.vault_path.as_deref().unwrap_or("(none)"),
        s.autostart,
        s.ink_enabled,
        s.clips_enabled,
        s.quick_capture_enabled,
        s.voice_enabled,
        s.voice_model,
        s.voice_indicator_placement
    );
}

/// Argument the autostart registration appends to the exe path, so a sign-in
/// launch can be told apart from the user opening the app themselves.
pub const AUTOSTART_ARG: &str = "--autostart";

/// Brings the registry entry in line with the setting.
///
/// Enabling always rewrites the entry rather than checking it first: the
/// value is the running exe's path, so re-registering is how a moved or
/// self-updated install stops pointing Windows at an exe that is gone.
/// Disabling only acts when there is something registered, so the common
/// case — the setting has never been switched on — touches nothing and logs
/// nothing on every start.
///
/// A debug build deliberately does nothing: its exe lives in
/// `target/debug/`, and registering that path would outlive the build
/// directory.
pub fn apply_autostart(app: &tauri::AppHandle, enabled: bool) {
    if cfg!(debug_assertions) {
        crate::diag!("autostart: skipped in a debug build (enabled={enabled})");
        return;
    }
    use tauri_plugin_autostart::ManagerExt;
    let manager = app.autolaunch();
    let result = if enabled {
        manager.enable()
    } else if manager.is_enabled().unwrap_or(false) {
        manager.disable()
    } else {
        return;
    };
    match result {
        Ok(()) => crate::diag!("autostart: start-with-Windows set to {enabled}"),
        Err(e) => crate::diag!("autostart: cannot set start-with-Windows to {enabled}: {e}"),
    }
}

/// Set once the main window has been revealed, so the frontend's call and the
/// fallback timer cannot both act (a second minimize would re-minimize a
/// window the user already restored).
static MAIN_REVEALED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// How long the main window may stay hidden waiting for the frontend (T-0491).
const REVEAL_FALLBACK: std::time::Duration = std::time::Duration::from_secs(5);

/// Shows the main window, which `tauri.conf.json` creates hidden so the
/// WebView never paints at 100% before the remembered zoom is applied.
///
/// When this process was launched by the sign-in registration it is shown
/// minimized instead of hidden: there is no tray icon, so a hidden main
/// window would only be reachable by launching the exe again (the
/// single-instance handler shows it). A taskbar button is the obvious way
/// back. Only the first call acts.
pub fn reveal_main_window(app: &tauri::AppHandle) {
    if MAIN_REVEALED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let Some(main_window) = app.get_webview_window("main") else {
        return;
    };
    if std::env::args().any(|arg| arg == AUTOSTART_ARG) {
        // Minimizing a hidden window shows it minimized.
        if let Err(e) = main_window.minimize() {
            crate::diag!("autostart: cannot minimize the main window: {e}");
        }
    } else if let Err(e) = main_window.show() {
        crate::diag!("cannot show the main window: {e}");
    }
}

pub fn run() {
    // First thing in the process: a release build has no console, so until
    // this runs every diagnostic (a panic included) is written nowhere.
    diag::init();
    update::cleanup_old();
    // As early as the migration allows: a report about a gesture that "does
    // nothing" is unreadable without knowing whether the feature was even
    // switched on. Recorded here rather than in `setup` so a startup that
    // dies half way still says what it was starting with.
    log_startup_settings(&storage::load());
    tauri::Builder::default()
        // Must be registered first (per tauri-plugin-single-instance docs).
        // Without this, every launch adds another process.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        // Start-with-Windows (T-0258). The extra argument is how the app
        // recognizes a sign-in launch and starts out of the way — see
        // `reveal_main_window`.
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![AUTOSTART_ARG]),
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        // No global-shortcut plugin (T-1001): the viewer registers no
        // hotkeys, so it can run beside other apps without stealing
        // their gestures.
        .manage(docs_viewer::DocsViewerState::default())
        .setup(|app| {
            // There is no tray icon, so a windowless process is unreachable —
            // exit the whole app when the main window closes. (The viewer
            // builds no hidden helper windows, so nothing else keeps Tauri
            // alive past this point.)
            if let Some(main_window) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        app_handle.exit(0);
                    }
                });
            }
            let cfg = storage::load();
            // Re-assert the start-with-Windows registration against the exe
            // that is actually running. The viewer ships as a portable exe and
            // self-updates in place, so the registered path goes stale the
            // moment the user moves the folder; rewriting it on every start
            // is cheaper than detecting that.
            apply_autostart(app.handle(), cfg.settings.autostart);
            // The window starts hidden; the frontend reveals it once the saved
            // zoom is applied. If it never does (load failure, IPC error), do
            // it here so the app cannot be left invisible.
            let fallback_handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(REVEAL_FALLBACK);
                reveal_main_window(&fallback_handle);
            });
            // The viewer builds no helper windows and runs no background
            // agents: docs viewer windows are created on demand.
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::open_explorer,
            commands::app_version,
            commands::reveal_main_window,
            commands::diagnostic_log,
            commands::diagnostic_log_info,
            commands::log_frontend_error,
            commands::check_update,
            commands::apply_update,
            commands::restart_app,
            commands::list_tasks,
            commands::update_task,
            commands::create_vault_project,
            commands::next_project_folder,
            commands::resolve_project_dir,
            commands::list_vault_projects,
            commands::set_vault_project_order,
            commands::list_schedules,
            commands::read_schedule,
            commands::write_schedule,
            commands::create_schedule,
            commands::rename_schedule,
            commands::delete_schedule,
            commands::export_schedule_html,
            commands::copy_diagram_edit_prompt,
            commands::docs_create_diagram,
            commands::docs_rename_diagram,
            commands::docs_delete_diagram,
            commands::docs_roots,
            commands::add_docs_root,
            commands::remove_docs_root,
            commands::update_docs_root,
            commands::docs_open_external,
            commands::docs_reveal,
            commands::docs_list_dir,
            commands::docs_read_file,
            commands::docs_read_asset,
            commands::docs_resolve_open_path,
            commands::docs_resolve_wikilink,
            commands::docs_shortcuts,
            commands::set_docs_shortcuts,
            commands::docs_list_pane,
            commands::set_docs_list_pane,
            commands::docs_allow_remote_images,
            commands::set_docs_allow_remote_images,
            commands::docs_show_hidden,
            commands::set_docs_show_hidden,
            commands::docs_plantuml_server,
            commands::set_docs_plantuml_server,
            commands::docs_render_plantuml,
            commands::open_docs_viewer,
            commands::docs_viewer_payload,
            commands::read_diagram,
            commands::write_diagram,
            commands::export_diagram_file,
            commands::export_diagram_png,
            commands::list_mindmaps,
            commands::read_mindmap,
            commands::write_mindmap,
            commands::create_mindmap,
            commands::rename_mindmap,
            commands::delete_mindmap,
            commands::export_mindmap_file,
            commands::export_mindmap_png,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
