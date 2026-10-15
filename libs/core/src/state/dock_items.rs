use std::{collections::HashSet, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::system_state::{Relaunch, RelaunchArguments};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(default, rename_all = "camelCase")]
pub struct DockItemData {
    /// internal UUID to differentiate items
    pub id: uuid::Uuid,
    /// display name of the item
    pub display_name: String,
    /// Application user model id.
    pub umid: Option<String>,
    /// path to file or program.
    pub path: PathBuf,
    /// the item will persist after all windows are closed
    pub pinned: bool,
    /// this item should not be pinnable
    pub prevent_pinning: bool,
    /// custom information to relaunch this app, if none, UMID or path should be used
    pub relaunch: Option<Relaunch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(tag = "type")]
pub enum DockItem {
    #[serde(alias = "PinnedApp", alias = "Pinned")]
    DeprecatedOldPinned(OldDockItemData),
    AppOrFile(DockItemData),
    Separator {
        id: uuid::Uuid,
    },
    Media {
        id: uuid::Uuid,
    },
    /// deprecated: dropped on load
    #[serde(rename = "StartMenu")]
    DeprecatedStartMenu {
        id: uuid::Uuid,
    },
    /// deprecated: dropped on load
    #[serde(rename = "ShowDesktop")]
    DeprecatedShowDesktop {
        id: uuid::Uuid,
    },
    /// deprecated: dropped on load
    #[serde(rename = "TrashBin")]
    DeprecatedTrashBin {
        id: uuid::Uuid,
    },
    /// deprecated: kept only so old state files still deserialize; dropped on load
    #[serde(rename = "Plugin")]
    DeprecatedPlugin {
        id: uuid::Uuid,
        plugin: String,
    },
}

impl DockItem {
    pub fn id(&self) -> &uuid::Uuid {
        match self {
            DockItem::DeprecatedOldPinned(data) => &data.id,
            DockItem::AppOrFile(data) => &data.id,
            DockItem::Separator { id } => id,
            DockItem::Media { id } => id,
            DockItem::DeprecatedStartMenu { id } => id,
            DockItem::DeprecatedShowDesktop { id } => id,
            DockItem::DeprecatedTrashBin { id } => id,
            DockItem::DeprecatedPlugin { id, .. } => id,
        }
    }

    fn set_id(&mut self, identifier: uuid::Uuid) {
        match self {
            DockItem::DeprecatedOldPinned(data) => data.id = identifier,
            DockItem::AppOrFile(data) => data.id = identifier,
            DockItem::Separator { id } => *id = identifier,
            DockItem::Media { id } => *id = identifier,
            DockItem::DeprecatedStartMenu { id } => *id = identifier,
            DockItem::DeprecatedShowDesktop { id } => *id = identifier,
            DockItem::DeprecatedTrashBin { id } => *id = identifier,
            DockItem::DeprecatedPlugin { id, .. } => *id = identifier,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(export))]
pub struct DockItems {
    /// Whether the reordering possible on the weg
    pub is_reorder_disabled: bool,
    pub left: Vec<DockItem>,
    pub center: Vec<DockItem>,
    pub right: Vec<DockItem>,
}

impl DockItems {
    fn migrate_item(item: DockItem) -> Option<DockItem> {
        let mut data = match item {
            DockItem::DeprecatedOldPinned(data) => data,
            DockItem::DeprecatedStartMenu { .. }
            | DockItem::DeprecatedShowDesktop { .. }
            | DockItem::DeprecatedTrashBin { .. }
            | DockItem::DeprecatedPlugin { .. } => return None,
            other => return Some(other),
        };

        // migration step for items before v2.1.6
        if data.subtype == OldDockItemSubtype::UnknownV2_1_6 {
            data.subtype = if data.relaunch_program.to_lowercase().contains(".exe") {
                OldDockItemSubtype::App
            } else if data.path.is_dir() {
                OldDockItemSubtype::Folder
            } else {
                OldDockItemSubtype::File
            };
        }

        if data.subtype == OldDockItemSubtype::Folder {
            return None;
        }

        // migration of old scheme before v2.5
        if let Some(args) = &data.relaunch_args
            && data.relaunch_program.contains("explorer")
            && args.to_string().starts_with("shell:AppsFolder")
        {
            data.relaunch_program = args.to_string();
            data.relaunch_args = None;
        }

        if data.relaunch_program.is_empty() {
            data.relaunch_program = data.path.to_string_lossy().to_string();
        }

        let relaunch = Some(Relaunch {
            command: data.relaunch_program,
            args: data.relaunch_args,
            working_dir: data.relaunch_in,
            icon: None,
        });

        Some(DockItem::AppOrFile(DockItemData {
            id: data.id,
            display_name: data.display_name,
            umid: data.umid,
            path: data.path,
            pinned: true,
            prevent_pinning: data.pin_disabled,
            relaunch,
        }))
    }

