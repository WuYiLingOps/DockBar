use std::{collections::HashMap, path::PathBuf};

use dockbar_core::system_state::RelaunchArguments;

use tauri::WebviewWindow;

use crate::{
    app::{DockBarApp, get_app_handle},
    error::Result,
    utils::{
        constants::DOCKBAR_COMMON,
        icon_extractor::{request_icon_extraction_from_file, request_icon_extraction_from_umid},
    },
    widgets::{
        permissions::{WidgetPerm, request_widget_permission},
        popups::shortcut_registering::REG_SHORTCUT_DATA,
    },
    windows_api::WindowsApi,
};

impl crate::tauri_handlers::Handlers {
    pub fn log_from_webview(level: u8, message: String, location: String) {
        let level = match level {
            1 => log::Level::Trace,
            2 => log::Level::Debug,
            3 => log::Level::Info,
            4 => log::Level::Warn,
            _ => log::Level::Error,
        };
        log::log!(target: &location, level, "{message}");
    }

    pub fn open_file(webview: tauri::WebviewWindow, path: String) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::OpenFile)?;
        WindowsApi::execute(path, None, None, false)
    }

    pub fn select_file_on_explorer(path: PathBuf) -> Result<()> {
        let path = path.to_string_lossy().to_string();
        std::process::Command::new(DOCKBAR_COMMON.system_dir().join("explorer.exe"))
            .args(["/select,", &path])
            .spawn()?;
        Ok(())
    }

    pub fn run(
        webview: tauri::WebviewWindow,
        program: String,
        args: Option<RelaunchArguments>,
        working_dir: Option<PathBuf>,
        elevated: bool,
    ) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::Run)?;
        let args = args.map(|args| args.to_string());
        WindowsApi::execute(program, args, working_dir, elevated)
    }

    pub fn app_exit() {
        get_app_handle().exit(0);
    }

    pub fn app_restart() {
        get_app_handle().restart();
    }

    pub fn is_dev_mode() -> bool {
        tauri::is_dev()
    }

    pub fn has_fixed_runtime() -> bool {
        crate::utils::has_fixed_runtime()
    }

    pub fn is_appx_package() -> bool {
        crate::utils::is_running_as_appx()
    }

    pub fn get_user_envs() -> HashMap<String, String> {
        std::env::vars().collect::<HashMap<String, String>>()
    }

    pub fn set_auto_start(enabled: bool) -> Result<()> {
        DockBarApp::set_auto_start(enabled)
    }

    pub fn get_auto_start_status() -> Result<bool> {
        DockBarApp::is_auto_start_enabled()
    }

    // used to request icon extraction
    pub fn get_icon(path: Option<PathBuf>, umid: Option<String>) -> Result<()> {
        if let Some(umid) = umid {
            request_icon_extraction_from_umid(&umid.into());
        }
        if let Some(path) = path {
            request_icon_extraction_from_file(&path);
        }
        Ok(())
    }

    pub async fn request_to_user_input_shortcut(
        window: WebviewWindow,
        callback_event: String,
    ) -> Result<()> {
        // The privileged service (which captured raw key input) was removed; the
        // registering dialog still works but only accepts shortcuts typed through
        // the manual edit flow.
        let mut data = REG_SHORTCUT_DATA.lock();
        data.response_view_label = Some(window.label().to_string());
        data.response_event = Some(callback_event);
        Ok(())
    }
}
