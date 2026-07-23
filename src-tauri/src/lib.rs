mod content;

use content::{ColumnMeta, ContentState, MetaStore, PathInfo, PlatformInfo};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, RunEvent, State, WindowEvent,
};

static CLOSE_PROMPT_OPEN: AtomicBool = AtomicBool::new(false);
static REPLACE_RESTART_ARMED: AtomicBool = AtomicBool::new(false);

#[tauri::command]
fn get_path_info(app: AppHandle, state: State<'_, ContentState>) -> Result<PathInfo, String> {
    Ok(content::path_info(&app, &state))
}

#[tauri::command]
fn get_platform_info(app: AppHandle) -> Result<PlatformInfo, String> {
    Ok(content::platform_info(&app))
}

#[tauri::command]
fn get_content_root(app: AppHandle, state: State<'_, ContentState>) -> Result<String, String> {
    Ok(content::content_root(&app, &state)
        .to_string_lossy()
        .to_string())
}

#[tauri::command]
fn get_columns_path(app: AppHandle, state: State<'_, ContentState>) -> Result<String, String> {
    Ok(content::columns_root(&app, &state)
        .to_string_lossy()
        .to_string())
}

#[tauri::command]
fn get_default_columns_path(app: AppHandle) -> Result<String, String> {
    Ok(content::default_columns_path(&app)
        .to_string_lossy()
        .to_string())
}

#[tauri::command]
fn set_columns_path(
    app: AppHandle,
    state: State<'_, ContentState>,
    path: Option<String>,
) -> Result<PathInfo, String> {
    content::set_columns_path(&app, &state, path)
}

#[tauri::command]
fn set_session_columns_path(
    app: AppHandle,
    state: State<'_, ContentState>,
    path: Option<String>,
) -> Result<PathInfo, String> {
    content::set_session_columns_path(&app, &state, path)
}

#[tauri::command]
fn complete_path_onboarding(
    app: AppHandle,
    state: State<'_, ContentState>,
    path: Option<String>,
) -> Result<PathInfo, String> {
    content::complete_path_onboarding(&app, &state, path)
}

#[tauri::command]
fn list_columns(app: AppHandle, state: State<'_, ContentState>) -> Result<Vec<ColumnMeta>, String> {
    content::scan_columns(&app, &state)
}

#[tauri::command]
fn read_article(
    app: AppHandle,
    state: State<'_, ContentState>,
    path: String,
) -> Result<String, String> {
    content::read_article(&app, &state, &path)
}

#[tauri::command]
fn read_file_bytes(
    app: AppHandle,
    state: State<'_, ContentState>,
    path: String,
) -> Result<Vec<u8>, String> {
    content::read_file_bytes(&app, &state, &path)
}

#[tauri::command]
fn resolve_file_path(
    app: AppHandle,
    state: State<'_, ContentState>,
    path: String,
) -> Result<String, String> {
    content::resolve_file_path(&app, &state, &path)
}

#[tauri::command]
fn load_meta(app: AppHandle, state: State<'_, ContentState>) -> Result<MetaStore, String> {
    content::load_meta(&app, &state)
}

#[tauri::command]
fn save_meta(
    app: AppHandle,
    state: State<'_, ContentState>,
    meta: MetaStore,
) -> Result<(), String> {
    content::save_meta(&app, &state, &meta)
}

#[tauri::command]
fn resolve_asset(
    app: AppHandle,
    state: State<'_, ContentState>,
    column_dir: String,
    relative: String,
) -> Result<String, String> {
    content::resolve_asset(&app, &state, &column_dir, &relative)
}

#[tauri::command]
fn refresh_columns(
    app: AppHandle,
    state: State<'_, ContentState>,
) -> Result<Vec<ColumnMeta>, String> {
    content::scan_columns(&app, &state)
}

fn show_main_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let show_i = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "退出 Note Reader", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| tauri::Error::FailedToReceiveMessage)?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .menu(&menu)
        .tooltip("Note Reader")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

