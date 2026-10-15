mod dock_items;
mod icon_pack;
pub mod settings;
mod start_menu;
mod widget;

pub use dock_items::*;
pub use icon_pack::*;
pub use settings::*;
pub use start_menu::*;
pub use widget::dialog::*;
pub use widget::permissions::*;
pub use widget::*;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsBackup {
    pub data: serde_json::Value,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
