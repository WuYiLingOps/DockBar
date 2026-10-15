use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::Widget;

/// Declaration for a system-level shortcut (not attached to any specific widget definition).
/// Hardcoded in Rust; exposed to the frontend via the `StateGetSystemShortcuts` command.
/// The user can override keys via `DockBarShortcutsSettings.shortcuts`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(export))]
pub struct SystemShortcutDeclaration {
    pub id: String,
    pub command: Vec<String>,
    pub label: String,
    pub default_keys: Vec<String>,
    /// If true, user cannot change the keys for this shortcut.
    pub readonly: bool,
}

/// Minimal struct sent to the service after the background has resolved all shortcut overrides.
/// Contains only what the service needs to register hotkeys — no widget info, no enums.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(export))]
pub struct ResolvedShortcut {
    pub command: Vec<String>,
    pub keys: Vec<String>,
}

/// User-facing shortcut settings.
/// - `enabled`: global on/off toggle.
/// - `shortcuts`: key overrides for **system-level** shortcut declarations (`id -> keys`).
///   Widget shortcut overrides live inside each widget's `$shortcuts` in `SettingsByWidget`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(export))]
pub struct DockBarShortcutsSettings {
    pub enabled: bool,
    pub shortcuts: HashMap<String, Vec<String>>,
}

impl Default for DockBarShortcutsSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            shortcuts: HashMap::new(),
        }
    }
}

// =====================================================================
// Helpers
// =====================================================================

macro_rules! cmd {
    ($($arg:expr),+ $(,)?) => {
        vec![$($arg.to_string()),+]
    };
}

macro_rules! decl {
    ($id:expr, $label:expr, $command:expr, $default_keys:expr, readonly) => {
        SystemShortcutDeclaration {
            id: $id.to_string(),
            label: $label.to_string(),
            command: $command,
            default_keys: $default_keys,
            readonly: true,
        }
    };
    ($id:expr, $label:expr, $command:expr, $default_keys:expr) => {
        SystemShortcutDeclaration {
            id: $id.to_string(),
            label: $label.to_string(),
            command: $command,
            default_keys: $default_keys,
            readonly: false,
        }
    };
}

// =====================================================================
// System shortcut declarations (hardcoded defaults)
// =====================================================================

/// Returns all hardcoded system-level shortcut declarations.
/// These are shortcuts that are not tied to any single widget (misc).
pub fn system_shortcut_declarations() -> Vec<SystemShortcutDeclaration> {
    // ---- Misc (readonly) ----
    let decls = vec![decl!(
        "shortcuts-pause-toggle",
        "t:shortcuts.labels.pause_toggle",
        cmd!["toggle-shortcuts-pause"],
        cmd!["Ctrl", "Win", "Alt", "P"]
    )];

    decls
}

// =====================================================================
// Resolution
// =====================================================================

use super::Settings;

/// Resolves all shortcut declarations (widget-declared + system-hardcoded) against
/// user-configured overrides, respecting widget enabled state.
///
/// Returns a flat list of `ResolvedShortcut` ready to be sent to the service, and a
/// boolean that is `true` when at least two entries share the same key combination.
/// Returns an empty `Vec` (and `false`) if `settings.shortcuts.enabled` is false.
pub fn resolve_shortcuts(
    settings: &Settings,
    widgets: &[&Widget],
) -> (Vec<ResolvedShortcut>, bool) {
    if !settings.shortcuts.enabled {
        return (vec![], false);
    }

    let mut resolved = Vec::new();

    // 1. Widget-declared shortcuts
    for widget in widgets {
        if !settings.is_widget_enabled(&widget.id) {
            continue;
        }

        let overrides = settings.by_widget.get_shortcut_overrides(&widget.id);
        for decl in &widget.shortcuts {
            let keys = if decl.readonly {
                decl.default_keys.clone()
            } else {
                overrides
                    .get(&decl.id)
                    .cloned()
                    .unwrap_or_else(|| decl.default_keys.clone())
            };
            if keys.is_empty() {
                continue;
            }
            resolved.push(ResolvedShortcut {
                command: decl.command.clone(),
                keys,
            });
        }
    }

    // 2. System-level declarations
    let system_overrides = &settings.shortcuts.shortcuts;
    for decl in system_shortcut_declarations() {
        let keys = system_overrides
            .get(&decl.id)
            .cloned()
            .unwrap_or_else(|| decl.default_keys.clone());

        if keys.is_empty() {
            continue;
        }

        resolved.push(ResolvedShortcut {
            command: decl.command.clone(),
            keys,
        });
    }

    // Detect duplicate key combinations across all resolved entries.
    let has_conflicts = {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        resolved.iter().any(|r| {
            let normalized = r
                .keys
                .iter()
                .map(|k| k.to_lowercase())
                .collect::<Vec<_>>()
                .join("+");
            !seen.insert(normalized)
        })
    };

    (resolved, has_conflicts)
}
