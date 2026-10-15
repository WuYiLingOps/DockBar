use std::{collections::HashMap, sync::Once};

use dockbar_core::{
    handlers::DockBarEvent,
    state::StartMenuEntry,
    system_state::{FocusedApp, UserAppWindow, UserAppWindowColors, UserAppWindowPreview},
};
use windows::Win32::UI::Shell::{IShellDispatch6, Shell};

use crate::{
    app::emit_to_webviews,
    error::Result,
    modules::apps::application::{UserAppsManager, previews::WinPreviewManager},
    modules::start::application::StartMenuManager,
    widgets::permissions::{WidgetPerm, request_widget_permission},
    windows_api::{Com, input::Mouse, window::Window},
};

/// Lazy initialization wrapper that registers Tauri events on first access
/// This keeps Tauri logic separate from system logic while ensuring lazy initialization
fn get_apps_manager() -> &'static UserAppsManager {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        UserAppsManager::subscribe(|_event| {
            emit_to_webviews(
                DockBarEvent::UserAppWindowsChanged,
                UserAppsManager::instance().interactable_windows.to_vec(),
            );
        });

        WinPreviewManager::subscribe(|event| {
            use crate::modules::apps::application::previews::WinPreviewEvent;
            match event {
                WinPreviewEvent::Captured(_) | WinPreviewEvent::Cleaned(_) => {
                    emit_to_webviews(
                        DockBarEvent::UserAppWindowsPreviewsChanged,
                        WinPreviewManager::instance().get_previews(),
                    );
                }
                WinPreviewEvent::ColorsUpdated(_) => {
                    emit_to_webviews(
                        DockBarEvent::UserAppWindowsColorsChanged,
                        WinPreviewManager::instance().get_colors(),
                    );
                }
            }
        });
    });
    UserAppsManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_focused_app() -> FocusedApp {
        Window::get_foregrounded().as_focused_app_information()
    }

    pub fn get_mouse_position() -> [i32; 2] {
        let point = Mouse::get_cursor_pos().unwrap_or_default();
        [point.x, point.y]
    }

    pub fn get_key_state(key: String) -> Result<bool> {
        use win_hotkeys::VKey;
        use win_hotkeys::state::KeyboardState;

        let key = VKey::from_keyname(&key)?;
        if key == VKey::Menu {
            return Ok(KeyboardState::async_is_key_down(VKey::Menu.to_vk_code())
                || KeyboardState::async_is_key_down(VKey::LMenu.to_vk_code())
                || KeyboardState::async_is_key_down(VKey::RMenu.to_vk_code()));
        }

        Ok(KeyboardState::async_is_key_down(key.to_vk_code()))
    }

    pub fn get_user_app_windows() -> Vec<UserAppWindow> {
        get_apps_manager().interactable_windows.to_vec()
    }

    pub fn get_user_app_windows_previews() -> HashMap<isize, UserAppWindowPreview> {
        get_apps_manager();
        WinPreviewManager::instance().get_previews()
    }

    pub fn get_user_app_windows_colors() -> HashMap<isize, UserAppWindowColors> {
        get_apps_manager();
        WinPreviewManager::instance().get_colors()
    }

    /// Installed apps (start-menu shortcuts and packaged apps) for app-picker UIs.
    /// The list is empty until `StartMenuManager::initialize` finishes at startup.
    ///
    /// The start-menu folders also contain non-shortcut files (desktop.ini, .url, ...),
    /// which the manager keeps for name resolution; only launchable entries are
    /// exposed here: `.lnk` shortcuts and packaged apps (umid, empty path).
    pub fn get_start_menu_items() -> Vec<StartMenuEntry> {
        StartMenuManager::instance()
            .get_all()
            .into_iter()
            .filter(|item| {
                item.path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("lnk"))
                    || item.umid.is_some()
            })
            .map(|item| StartMenuEntry {
                display_name: item.display_name.clone(),
                path: item.path.clone(),
                umid: item.umid.clone(),
            })
            .collect()
    }

    /// This function is called show_desktop but acts more like minimize_all
    pub fn show_desktop(webview: tauri::WebviewWindow) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::ManageAppWindows)?;
        Com::run_with_context(|| {
            let shell: IShellDispatch6 = Com::create_instance(&Shell)?;
            unsafe { shell.ToggleDesktop()? };
            Ok(())
        })
    }
}
