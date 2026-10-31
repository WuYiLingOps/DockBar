use std::sync::atomic::Ordering;

use tauri::{AppHandle, Emitter, Wry};
use winreg::{RegKey, enums::HKEY_CURRENT_USER};

use dockbar_core::state::shortcuts::resolve_shortcuts;

use crate::{
    APP_HANDLE,
    error::{Result, ResultLogExt},
    hook::register_win_hook,
    migrations::Migrations,
    modules::start::application::StartMenuManager,
    resources::RESOURCES,
    state::application::{AppSettings, FULL_STATE, initialize_user_resources_watcher},
    widgets::{
        manager::{GAME_MODE_ACTIVE, WIDGET_MANAGER},
        popups::shortcut_conflicts::show_shortcut_conflict_popup,
    },
    windows_api::event_window::{IS_INTERACTIVE_SESSION, create_background_window},
};

const AUTOSTART_RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const AUTOSTART_VALUE_NAME: &str = "DockBar";

/// Tauri app handle
pub fn get_app_handle<'a>() -> &'a AppHandle<Wry> {
    APP_HANDLE
        .get()
        .expect("get_app_handle called but app is still not initialized")
}

pub fn emit_to_webviews<S>(event: &str, payload: S)
where
    S: serde::Serialize,
{
    if !IS_INTERACTIVE_SESSION.load(Ordering::Acquire) {
        return;
    }

    // serialized once and shared by both transports
    let payload = match serde_json::to_string(&payload) {
        Ok(payload) => payload,
        Err(err) => {
            log::error!("Failed to serialize event {event}: {err}");
            return;
        }
    };

    crate::server::events::EVENTS_HUB.publish(event, &payload);
    // kept for backward compatibility with widgets listening via the tauri api
    get_app_handle().emit_str(event, payload).log_error();
}

pub struct DockBarApp {}

/* ============== Methods ============== */
impl DockBarApp {
    /// this fn prepares the app loading assets and settings in memory parallelly
    pub async fn pre_start() -> Result<()> {
        Migrations::run_all()?;

        let webview_check = tokio::task::spawn(async {
            crate::measure!(
                "Webview optimal state",
                crate::utils::integrity::check_for_webview_optimal_state().await
            )
        });

        crate::measure!("RESOURCES", RESOURCES.initialize().await);
        crate::measure!("START_MENU", StartMenuManager::initialize().log_error());
        crate::measure!("APP SETTINGS", FULL_STATE.load());

        if !webview_check.await.unwrap_or(false) {
            return Err("Webview optimal state check failed".into());
        }

        FULL_STATE.rcu(|state| {
            let mut state = state.cloned();
            state.complete_initialization(&RESOURCES);
            state
        });

        Ok(())
    }

    pub async fn start() -> Result<()> {
        let state = FULL_STATE.load();
        rust_i18n::set_locale(state.locale());

        WIDGET_MANAGER.reconcile()?;
        create_background_window()?;
        register_win_hook()?;

        initialize_user_resources_watcher()?;
        Ok(())
    }

    pub fn on_settings_change(state: &AppSettings) -> Result<()> {
        rust_i18n::set_locale(state.locale());

        // If game-mode suspension was just turned off while the widgets are still
        // suspended, bring them back: the resume would otherwise only happen on
        // the next PowerModeChanged event, which may never arrive.
        if !state.settings.suspend_on_game_mode
            && GAME_MODE_ACTIVE.load(std::sync::atomic::Ordering::Acquire)
        {
            WIDGET_MANAGER.resume_all().log_error();
        }

        let widgets = RESOURCES.widgets();
        let widget_refs: Vec<_> = widgets.iter().map(|w| w.as_ref()).collect();
        let (resolved, has_conflicts) = resolve_shortcuts(&state.settings, &widget_refs);
        if has_conflicts {
            show_shortcut_conflict_popup().log_error();
        }
        crate::cli::shortcuts::set_resolved_shortcuts(resolved);

        Ok(())
    }

    pub fn is_auto_start_enabled() -> Result<bool> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = hkcu.open_subkey(AUTOSTART_RUN_KEY)?;
        Ok(run_key.get_value::<String, _>(AUTOSTART_VALUE_NAME).is_ok())
    }

    pub fn set_auto_start(enabled: bool) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = hkcu.open_subkey_with_flags(
            AUTOSTART_RUN_KEY,
            winreg::enums::KEY_SET_VALUE | winreg::enums::KEY_QUERY_VALUE,
        )?;

        if enabled {
            let exe = std::env::current_exe()?;
            let command = format!("\"{}\"", exe.to_string_lossy());
            run_key.set_value(AUTOSTART_VALUE_NAME, &command)?;
        } else {
            let _ = run_key.delete_value(AUTOSTART_VALUE_NAME);
        }
        Ok(())
    }
}
