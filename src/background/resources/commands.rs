use dockbar_core::{
    resource::DockBarResource,
    state::{IconPack, Widget},
};

use crate::{error::Result, resources::RESOURCES, utils::icon_extractor::queue::IconExtractor};
use std::sync::Arc;

impl crate::tauri_handlers::Handlers {
    pub fn state_get_widgets() -> Vec<Arc<Widget>> {
        RESOURCES.widgets()
    }

    pub fn state_get_icon_packs() -> Vec<Arc<IconPack>> {
        RESOURCES.icon_packs()
    }

    pub async fn state_delete_cached_icons() -> Result<()> {
        // Take the pack before awaiting — never hold MutexGuard across an await point
        let pack = { RESOURCES.system_icon_pack.lock().take() };
        if let Some(pack) = pack {
            pack.delete().await?;
        }
        IconExtractor::instance().clear_failures();
        RESOURCES.ensure_system_icon_pack(true)?;
        RESOURCES.emit_icon_packs();
        Ok(())
    }
}
