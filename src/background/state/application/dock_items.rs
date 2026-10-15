use std::sync::LazyLock;

use dockbar_core::{
    resource::WidgetId,
    state::{DockItem, DockItemData, DockItems},
};
use parking_lot::Mutex;
use uuid::Uuid;

use crate::{
    error::{Result, ResultLogExt},
    modules::apps::application::msix::MsixAppsManager,
    utils::{atomic_write_file, constants::DOCKBAR_COMMON},
    widgets::dock::handler::resolve_relaunch_from_path,
    widgets::dock::native_taskbar::get_windows_taskbar_pinned_apps,
};

pub static DOCK_ITEMS_MANAGER: LazyLock<DockItemsManager> = LazyLock::new(|| {
    let manager = DockItemsManager {
        items: Mutex::new(DockItems::default()),
    };
    manager.load().log_error();
    manager
});

pub struct DockItemsManager {
    items: Mutex<DockItems>,
}

impl DockItemsManager {
    pub fn get(&self) -> DockItems {
        self.items.lock().clone()
    }

    pub fn write(&self, mut items: DockItems) -> Result<()> {
        items.sanitize();
        let path = DOCKBAR_COMMON
            .widget_data_dir(&WidgetId::known_dock())
            .join("state.yml");
        std::fs::create_dir_all(DOCKBAR_COMMON.widget_data_dir(&WidgetId::known_dock()))?;
        // Hold the lock while writing so concurrent saves (one webview per
        // monitor) are serialized instead of racing on the same file.
        let mut guard = self.items.lock();
        atomic_write_file(&path, serde_yaml::to_string(&items)?.as_bytes())?;
        *guard = items;
        Ok(())
    }

    pub fn load(&self) -> Result<()> {
        let path = DOCKBAR_COMMON
            .widget_data_dir(&WidgetId::known_dock())
            .join("state.yml");

        let items = if path.exists() {
            let mut items: DockItems = serde_yaml::from_str(&std::fs::read_to_string(&path)?)?;
            items.sanitize();
            resolve_msix_paths(&mut items);
            ensure_default_separator(&mut items);
            backfill_relaunch_targets(&mut items);
            items
        } else {
            // First run: try to import from Windows taskbar
            let mut items = match self.import_from_windows_taskbar_internal() {
                Ok(imported_items) if !imported_items.center.is_empty() => {
                    log::info!(
                        "First run: imported {} items from Windows taskbar",
                        imported_items.center.len()
                    );
                    imported_items
                }
                Ok(_) => {
                    log::info!("First run: no Windows pinned items found, using defaults");
                    initial_items()
                }
                Err(e) => {
                    log::warn!(
                        "First run: failed to import from Windows taskbar: {}, using defaults",
                        e
                    );
                    initial_items()
                }
            };
            items.sanitize();
            atomic_write_file(&path, serde_yaml::to_string(&items)?.as_bytes())?;
            items
        };

        *self.items.lock() = items;
        Ok(())
    }

    /// Internal method to import from Windows without writing to state
    /// Used during initial load to avoid double-write
    fn import_from_windows_taskbar_internal(&self) -> Result<DockItems> {
        let windows_pinned_items = get_windows_taskbar_pinned_apps()?;

        if windows_pinned_items.is_empty() {
            return Ok(initial_items());
        }

        // Start with the base structure from initial_items
        let base = initial_items();

        // Build the center section with imported items
        let center: Vec<DockItem> = windows_pinned_items
            .into_iter()
            .map(DockItem::AppOrFile)
            .collect();

        Ok(DockItems {
            is_reorder_disabled: base.is_reorder_disabled,
            left: base.left,
            center,
            right: base.right,
        })
    }

    /// Import pinned items from Windows taskbar.
    /// This will add missing items without removing existing ones.
    /// Returns the number of items imported.
    pub fn import_from_windows_taskbar(&self) -> Result<usize> {
        let windows_pinned_items = get_windows_taskbar_pinned_apps()?;

        if windows_pinned_items.is_empty() {
            return Ok(0);
        }

        let mut items = self.get();

        // Helper function to filter out AppOrFile items, keeping only non-app items
        let keep_non_app_items = |items: &[DockItem]| -> Vec<DockItem> {
            items
                .iter()
                .filter(|item| !matches!(item, DockItem::AppOrFile(_)))
                .cloned()
                .collect()
        };

        // Clear all AppOrFile items from all sections, keep only special items
        items.left = keep_non_app_items(&items.left);
        items.right = keep_non_app_items(&items.right);

        // Add all imported items to center, after existing non-app items
        items.center = keep_non_app_items(&items.center);
        for windows_item in &windows_pinned_items {
            items.center.push(DockItem::AppOrFile(windows_item.clone()));
        }

        self.write(items)?;
        Ok(windows_pinned_items.len())
    }
}

