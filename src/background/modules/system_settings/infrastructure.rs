use std::sync::Once;

use dockbar_core::{
    handlers::DockBarEvent,
    system_state::{Color, UIColors},
};

use crate::{
    app::emit_to_webviews,
    error::Result,
    modules::system_settings::application::{SystemSettings, SystemSettingsEvent},
};

/// Lazy initialization wrapper that registers Tauri events on first access
/// This keeps Tauri logic separate from system logic while ensuring lazy initialization
fn get_system_settings() -> &'static SystemSettings {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        SystemSettings::subscribe(|event| {
            if event == SystemSettingsEvent::ColorChanged
                && let Ok(colors) = SystemSettings::instance().get_colors()
            {
                emit_to_webviews(DockBarEvent::ColorsChanged, &colors);
            }

            if event == SystemSettingsEvent::ColorSchemeSwitched
                && let Ok(is_dark) = SystemSettings::instance().get_dark_mode()
            {
                emit_to_webviews(DockBarEvent::DarkModeChanged, is_dark);
            }
        });
    });
    SystemSettings::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_system_colors() -> Result<UIColors> {
        get_system_settings().get_colors()
    }

    pub fn set_system_accent_color(color: Color) -> Result<()> {
        SystemSettings::set_accent_color(color)
    }

    pub fn get_system_dark_mode() -> Result<bool> {
        get_system_settings().get_dark_mode()
    }

    pub fn set_system_dark_mode(enabled: bool) -> Result<()> {
        SystemSettings::set_dark_mode(enabled)
    }
}
