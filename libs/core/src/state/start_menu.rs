use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// An installed application entry (win32 start-menu shortcut or packaged app), as
/// exposed to widgets by the `GetStartMenuItems` command for app-picker UIs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(export))]
#[serde(default, rename_all = "camelCase")]
pub struct StartMenuEntry {
    /// display name of the app
    pub display_name: String,
    /// path to the shortcut or executable; empty for packaged apps
    pub path: PathBuf,
    /// Application user model id, present for packaged apps and some shortcuts
    pub umid: Option<String>,
}