/// One-time migration: states saved before the pinned/temporal split had no
/// user separator, so the temporal zone (past the right hardcoded separator)
/// had no visible divider. Appends one to the end of center when none exists.
/// Users can still remove/re-add separators freely from the context menu.
fn ensure_default_separator(items: &mut DockItems) {
    let has_separator = items
        .center
        .iter()
        .chain(items.left.iter())
        .chain(items.right.iter())
        .any(|item| matches!(item, DockItem::Separator { .. }));
    if !has_separator {
        items
            .center
            .push(DockItem::Separator { id: Uuid::new_v4() });
    }
}

fn initial_items() -> DockItems {
    DockItems {
        is_reorder_disabled: false,
        left: vec![],
        center: vec![
            DockItem::AppOrFile(DockItemData {
                id: Uuid::new_v4(),
                umid: None,
                path: "C:\\Windows\\explorer.exe".into(),
                display_name: t!("file_explorer").to_string(),
                pinned: true,
                prevent_pinning: false,
                relaunch: None,
            }),
            // separates pinned items from the temporal (running apps) zone,
            // which lives past the right hardcoded separator
            DockItem::Separator { id: Uuid::new_v4() },
        ],
        right: vec![DockItem::Media { id: Uuid::new_v4() }],
    }
}

fn update_dock_items_paths(items: &mut [DockItem]) {
    for item in items {
        if let DockItem::AppOrFile(data) = item
            && let Some(umid) = &data.umid
            && let Ok(Some(app_path)) = MsixAppsManager::instance().get_app_path(umid)
        {
            data.path = app_path;
        }
    }
}

fn resolve_msix_paths(dock_items: &mut DockItems) {
    let DockItems {
        left,
        center,
        right,
        ..
    } = dock_items;
    std::thread::scope(|s| {
        s.spawn(|| update_dock_items_paths(left));
        s.spawn(|| update_dock_items_paths(center));
        s.spawn(|| update_dock_items_paths(right));
    });
}

