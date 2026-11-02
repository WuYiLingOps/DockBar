use std::sync::Arc;

use crate::{
    app::emit_to_webviews,
    error::{Result, ResultLogExt},
    widgets::manager::WIDGET_MANAGER,
    widgets::permissions::WIDGET_PERMISSIONS,
};
use dockbar_core::{handlers::DockBarEvent, resource::ResourceKind, state::Widget};

use super::ResourceManager;

// Reusable getters for resources.
impl ResourceManager {
    pub fn widgets(&self) -> Vec<Arc<Widget>> {
        let mut widgets = Vec::new();
        self.widgets.iter_sync(|_, v| {
            widgets.push(v.clone());
            true
        });
        widgets
    }

    pub fn icon_packs(&self) -> Vec<Arc<dockbar_core::state::IconPack>> {
        let mut icon_packs = Vec::new();
        // Add system icon pack if it exists
        if let Some(system_pack) = self.system_icon_pack.lock().as_ref() {
            icon_packs.push(std::sync::Arc::new(system_pack.clone()));
        }
        // Add user icon packs
        self.icon_packs.iter_sync(|_, v| {
            icon_packs.push(v.clone());
            true
        });
        icon_packs
    }
}

impl ResourceManager {
    pub fn emit_widgets(&self) -> Result<()> {
        let widgets = self.widgets();
        emit_to_webviews(DockBarEvent::StateWidgetsChanged, widgets.clone());
        WIDGET_PERMISSIONS.remove_uninstalled_widgets().log_error();

        WIDGET_MANAGER.reconcile()?;
        Ok(())
    }

    pub fn emit_icon_packs(&self) {
        emit_to_webviews(DockBarEvent::StateIconPacksChanged, self.icon_packs())
    }

    pub fn emit_kind_changed(&self, kind: &ResourceKind) -> Result<()> {
        match kind {
            ResourceKind::Widget => self.emit_widgets()?,
            ResourceKind::IconPack => self.emit_icon_packs(),
        }
        Ok(())
    }
}
