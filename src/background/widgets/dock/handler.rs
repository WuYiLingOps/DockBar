use std::{
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
};

use dockbar_core::{
    handlers::DockBarEvent,
    state::DockItemData,
    system_state::{Relaunch, RelaunchArguments},
};

use crate::{
    app::emit_to_webviews,
    error::Result,
    modules::start::application::StartMenuManager,
    state::application::DOCK_ITEMS_MANAGER,
    widgets::permissions::{WidgetPerm, request_widget_permission},
    windows_api::{WindowsApi, window::Window},
};
use windows::Win32::{
    System::Threading::CREATE_NO_WINDOW,
    UI::WindowsAndMessaging::{SW_MINIMIZE, WM_CLOSE},
};

/// Launch info for a newly pinned item, resolved from what the user pinned.
///
/// `.lnk` shortcuts resolve to their target program (same fields the native
/// taskbar import stores); a plain executable or document pins as-is. Files
/// without a launch target stay `None`, matching the old behavior.
pub(crate) fn resolve_relaunch_from_path(path: &Path) -> Option<Relaunch> {
    if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("lnk"))
    {
        let (target, arguments, working_dir) = WindowsApi::resolve_lnk_target(path).ok()?;
        if target.as_os_str().is_empty() {
            return None;
        }
        let working_dir = if working_dir.as_os_str().is_empty() {
            target.parent().map(|p| p.to_path_buf())
        } else {
            Some(working_dir)
        };
        let args = if arguments.is_empty() {
            None
        } else {
            Some(RelaunchArguments::String(
                arguments.to_string_lossy().to_string(),
            ))
        };
        return Some(Relaunch {
            command: target.to_string_lossy().to_string(),
            args,
            working_dir,
            icon: None,
        });
    }

    if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
    {
        return Some(Relaunch {
            command: path.to_string_lossy().to_string(),
            args: None,
            working_dir: None,
            icon: None,
        });
    }

    None
}

impl crate::tauri_handlers::Handlers {
    pub fn dock_close_app(webview: tauri::WebviewWindow, hwnd: isize) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::ManageAppWindows)?;
        let window = Window::from(hwnd);
        WindowsApi::post_message(window.hwnd(), WM_CLOSE, 0, 0)?;
        Ok(())
    }

    pub fn dock_kill_app(webview: tauri::WebviewWindow, hwnd: isize) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::KillProcesses)?;
        let window = Window::from(hwnd);
        std::process::Command::new("taskkill.exe")
            .args(["/F", "/PID", &window.process().id().to_string()])
            .creation_flags(CREATE_NO_WINDOW.0)
            .spawn()?;
        Ok(())
    }

    pub fn dock_toggle_window_state(
        webview: tauri::WebviewWindow,
        hwnd: isize,
        was_focused: bool,
    ) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::ManageAppWindows)?;
        let window = Window::from(hwnd);
        // was_focused is intented to know if the window was focused before click on the dock item
        // on click the items makes the dock being focused.
        if was_focused {
            window.show_window_async(SW_MINIMIZE)?;
        } else {
            window.unminimize()?;
            window.focus()?;
        }
        Ok(())
    }

    pub fn dock_pin_item(
        webview: tauri::WebviewWindow,
        path: PathBuf,
        umid: Option<String>,
    ) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::ModifyConfiguration)?;

        // packaged apps have no path, only the umid
        let (umid, display_name) = if path.as_os_str().is_empty() {
            let umid = umid.ok_or("Invalid path")?;
            let display_name = StartMenuManager::instance()
                .get_by_file_umid(&umid)
                .map(|item| item.display_name.clone())
                .unwrap_or_else(|| umid.clone());
            (Some(umid), display_name)
        } else {
            if !path.exists() || path.is_dir() {
                return Err("Invalid path".into());
            }
            let umid = umid.or_else(|| WindowsApi::get_file_umid(&path).ok());
            let display_name =
                WindowsApi::get_executable_display_name(&path).unwrap_or_else(|_| {
                    path.file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Unknown".to_string())
                });
            (umid, display_name)
        };

        // Shortcuts must keep their launch target: the window only ever
        // exposes the target's process path, so without it the running app
        // can't be matched to this item and the dock would show it twice.
        let relaunch = resolve_relaunch_from_path(&path);

        let item = DockItemData {
            id: uuid::Uuid::new_v4(),
            display_name,
            umid,
            path,
            pinned: true,
            prevent_pinning: false,
            relaunch,
        };

        emit_to_webviews(DockBarEvent::DockAddItem, &item);
        Ok(())
    }

    pub fn dock_import_pinned_taskbar_items(webview: tauri::WebviewWindow) -> Result<usize> {
        request_widget_permission(&webview, WidgetPerm::ModifyConfiguration)?;
        let count = DOCK_ITEMS_MANAGER.import_from_windows_taskbar()?;

        // Emit event to notify all webviews about the updated items
        if count > 0 {
            let items = DOCK_ITEMS_MANAGER.get();
            emit_to_webviews(DockBarEvent::DockItemsChanged, items);
        }

        Ok(count)
    }
}