    fn migrate_items(items: Vec<DockItem>) -> Vec<DockItem> {
        items.into_iter().filter_map(Self::migrate_item).collect()
    }

    pub fn migrate(&mut self) {
        self.left = Self::migrate_items(std::mem::take(&mut self.left));
        self.center = Self::migrate_items(std::mem::take(&mut self.center));
        self.right = Self::migrate_items(std::mem::take(&mut self.right));
    }

    fn sanitize_items(dict: &mut HashSet<uuid::Uuid>, items: Vec<DockItem>) -> Vec<DockItem> {
        let mut result = Vec::new();
        for mut item in items {
            if let DockItem::AppOrFile(data) = &item {
                let should_ensure_path =
                    data.umid.is_none() || data.path.extension().is_some_and(|e| e == "lnk");
                if should_ensure_path && !data.path.exists() {
                    continue;
                }
            }

            if item.id().is_nil() {
                item.set_id(uuid::Uuid::new_v4());
            }

            if !dict.contains(item.id()) {
                dict.insert(*item.id());
                result.push(item);
            }
        }
        result
    }

    pub fn sanitize(&mut self) {
        self.migrate();
        let mut dict = HashSet::new();
        self.left = Self::sanitize_items(&mut dict, std::mem::take(&mut self.left));
        self.center = Self::sanitize_items(&mut dict, std::mem::take(&mut self.center));
        self.right = Self::sanitize_items(&mut dict, std::mem::take(&mut self.right));
    }
}

// ===================== DEPRECATED STRUCTS =====================

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(repr(enum = name)))]
pub enum OldDockItemSubtype {
    File,
    Folder,
    App,
    #[default]
    UnknownV2_1_6,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(default, rename_all = "camelCase")]
pub struct OldDockItemData {
    /// internal UUID to differentiate items
    pub id: uuid::Uuid,
    /// Subtype of the item (mandatory, but is optional for backward compatibility)
    pub subtype: OldDockItemSubtype,
    /// Application user model id.
    pub umid: Option<String>,
    /// path to file, folder or program.
    pub path: PathBuf,
    /// program to be executed
    pub relaunch_program: String,
    /// arguments to be passed to the relaunch program
    pub relaunch_args: Option<RelaunchArguments>,
    /// path where ejecute the relaunch command
    pub relaunch_in: Option<PathBuf>,
    /// display name of the item
    pub display_name: String,
    /// This intention is to prevent pinned state change, when this is neccesary
    #[serde(skip_deserializing)]
    pub pin_disabled: bool,
}

#[cfg(test)]
mod compat_tests {
    use super::*;

    /// Old user state files contain `type: "Plugin"` items (start menu / show
    /// desktop / trash bin built-ins). They must deserialize and be dropped
    /// without failing the whole file.
    #[test]
    fn old_state_with_plugin_items_deserializes_and_drops_them() {
        let raw = r#"
isReorderDisabled: false
left:
  - type: Plugin
    id: 11111111-2222-3333-4444-555555555555
    plugin: "@dockbar/dock-start-menu"
center:
  - type: AppOrFile
    id: 21111111-2222-3333-4444-555555555555
    display_name: Explorer
    path: C:\Windows\explorer.exe
    pinned: true
    prevent_pinning: false
  - type: Separator
    id: 31111111-2222-3333-4444-555555555555
right:
  - type: Media
    id: 41111111-2222-3333-4444-555555555555
"#;
        let mut items: DockItems = serde_yaml::from_str(raw).expect("old state must deserialize");
        items.sanitize();

        assert!(items.left.is_empty(), "plugin item must be dropped");
        assert_eq!(items.center.len(), 2, "app and separator must survive");
        assert!(matches!(items.center[0], DockItem::AppOrFile(_)));
        assert!(matches!(items.center[1], DockItem::Separator { .. }));
        assert_eq!(items.right.len(), 1);
        assert!(matches!(items.right[0], DockItem::Media { .. }));
    }

    /// Same file shape but with camelCase ids, as written by this app's
    /// serde config on versions that stored `type: "Plugin"` items.
    #[test]
    fn deprecated_builtin_variants_are_dropped() {
        let raw = r#"
isReorderDisabled: false
left:
  - type: StartMenu
    id: 11111111-2222-3333-4444-555555555555
  - type: ShowDesktop
    id: 12111111-2222-3333-4444-555555555555
right:
  - type: TrashBin
    id: 13111111-2222-3333-4444-555555555555
"#;
        let mut items: DockItems =
            serde_yaml::from_str(raw).expect("deprecated variants must parse");
        items.sanitize();
        assert!(items.left.is_empty() && items.right.is_empty());
    }
}
