use std::collections::HashMap;

use crate::resource::WidgetId;
use crate::state::*;
use crate::system_state::*;

macro_rules! slu_events_declaration {
    ($(
        $(@queued($($is_queued:tt)?))?
        $name:ident$(($payload:ty))? as $value:literal,
    )*) => {
        pub struct DockBarEvent;

        #[allow(non_upper_case_globals)]
        impl DockBarEvent {
            $(
                pub const $name: &'static str = $value;
            )*

            /// Events that must be delivered one by one. Any other event is a full-state
            /// snapshot, so transports may coalesce it and deliver only its latest value.
            const QUEUED: &'static [&'static str] = &[
                $($(Self::$name, $($is_queued)?)?)*
            ];

            /// Whether the given name is a declared event.
            pub fn exists(event: &str) -> bool {
                [$(Self::$name),*].contains(&event)
            }

            /// Whether the event can't be coalesced (it carries an action or a delta, not a snapshot).
            pub fn is_queued(event: &str) -> bool {
                Self::QUEUED.contains(&event)
            }

            #[allow(dead_code)]
            pub(crate) fn generate_ts_file(path: &str) {
                let content: Vec<String> = vec![
                    "// This file was generated via rust macros. Don't modify manually.".to_owned(),
                    "export enum DockBarEvent {".to_owned(),
                    $(
                        format!("  {} = '{}',", stringify!($name), Self::$name),
                    )*
                    "}\n".to_owned(),
                ];
                std::fs::write(path, content.join("\n")).unwrap();
            }
        }

        #[derive(Serialize)]
        #[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
        #[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(export))]
        pub enum DockBarEventPayload {
            $(
                #[serde(rename = $value)]
                $name($crate::__switch! {
                    if { $($payload)? }
                    do { Box<$($payload)?> }
                    else { () }
                }),
            )*
        }
    };
}

slu_events_declaration! {
    GlobalFocusChanged(FocusedApp) as "global-focus-changed",
    GlobalMouseMove([i32; 2]) as "global-mouse-move",

    SystemMonitorsChanged(Vec<PhysicalMonitor>) as "system::monitors-changed",

    UserAppWindowsChanged(Vec<UserAppWindow>) as "user::windows-changed",
    UserAppWindowsPreviewsChanged(HashMap<isize, UserAppWindowPreview>) as "user::windows-previews-changed",

    PowerStatus(PowerStatus) as "power-status",
    PowerMode(PowerMode) as "power-mode",
    BatteriesStatus(Vec<Battery>) as "batteries-status",

    ColorsChanged(UIColors) as "colors-changed",
    DarkModeChanged(bool) as "dark-mode-changed",

    StateSettingsChanged(Settings) as "settings-changed",
    StateIconPacksChanged(Vec<IconPack>) as "icon-packs",
    StateWidgetsChanged(Vec<Widget>) as "widgets-changed",
    WidgetPermissionsChanged(HashMap<WidgetId, HashMap<WidgetPerm, WidgetPermState>>) as "widget::permissions-changed",

    StatePerformanceModeChanged(PerformanceMode) as "state::performance-mode-changed",

    @queued()
    WidgetTriggered(WidgetTriggerPayload) as "widget::triggered",

    // DockBar
    @queued()
    DockAddItem(DockItemData) as "dock::add-item",
    DockItemsChanged(DockItems) as "dock::items-changed",

    // Shortcuts
    ShortcutsPaused(bool) as "shortcuts::paused",

    // Widget debug info
    WidgetDebugInfoChanged(Vec<WidgetDebugInfo>) as "widget::debug-info-changed",
}

/// Credentials of a webview to connect to the global events websocket.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct SelfEventsToken {
    /// Identifies the webview, sent as the `token` query param.
    pub token: String,
    /// Websocket url, e.g. `ws://127.0.0.1:37074/events`.
    pub endpoint: String,
}
