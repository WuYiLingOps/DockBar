use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{resource::WidgetId, state::by_widget::GenericWidgetSettings};

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
pub struct MonitorSettingsByWidget(HashMap<WidgetId, GenericWidgetSettings>);

impl MonitorSettingsByWidget {
    pub fn is_widget_enabled(&self, widget_id: &WidgetId) -> bool {
        self.0
            .get(widget_id)
            .is_none_or(|settings| settings.enabled)
    }

    pub fn remove(&mut self, widget_id: &WidgetId) -> Option<GenericWidgetSettings> {
        self.0.remove(widget_id)
    }

    pub fn insert(&mut self, widget_id: WidgetId, settings: GenericWidgetSettings) {
        self.0.insert(widget_id, settings);
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(default, rename_all = "camelCase")]
pub struct MonitorConfiguration {
    /// dictionary of settings by widget
    pub by_widget: MonitorSettingsByWidget,
}
