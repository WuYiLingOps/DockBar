//! System tray icon: the only interactive entry point the app offers outside of
//! its widget windows (widgets are created with `skip_taskbar(true)` by design).
//!
//! The icon lives in the notification area and provides a menu to open the
//! settings window, restart and exit the app.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{
    AppHandle, Wry,
    menu::{Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder, TrayIconEvent},
};

use crate::{
    app::get_app_handle,
    error::{Result, ResultLogExt},
    widgets::show_settings,
};

/// When set, `RunEvent::ExitRequested` is allowed through instead of being
/// prevented (see `app_callback` in main.rs). Tauri's `prevent_exit` already
/// ignores restarts, so only a manual exit needs this flag.
static FORCE_EXIT: AtomicBool = AtomicBool::new(false);

pub fn is_force_exit() -> bool {
    FORCE_EXIT.load(Ordering::Acquire)
}

const TRAY_ID: &str = "dockbar-tray";
const MENU_OPEN_SETTINGS: &str = "tray-open-settings";
const MENU_RESTART: &str = "tray-restart";
const MENU_EXIT: &str = "tray-exit";

/// Embeds `static/icons/32x32.png` into the binary. `include_image!` resolves
/// relative paths from `CARGO_MANIFEST_DIR` (= `src/`), not from this file.
const TRAY_ICON: tauri::image::Image<'_> = tauri::include_image!("static/icons/32x32.png");

/// Builds the tray icon and its menu. Kept alive by Tauri itself (tray icons
/// are reference-counted and registered on the app), the returned handle is
/// only for logging errors at the call site.
pub fn create_tray() -> Result<TrayIcon<Wry>> {
    let handle = get_app_handle();

    let open_settings = MenuItem::with_id(
        handle,
        MENU_OPEN_SETTINGS,
        t!("tray.open_settings"),
        true,
        None::<&str>,
    )?;
    let restart = MenuItem::with_id(handle, MENU_RESTART, t!("tray.restart"), true, None::<&str>)?;
    let exit = MenuItem::with_id(handle, MENU_EXIT, t!("tray.exit"), true, None::<&str>)?;

    let menu = Menu::with_items(handle, &[&open_settings, &restart, &exit])?;

    let tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(TRAY_ICON)
        .tooltip("DockBar")
        .menu(&menu)
        // left click opens the settings directly, menu stays on right click
        .show_menu_on_left_click(false)
        .on_menu_event(|handle, event| on_menu_event(handle, &event.id().0))
        .on_tray_icon_event(|_tray, event| {
            if let TrayIconEvent::Click { button, .. } = event
                && button == tauri::tray::MouseButton::Left
            {
                open_settings_detached();
            }
        })
        .build(handle)?;

    Ok(tray)
}

/// Opens the settings outside of the main thread. Tray events are dispatched on
/// the event-loop thread, and creating a widget webview synchronously there is
/// re-entrant (see the `WindowEvent::Destroyed` handler in `widgets/loader.rs`).
fn open_settings_detached() {
    std::thread::spawn(|| show_settings().log_error());
}

fn on_menu_event(_handle: &AppHandle<Wry>, id: &str) {
    match id {
        MENU_OPEN_SETTINGS => open_settings_detached(),
        MENU_RESTART => {
            log::info!("Restart requested from tray");
            get_app_handle().restart();
        }
        MENU_EXIT => {
            log::info!("Exit requested from tray");
            FORCE_EXIT.store(true, Ordering::Release);
            get_app_handle().exit(0);
        }
        _ => {}
    }
}
