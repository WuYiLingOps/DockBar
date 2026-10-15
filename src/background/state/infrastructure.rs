use std::path::PathBuf;

use dockbar_core::state::{DockItems, PerformanceMode, Settings, by_monitor::MonitorConfiguration};
use tauri::WebviewWindow;

use crate::{
    error::Result,
    state::application::performance::PERFORMANCE_MODE,
    widgets::permissions::{WidgetPerm, request_widget_permission},
};

use super::application::{DOCK_ITEMS_MANAGER, FULL_STATE};

impl crate::tauri_handlers::Handlers {
    pub fn state_get_dock_items() -> DockItems {
        DOCK_ITEMS_MANAGER.get()
    }

    pub fn state_write_dock_items(webview: WebviewWindow, items: DockItems) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::ModifyConfiguration)?;
        DOCK_ITEMS_MANAGER.write(items)
    }

    pub fn state_get_settings(path: Option<PathBuf>) -> Result<Settings> {
        if let Some(path) = path {
            Ok(Settings::load(path)?)
        } else {
            Ok(FULL_STATE.load().settings.clone())
        }
    }

    pub fn state_get_default_settings() -> Result<Settings> {
        let mut settings = Settings::default();
        settings.sanitize()?;
        Ok(settings)
    }

    pub fn state_get_default_monitor_settings() -> MonitorConfiguration {
        MonitorConfiguration::default()
    }

    pub fn state_write_settings(webview: WebviewWindow, mut settings: Settings) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::ModifyConfiguration)?;
        settings.sanitize()?;
        FULL_STATE.rcu(move |state| {
            let mut state = state.cloned();
            state.settings = settings.clone();
            state
        });
        FULL_STATE.load().write_settings()?;
        Ok(())
    }

    pub fn state_get_performance_mode() -> PerformanceMode {
        PERFORMANCE_MODE.load()
    }
}
