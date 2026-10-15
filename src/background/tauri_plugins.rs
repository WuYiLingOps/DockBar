use tauri::{Builder, Wry};

pub fn register_plugins(app_builder: Builder<Wry>) -> Builder<Wry> {
    // tauri_plugin_deep_link is intentionally omitted: the URI scheme (dockbar.uri) is
    // declared in tauri.conf.json only for the bundler to inject it into installer.nsi;
    // no runtime URL callback handling is needed.
    //
    // tauri_plugin_updater is removed: this build has no update server and no keys.
    app_builder.plugin(tauri_plugin_dialog::init())
}