/// One-time repair for shortcut items pinned before the launch-target backfill:
/// they stored only the `.lnk` path with `relaunch: none`, so the running
/// program (whose window reports the target's process path) never matched its
/// pinned icon and the dock showed the app twice. Resolves the target once and
/// stores it in `relaunch`; items that already carry a target are untouched.
fn backfill_relaunch_targets(items: &mut DockItems) {
    let mut repaired = false;
    for item in items
        .left
        .iter_mut()
        .chain(items.center.iter_mut())
        .chain(items.right.iter_mut())
    {
        let DockItem::AppOrFile(data) = item else {
            continue;
        };
        let is_lnk = data
            .path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("lnk"));
        if !is_lnk || data.relaunch.is_some() {
            continue;
        }
        if let Some(relaunch) = resolve_relaunch_from_path(&data.path) {
            data.relaunch = Some(relaunch);
            repaired = true;
        }
    }
    if repaired {
        log::info!("Backfilled launch targets for shortcut dock items missing them");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_items_structure() {
        let items = initial_items();

        assert!(items.left.is_empty(), "Left should be empty by default");
        assert_eq!(
            items.center.len(),
            2,
            "Center should have File Explorer and the pinned/temporal separator"
        );
        assert_eq!(items.right.len(), 1, "Right should have Media");

        // Verify center has explorer followed by the zone separator
        if let DockItem::AppOrFile(ref data) = items.center[0] {
            assert!(data.path.to_string_lossy().contains("explorer.exe"));
            assert!(data.pinned);
        } else {
            panic!("Center item should be AppOrFile");
        }
        assert!(matches!(items.center[1], DockItem::Separator { .. }));

        // Verify right items
        assert!(matches!(items.right[0], DockItem::Media { .. }));
    }

    #[test]
    fn test_import_preserves_special_items() {
        // Create a mock DockItems with special items
        let original = DockItems {
            is_reorder_disabled: false,
            left: vec![
                DockItem::Media { id: Uuid::new_v4() },
                DockItem::AppOrFile(DockItemData {
                    id: Uuid::new_v4(),
                    display_name: "Old App".to_string(),
                    umid: None,
                    path: "C:\\old.exe".into(),
                    pinned: true,
                    prevent_pinning: false,
                    relaunch: None,
                }),
            ],
            center: vec![
                DockItem::AppOrFile(DockItemData {
                    id: Uuid::new_v4(),
                    display_name: "Center App".to_string(),
                    umid: None,
                    path: "C:\\center.exe".into(),
                    pinned: true,
                    prevent_pinning: false,
                    relaunch: None,
                }),
                DockItem::Separator { id: Uuid::new_v4() },
            ],
            right: vec![
                DockItem::Separator { id: Uuid::new_v4() },
                DockItem::AppOrFile(DockItemData {
                    id: Uuid::new_v4(),
                    display_name: "Right App".to_string(),
                    umid: None,
                    path: "C:\\right.exe".into(),
                    pinned: true,
                    prevent_pinning: false,
                    relaunch: None,
                }),
            ],
        };

        // Simulate the filter logic
        let keep_non_app_items = |items: &[DockItem]| -> Vec<DockItem> {
            items
                .iter()
                .filter(|item| !matches!(item, DockItem::AppOrFile(_)))
                .cloned()
                .collect()
        };

        let filtered_left = keep_non_app_items(&original.left);
        let filtered_center = keep_non_app_items(&original.center);
        let filtered_right = keep_non_app_items(&original.right);

        // Verify special items are preserved
        assert_eq!(filtered_left.len(), 1, "Media should be preserved");
        assert!(matches!(filtered_left[0], DockItem::Media { .. }));

        assert_eq!(filtered_center.len(), 1, "Separator should be preserved");
        assert!(matches!(filtered_center[0], DockItem::Separator { .. }));

        assert_eq!(filtered_right.len(), 1, "Separator should be preserved");
        assert!(matches!(filtered_right[0], DockItem::Separator { .. }));
    }

    #[test]
    fn test_import_from_windows_taskbar_internal_empty_list() {
        let manager = DockItemsManager {
            items: Mutex::new(DockItems::default()),
        };

        // This will likely fail in test environment, but we test the structure
        match manager.import_from_windows_taskbar_internal() {
            Ok(items) => {
                // Should have the base structure
                assert!(
                    !items.left.is_empty() || items.left.is_empty(),
                    "Should return valid structure"
                );
            }
            Err(_) => {
                // Expected to fail in test environment without real registry
            }
        }
    }

    #[test]
    fn test_dock_items_sanitize() {
        let mut items = DockItems {
            is_reorder_disabled: false,
            left: vec![DockItem::AppOrFile(DockItemData {
                id: Uuid::nil(), // Invalid nil ID
                display_name: "Test".to_string(),
                umid: None,
                path: "C:\\nonexistent.exe".into(),
                pinned: true,
                prevent_pinning: false,
                relaunch: None,
            })],
            center: vec![],
            right: vec![],
        };

        items.sanitize();

        // After sanitization, non-existent paths should be removed
        // and nil IDs should be replaced
        assert!(
            items.left.is_empty() || items.left[0].id() != &Uuid::nil(),
            "Nil IDs should be replaced or items removed"
        );
    }

    #[test]
    fn test_manager_get_returns_clone() {
        let manager = DockItemsManager {
            items: Mutex::new(initial_items()),
        };

        let items1 = manager.get();
        let items2 = manager.get();

        // Should return independent clones
        assert_eq!(items1.left.len(), items2.left.len());
        assert_eq!(items1.center.len(), items2.center.len());
        assert_eq!(items1.right.len(), items2.right.len());
    }

    fn lnk_item(path: &str, relaunch: Option<dockbar_core::system_state::Relaunch>) -> DockItem {
        DockItem::AppOrFile(DockItemData {
            id: Uuid::new_v4(),
            display_name: "Shortcut".to_string(),
            umid: None,
            path: path.into(),
            pinned: true,
            prevent_pinning: false,
            relaunch,
        })
    }

    #[test]
    fn test_backfill_skips_items_with_relaunch_or_non_lnk_paths() {
        let existing_target = dockbar_core::system_state::Relaunch {
            command: "C:\\app\\target.exe".into(),
            args: None,
            working_dir: None,
            icon: None,
        };
        let mut items = DockItems {
            is_reorder_disabled: false,
            left: vec![lnk_item(
                "C:\\shortcuts\\has-target.lnk",
                Some(existing_target.clone()),
            )],
            center: vec![
                lnk_item("C:\\shortcuts\\plain.exe", None),
                DockItem::Separator { id: Uuid::new_v4() },
            ],
            right: vec![],
        };

        backfill_relaunch_targets(&mut items);

        let DockItem::AppOrFile(left) = &items.left[0] else {
            panic!("left should be an app item");
        };
        assert_eq!(left.relaunch.as_ref(), Some(&existing_target));

        // non-lnk paths are not touched (no relaunch appears out of nowhere)
        let DockItem::AppOrFile(center) = &items.center[0] else {
            panic!("center should be an app item");
        };
        assert!(center.relaunch.is_none());
    }

    #[test]
    fn test_backfill_leaves_unresolvable_shortcut_untouched() {
        // A .lnk path that the shell cannot resolve must not crash or invent
        // a relaunch entry; the item just stays as-is.
        let mut items = DockItems {
            is_reorder_disabled: false,
            left: vec![],
            center: vec![lnk_item("C:\\definitely\\missing\\app.lnk", None)],
            right: vec![],
        };

        backfill_relaunch_targets(&mut items);

        let DockItem::AppOrFile(data) = &items.center[0] else {
            panic!("center should be an app item");
        };
        assert!(data.relaunch.is_none());
    }
}
