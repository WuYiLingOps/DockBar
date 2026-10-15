#[cfg(feature = "gen-binds")]
use crate::{
    handlers::SelfEventsToken, rect::Rect, resource::*, state::by_monitor::MonitorConfiguration,
    state::context_menu::*, state::settings::shortcuts::SystemShortcutDeclaration, state::*,
    system_state::*, utils::TsVoid,
};

#[cfg(feature = "gen-binds")]
macro_rules! type_or_void {
    () => {
        TsVoid
    };
    ($type:ty) => {
        $type
    };
}

/// Selects between passing a command's result through as-is (when the command is
/// declared `@fallible`, so `Handlers::$fn_name` already returns `crate::Result<T>`)
/// or wrapping it in `Ok(..)` (when the command has no `@fallible` marker, so
/// `Handlers::$fn_name` returns a plain `T` that can never fail).
#[doc(hidden)]
#[macro_export]
macro_rules! __slu_command_result {
    (@fallible $e:expr) => {
        $e
    };
    ($e:expr) => {
        Ok($e)
    };
}

macro_rules! slu_commands_declaration {
    ($(
        $key:ident =
            $(@webview($($with_webview:tt)?))?
            $(@async($($with_async:tt)?))?
            $(@fallible($($is_fallible:tt)?))?
            $fn_name:ident(
            $(
                $(#[$attr:meta])*
                $arg:ident: $arg_type:ty
            ),*
        ) $(-> $return_type:ty)?,
    )*) => {
        #[cfg(feature = "gen-binds")]
        pub struct DockBarCommand;

        #[cfg(feature = "gen-binds")]
        impl DockBarCommand {
            pub(crate) fn generate_ts_file(path: &str) {
                let mut content: Vec<String> = std::vec::Vec::new();

                content.push("// This file was generated via rust macros. Don't modify manually.".to_owned());
                content.push("export enum DockBarCommand {".to_owned());
                $(
                    content.push(format!("  {} = '{}',", stringify!($key), stringify!($fn_name)));
                )*
                content.push("}\n".to_owned());

                std::fs::write(path, content.join("\n")).unwrap();
            }
        }

        /// Internal used as mapping of commands to their arguments
        #[cfg(feature = "gen-binds")]
        #[allow(non_camel_case_types, dead_code)]
        #[derive(Deserialize, ts_rs::TS)]
        #[serde(rename_all_fields = "camelCase")]
        #[ts(export)]
        enum DockBarCmdArgumentMap {
            $(
                $fn_name {
                    $(
                        $(#[$attr])*
                        $arg: $arg_type
                    ),*
                },
            )*
        }

        /// Internal used as mapping of commands to their return types
        #[cfg(feature = "gen-binds")]
        #[allow(non_camel_case_types, dead_code)]
        #[derive(Serialize, ts_rs::TS)]
        #[ts(export)]
        enum DockBarCmdReturnMap {
            $(
                $fn_name(type_or_void!($($return_type)?)),
            )*
        }

        #[macro_export]
        #[allow(clippy::crate_in_macro_def)]
        macro_rules! gen_tauri_wrappers {
            () => {
                $(
                    #[tauri::command(async)]
                    async fn $fn_name(
                        $(webview: tauri::WebviewWindow, $($with_webview)?)?
                        $($arg:$arg_type),*
                    ) -> crate::Result<$($return_type)?> {
                        $crate::__slu_command_result!(
                            $(@fallible $($is_fallible)?)?
                            Handlers::$fn_name($(webview, $($with_webview)?)? $($arg),*) $($($with_async)? .await)?
                        )
                    }
                )*
            };
        }

        #[macro_export]
        macro_rules! gen_tauri_invoke_handler {
            () => {
                tauri::generate_handler![
                    $(
                        $fn_name,
                    )*
                ]
            };
        }
    };
}

slu_commands_declaration! {
    // Logging
    LogFromWebview = log_from_webview(level: u8, message: String, location: String),

    // General
    OpenFile =
        @webview()
        @fallible()
        open_file(path: String), // String is used here as explorer can open uris too
    SelectFileOnExplorer =
        @fallible()
        select_file_on_explorer(path: std::path::PathBuf),
    Run =
        @webview()
        @fallible()
        run(program: String, args: Option<RelaunchArguments>, working_dir: Option<std::path::PathBuf>, elevated: bool),
    // SimulatePerm = simulate_perm(widget_id: String, perm: String),

    AppExit = app_exit(),
    AppRestart = app_restart(),
    IsDevMode = is_dev_mode() -> bool,
    IsAppxPackage = is_appx_package() -> bool,
    HasFixedRuntime = has_fixed_runtime() -> bool,

    GetFocusedApp = get_focused_app() -> FocusedApp,
    GetMousePosition = get_mouse_position() -> [i32; 2],
    GetKeyState =
        @fallible()
        get_key_state(key: String) -> bool,

    GetUserEnvs = get_user_envs() -> std::collections::HashMap<String, String>,
    ShowStartMenu =
        @fallible()
        show_start_menu(),
    GetIcon =
        @fallible()
        get_icon(
        #[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(optional = nullable))]
        path: Option<std::path::PathBuf>,
        #[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(optional = nullable))]
        umid: Option<String>
    ),
    ShowDesktop =
        @webview()
        @fallible()
        show_desktop(),

    RequestToUserInputShortcut =
        @webview()
        @async()
        @fallible()
        request_to_user_input_shortcut(callback_event: String),

    // System
    SystemGetMonitors = get_connected_monitors() -> Vec<PhysicalMonitor>,
    SetMonitorHdr =
        @fallible()
        set_monitor_hdr(id: MonitorId, state: bool),
    SetMonitorBrightness =
        @fallible()
        set_monitor_brightness(id: MonitorId, brightness: u8),
    SystemGetColors =
        @fallible()
        get_system_colors() -> UIColors,
    SystemSetAccentColor =
        @fallible()
        set_system_accent_color(color: Color),
    SystemGetDarkMode =
        @fallible()
        get_system_dark_mode() -> bool,
    SystemSetDarkMode =
        @fallible()
        set_system_dark_mode(enabled: bool),
    RegisterAppBar =
        @webview()
        @fallible()
        register_app_bar(rect: Rect, edge: AppBarEdge),
    UnregisterAppBar =
        @webview()
        @fallible()
        unregister_app_bar(),

    // DockBar Settings
    StateGetDefaultSettings =
        @fallible()
        state_get_default_settings() -> Settings,
    StateGetDefaultMonitorSettings = state_get_default_monitor_settings() -> MonitorConfiguration,

    SetAutoStart =
        @fallible()
        set_auto_start(enabled: bool),
    GetAutoStartStatus =
        @fallible()
        get_auto_start_status() -> bool,

    StateGetDockItems = state_get_dock_items() -> DockItems,
    StateWriteDockItems =
        @webview()
        @fallible()
        state_write_dock_items(items: DockItems),
    StateGetSettings =
        @fallible()
        state_get_settings(path: Option<std::path::PathBuf>) -> Settings,
    StateWriteSettings =
        @webview()
        @fallible()
        state_write_settings(settings: Settings),
    StateGetWidgets = state_get_widgets() -> Vec<std::sync::Arc<Widget>>,
    StateGetIconPacks = state_get_icon_packs() -> Vec<std::sync::Arc<IconPack>>,
    StateGetSystemShortcuts = state_get_system_shortcuts() -> Vec<SystemShortcutDeclaration>,
    StateDeleteCachedIcons =
        @async()
        @fallible()
        state_delete_cached_icons(),
    RegisterUserCustomAppIcon =
        @async()
        @fallible()
        register_user_custom_app_icon(icon_base64: String, entry: IconPackEntry),
    DeleteUserCustomAppIcon =
        @async()
        @fallible()
        delete_user_custom_app_icon(entry: IconPackEntry),
    StateGetPerformanceMode = state_get_performance_mode() -> PerformanceMode,

    // Widgets
    DebugGetWidgetsStatuses = debug_get_widgets_statuses() -> Vec<WidgetDebugInfo>,
    DebugOpenDevTools =
        @fallible()
        debug_open_dev_tools(label: String),
    TriggerWidget =
        @fallible()
        trigger_widget(payload: WidgetTriggerPayload),
    TriggerContextMenu =
        @webview()
        @fallible()
        trigger_context_menu(menu: ContextMenu, forward_to: Option<String>),
    TriggerDialog =
        @webview()
        @fallible()
        trigger_dialog(dialog: Dialog),
    SetCurrentWidgetStatus =
        @webview()
        @fallible()
        set_current_widget_status(status: WidgetStatus),
    GetSelfWindowId =
        @webview()
        @fallible()
        get_self_window_handle() -> isize,
    NormalizeSelfDevicePixelRatio =
        @webview()
        @fallible()
        normalize_self_device_pixel_ratio(),
    SetSelfPosition =
        @webview()
        @fallible()
        set_self_position(rect: Rect),
    SetSelfZOrder =
        @webview()
        @fallible()
        set_self_z_order(z_order: ZOrder),
    WriteFile =
        @webview()
        @fallible()
        write_data_file(filename: String, content: String),
    ReadFile =
        @webview()
        @fallible()
        read_data_file(filename: String) -> String,
    GetWidgetPermissions =
        get_widget_permissions() -> std::collections::HashMap<WidgetId, std::collections::HashMap<WidgetPerm, WidgetPermState>>,
    // Only callable by the settings widget.
    SetWidgetPermissions =
        @webview()
        @fallible()
        set_widget_permissions(permissions: std::collections::HashMap<WidgetId, std::collections::HashMap<WidgetPerm, WidgetPermState>>),

    // User
    GetUserAppWindows = get_user_app_windows() -> Vec<UserAppWindow>,
    GetUserAppWindowsPreviews = get_user_app_windows_previews() -> std::collections::HashMap<isize, UserAppWindowPreview>,
    GetUserAppWindowsColors = get_user_app_windows_colors() -> std::collections::HashMap<isize, UserAppWindowColors>,
    GetStartMenuItems = get_start_menu_items() -> Vec<StartMenuEntry>,

    // events server
    GetSelfToken =
        @webview()
        get_self_token() -> SelfEventsToken,

    // Power
    GetPowerStatus = get_power_status() -> PowerStatus,
    GetPowerMode = get_power_mode() -> PowerMode,
    GetBatteries = get_batteries() -> Vec<Battery>,

    // DockBar
    WegCloseApp =
        @webview()
        @fallible()
        dock_close_app(hwnd: isize),
    WegKillApp =
        @webview()
        @fallible()
        dock_kill_app(hwnd: isize),
    WegToggleWindowState =
        @webview()
        @fallible()
        dock_toggle_window_state(hwnd: isize, was_focused: bool),
    WegPinItem =
        @webview()
        @fallible()
        dock_pin_item(
        path: std::path::PathBuf,
        /// packaged apps have no path, so they are pinned by umid
        #[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(optional = nullable))]
        umid: Option<String>
    ),
    WegImportPinnedTaskbarItems =
        @webview()
        @fallible()
        dock_import_pinned_taskbar_items() -> usize,
}
