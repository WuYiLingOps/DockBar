use dockbar_core::{
    handlers::SelfEventsToken, rect::Rect, resource::*, state::by_monitor::MonitorConfiguration,
    state::context_menu::*, state::settings::shortcuts::SystemShortcutDeclaration, state::*,
    system_state::*,
};

pub struct Handlers;

dockbar_core::gen_tauri_wrappers!();

pub fn register_invoke_handler(
    app_builder: tauri::Builder<tauri::Wry>,
) -> tauri::Builder<tauri::Wry> {
    app_builder.invoke_handler(dockbar_core::gen_tauri_invoke_handler!())
}