fn apply_close_action(app: &AppHandle, action: &str) {
    match action {
        "tray" => {
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.hide();
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}

/// When the user replaces the .app / exe while this process is still running (typical macOS DMG
/// drag-install), keep watching the on-disk binary and relaunch into the new build.
/// Hot-updater flows already relaunch themselves; manual install does not — this closes that gap.
fn spawn_binary_replace_watcher(app: AppHandle) {
    // Dev/`tauri dev` rebuilds the debug binary often; skip to avoid surprise restarts.
    if cfg!(debug_assertions) {
        return;
    }

    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let Ok(meta) = std::fs::metadata(&exe) else {
        return;
    };
    let start_len = meta.len();
    let start_modified = meta.modified().ok();

    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(2));

            let changed = match std::fs::metadata(&exe) {
                Ok(m) => {
                    let len_changed = m.len() != start_len;
                    let mtime_changed = m.modified().ok() != start_modified;
                    len_changed || mtime_changed
                }
                // Mid-copy the file may briefly vanish; treat as a replace in progress.
                Err(_) => true,
            };

            if !changed {
                continue;
            }
            if REPLACE_RESTART_ARMED.swap(true, Ordering::SeqCst) {
                return;
            }

            // Let the installer / Finder finish writing the bundle.
            std::thread::sleep(Duration::from_millis(1200));

            // Prefer relaunch; if the new binary is not ready yet, fall back to exit so the
            // user at least is not stuck on the old in-memory build.
            for _ in 0..8 {
                if std::fs::metadata(&exe).is_ok() {
                    tauri::process::restart(&app.env());
                }
                std::thread::sleep(Duration::from_millis(400));
            }
            app.exit(0);
            return;
        }
    });
}

fn handle_close_requested(window: &tauri::Window) {
    if CLOSE_PROMPT_OPEN.swap(true, Ordering::SeqCst) {
        return;
    }

    let app = window.app_handle().clone();
    let remembered = window
        .try_state::<ContentState>()
        .and_then(|state| content::close_behavior(&state));

    if let Some(action) = remembered {
        CLOSE_PROMPT_OPEN.store(false, Ordering::SeqCst);
        apply_close_action(&app, &action);
        return;
    }

    if window.emit("close-prompt", ()).is_err() {
        CLOSE_PROMPT_OPEN.store(false, Ordering::SeqCst);
    }
}

#[tauri::command]
fn get_close_behavior(state: State<'_, ContentState>) -> Option<String> {
    content::close_behavior(&state)
}

#[tauri::command]
fn set_close_behavior(
    app: AppHandle,
    state: State<'_, ContentState>,
    behavior: Option<String>,
) -> Result<(), String> {
    content::set_close_behavior(&app, &state, behavior)
}

#[tauri::command]
fn resolve_window_close(
    app: AppHandle,
    state: State<'_, ContentState>,
    action: String,
    remember: bool,
) -> Result<(), String> {
    CLOSE_PROMPT_OPEN.store(false, Ordering::SeqCst);
    let action = action.trim().to_string();
    if action != "tray" && action != "quit" && action != "cancel" {
        return Err("无效操作".into());
    }
    if remember && (action == "tray" || action == "quit") {
        content::set_close_behavior(&app, &state, Some(action.clone()))?;
    }
    if action != "cancel" {
        apply_close_action(&app, &action);
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let state = ContentState::load(app.handle());
            app.manage(state);
            setup_tray(app)?;
            spawn_binary_replace_watcher(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                handle_close_requested(window);
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_path_info,
            get_platform_info,
            get_content_root,
            get_columns_path,
            get_default_columns_path,
            set_columns_path,
            set_session_columns_path,
            complete_path_onboarding,
            list_columns,
            read_article,
            read_file_bytes,
            resolve_file_path,
            load_meta,
            save_meta,
            resolve_asset,
            refresh_columns,
            get_close_behavior,
            set_close_behavior,
            resolve_window_close
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let RunEvent::Reopen {
                has_visible_windows,
                ..
            } = event
            {
                if !has_visible_windows {
                    show_main_window(app_handle);
                }
            }
        });
}
