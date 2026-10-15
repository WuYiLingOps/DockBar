// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(never_type)]

mod app;
mod cli;
mod error;
mod exposed;
mod hook;
mod logger;
mod migrations;
mod modules;
mod resources;
mod server;
mod state;
mod tauri_context;
mod tauri_handlers;
mod tauri_plugins;
mod utils;
mod widgets;
mod windows_api;

#[macro_use]
extern crate rust_i18n;
i18n!("background/i18n", fallback = "en");

/// Measures the wall time of an expression and logs it (sync or async alike).
#[macro_export]
macro_rules! measure {
    ($label:expr, $body:expr) => {{
        let __start = std::time::Instant::now();
        let __result = $body;
        log::debug!("[perf] {}: {}ms", $label, __start.elapsed().as_millis());
        __result
    }};
}

use std::sync::{OnceLock, atomic::AtomicBool};

use app::DockBarApp;
use cli::{SelfPipe, handle_console_client};
use error::Result;
use logger::DockBarLogger;
use tauri_handlers::register_invoke_handler;
use tauri_plugins::register_plugins;
use utils::{
    integrity::{is_already_running, print_initial_information, restart_as_appx},
    is_running_as_appx, was_installed_using_msix,
};

use crate::{app::get_app_handle, error::ResultLogExt, utils::constants::DOCKBAR_COMMON};

static APP_HANDLE: OnceLock<tauri::AppHandle<tauri::Wry>> = OnceLock::new();
static TOKIO_RUNTIME_HANDLE: OnceLock<tokio::runtime::Handle> = OnceLock::new();

static SILENT: AtomicBool = AtomicBool::new(false);
static VERBOSE: AtomicBool = AtomicBool::new(false);

pub fn is_local_dev() -> bool {
    cfg!(dev)
}

pub fn get_tokio_handle() -> &'static tokio::runtime::Handle {
    TOKIO_RUNTIME_HANDLE
        .get()
        .expect("Tokio runtime was not initialized")
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    if let Err(err) = DockBarLogger::init() {
        let fallback = std::env::temp_dir().join("dockbar-logger-error.log");
        let _ = std::fs::write(&fallback, format!("Failed to initialize logger: {err:?}"));
        return std::process::ExitCode::from(1);
    }

    if let Err(err) = handle_console_client().await {
        log::error!("Failed to execute command: {err:?}");
        return std::process::ExitCode::from(1);
    };

    if is_already_running() {
        SelfPipe::request_open_settings().await.log_error();
        return std::process::ExitCode::from(0);
    }

    if was_installed_using_msix() && !is_running_as_appx() {
        log::info!("GUI was installed using MSIX, restarting as appx...");
        restart_as_appx().log_error();
        return std::process::ExitCode::from(0);
    }

    TOKIO_RUNTIME_HANDLE
        .set(tokio::runtime::Handle::current())
        .expect("Failed to set runtime handle");

    rust_i18n::set_locale(&dockbar_core::state::Settings::get_app_language());

    let mut app_builder = tauri::Builder::default();
    app_builder = register_plugins(app_builder);
    app_builder = register_invoke_handler(app_builder);

    // if no custom runtime is present, the app will use the installed with the system
    if let Some(path) = crate::utils::get_fixed_runtime_path() {
        unsafe { std::env::set_var("WEBVIEW2_BROWSER_EXECUTABLE_FOLDER", path) };
    }

    let app = app_builder
        .setup(|app| {
            APP_HANDLE.set(app.handle().to_owned()).unwrap();
            tokio::spawn(async move {
                let handle = get_app_handle();
                if let Err(err) = crate::measure!("Setup", setup(handle).await) {
                    log::error!("Error while setting up: {err:?}");
                    handle.exit(1);
                }
            });
            Ok(())
        })
        .build(tauri_context::get_context())
        .expect("Error while building tauri application");

    // share the current runtime with Tauri
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    let exit_code = app.run_return(app_callback);
    std::process::ExitCode::from(exit_code as u8)
}

async fn setup(_app_handle: &tauri::AppHandle<tauri::Wry>) -> Result<()> {
    print_initial_information();
    create_main_folders()?;

    SelfPipe::start_listener()?;

    if let Err(err) = utils::integrity::validate_webview_runtime() {
        match err {
            utils::integrity::IntegrityError::WebviewRuntimeNotInstalled => {
                utils::integrity::show_not_installed_dialog(get_app_handle())?;
            }
            utils::integrity::IntegrityError::WebviewRuntimeOutdated => {
                utils::integrity::show_outdated_dialog(get_app_handle())?;
            }
        }
        return Err(format!("Integrity check failed: {err:?}").into());
    }

    if let Err(err) = crate::measure!(
        "Integrity check",
        utils::integrity::ensure_bundle_files_integrity(get_app_handle()).await
    ) {
        utils::integrity::show_bundle_integrity_dialog(get_app_handle());
        return Err(err);
    }

    DockBarApp::pre_start().await?;
    // before the widgets are created, so they can connect to the events websocket right away
    server::http::start_http_server();
    DockBarApp::start().await?;

    Ok(())
}

fn create_main_folders() -> Result<()> {
    std::fs::create_dir_all(DOCKBAR_COMMON.app_temp_dir())?;
    std::fs::create_dir_all(DOCKBAR_COMMON.app_data_dir())?;
    std::fs::create_dir_all(DOCKBAR_COMMON.app_cache_dir())?;
    std::fs::create_dir_all(DOCKBAR_COMMON.app_data_dir().join("data"))?;

    std::fs::create_dir_all(DOCKBAR_COMMON.user_icons_path())?;
    std::fs::create_dir_all(DOCKBAR_COMMON.user_widgets_path())?;
    Ok(())
}

fn app_callback(_: &tauri::AppHandle<tauri::Wry>, event: tauri::RunEvent) {
    match event {
        tauri::RunEvent::Ready => {
            log::info!("Tauri Application is ready.");
        }
        tauri::RunEvent::Resumed => {
            log::info!("Tauri Event Loop was resumed.");
        }
        tauri::RunEvent::ExitRequested { api, code, .. } => match code {
            // prevent close background on webview windows closing
            Some(_) => api.prevent_exit(),
            None => api.prevent_exit(),
        },
        tauri::RunEvent::Exit => {
            log::info!("───────────────────── Exiting DockBar ─────────────────────");
        }
        _ => {}
    }
}
