use std::path::Path;

use dockbar_core::{
    handlers::DockBarEvent,
    state::{CssStyles, Dialog, DialogContent, Settings},
};

use crate::{
    app::{DockBarApp, emit_to_webviews},
    error::{Result, ResultLogExt},
    utils::constants::DOCKBAR_COMMON,
    widgets::{manager::WIDGET_MANAGER, trigger_dialog_backend},
};

use super::AppSettings;

impl AppSettings {
    pub(super) fn emit_settings(&self) -> Result<()> {
        emit_to_webviews(DockBarEvent::StateSettingsChanged, &self.settings);
        DockBarApp::on_settings_change(self)?;
        WIDGET_MANAGER.reconcile().log_error();
        Ok(())
    }

    pub(super) fn read_settings() -> Settings {
        let path = DOCKBAR_COMMON.settings_path();
        if path.exists() {
            match Settings::load(path) {
                Ok(settings) => return settings,
                Err(err) => {
                    log::error!("Failed to read settings: {err}");
                    show_corrupted_state_to_user(DOCKBAR_COMMON.settings_path());
                }
            }
        }
        Settings::default()
    }

    pub fn write_settings(&self) -> Result<()> {
        self.settings.save(DOCKBAR_COMMON.settings_path())?;
        self.emit_settings()?;
        Ok(())
    }
}

fn show_corrupted_state_to_user(path: &Path) {
    let path = path.to_path_buf();
    std::thread::spawn(move || {
        let dialog = Dialog {
            title: vec![DialogContent::Group {
                items: vec![
                    DialogContent::Icon {
                        name: "BiSolidError".to_string(),
                        styles: Some(
                            CssStyles::new()
                                .add("color", "var(--color-red-800)")
                                .add("height", "1.2rem"),
                        ),
                    },
                    DialogContent::Text {
                        value: t!("runtime.corrupted_data").to_string(),
                        styles: None,
                    },
                ],
                styles: Some(CssStyles::new().add("alignItems", "center")),
            }],
            content: vec![
                DialogContent::Text {
                    value: t!("runtime.corrupted_file").to_string(),
                    styles: None,
                },
                DialogContent::Text {
                    value: format!("{}: {:?}", t!("runtime.corrupted_file_path"), path),
                    styles: None,
                },
            ],
            ..Default::default()
        };

        trigger_dialog_backend(dialog).log_error();
    });
}
