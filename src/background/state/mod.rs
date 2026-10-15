pub mod application;
pub mod infrastructure;

use application::AppSettings;
use dockbar_core::{resource::WidgetId, system_state::MonitorId};
use uuid::Uuid;

impl AppSettings {
    pub fn is_widget_enabled(&self, widget_id: &WidgetId) -> bool {
        self.settings.is_widget_enabled(widget_id)
    }

    pub fn is_widget_enable_on_monitor(
        &self,
        widget_id: &WidgetId,
        monitor_id: &MonitorId,
    ) -> bool {
        self.settings
            .is_widget_enabled_on_monitor(widget_id, monitor_id)
    }

    pub fn is_dock_enabled(&self) -> bool {
        self.is_widget_enabled(&WidgetId::known_dock())
    }

    pub fn get_widget_instances_ids(&self, widget_id: &WidgetId) -> Vec<Uuid> {
        let config = self.settings.by_widget.others.get(widget_id);
        match config {
            Some(config) => config
                .instances
                .as_ref()
                .map_or_else(Default::default, |i| i.keys().cloned().collect()),
            None => Vec::new(),
        }
    }

    pub fn locale(&self) -> &String {
        &self.settings.language
    }
}
