mod apps_config;
mod dock_items;
pub mod performance;
mod settings;

pub use dock_items::DOCK_ITEMS_MANAGER;

use arc_swap::ArcSwap;
use dockbar_core::{
    resource::ResourceKind,
    state::{AppsConfigurationList, Settings},
};
use notify_debouncer_full::{
    DebounceEventResult, DebouncedEvent, Debouncer, FileIdMap, new_debouncer,
    notify::{ReadDirectoryChangesWatcher, RecursiveMode},
};
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, LazyLock, OnceLock},
    time::Duration,
};

use crate::{
    error::{Result, ResultLogExt},
    resources::{RESOURCES, ResourceManager},
    utils::constants::DOCKBAR_COMMON,
};

/// Bundled per-app configs (src/static/apps_templates/*.yml), used at runtime
/// by window filtering (`Window::get_app_config`).
pub static BUNDLED_SETTINGS_BY_APP: LazyLock<Arc<AppsConfigurationList>> = LazyLock::new(|| {
    Arc::new(match AppSettings::load_bundled_settings_by_app() {
        Ok(list) => list,
        Err(e) => {
            log::error!("Failed to load bundled settings by app: {e}");
            AppsConfigurationList::default()
        }
    })
});

pub static FULL_STATE: LazyLock<ArcSwap<AppSettings>> = LazyLock::new(|| {
    ArcSwap::from_pointee({
        log::trace!("Creating new State Manager");
        AppSettings::new()
    })
});

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub settings: Settings,
}

impl AppSettings {
    fn new() -> Self {
        Self {
            settings: AppSettings::read_settings(),
        }
    }

    /// Shorthand of `FullState::clone` on Arc reference
    ///
    /// Intended to be used with `ArcSwap::rcu` to mofify the state
    pub fn cloned(&self) -> Self {
        self.clone()
    }

    /// Run RESOURCES-dependent initialization steps.
    /// Call this after both FULL_STATE and RESOURCES are initialized so the two can
    /// be loaded in parallel.
    pub fn complete_initialization(&mut self, _resources: &ResourceManager) {}
}

static USER_RESOURCES_WATCHER: OnceLock<Debouncer<ReadDirectoryChangesWatcher, FileIdMap>> =
    OnceLock::new();

pub fn initialize_user_resources_watcher() -> Result<()> {
    log::trace!("Starting DockBar Files Watcher");
    let mut debouncer = new_debouncer(
        Duration::from_millis(100),
        None,
        |result: DebounceEventResult| match result {
            Ok(events) => {
                let changed = join_and_filter_debounced_changes(events);
                crate::get_tokio_handle()
                    .spawn(async move { process_changes(&changed).await.log_error() });
            }
            Err(errors) => errors
                .iter()
                .for_each(|e| log::error!("File Watcher Error: {e:?}")),
        },
    )?;

    debouncer.watch(DOCKBAR_COMMON.app_data_dir(), RecursiveMode::Recursive)?;
    USER_RESOURCES_WATCHER.set(debouncer).ok();
    Ok(())
}

fn join_and_filter_debounced_changes(events: Vec<DebouncedEvent>) -> HashSet<PathBuf> {
    let mut result = HashSet::new();
    for event in events {
        for path in event.event.paths {
            if !path.is_dir() {
                result.insert(path);
            }
        }
    }
    result
}

async fn process_changes(changed: &HashSet<PathBuf>) -> Result<()> {
    let mut widgets_changed = false;
    let mut icons_changed = false;

    // Single iteration over the changed paths
    for path in changed {
        if !icons_changed && path.starts_with(DOCKBAR_COMMON.user_icons_path()) {
            icons_changed = true;
        };

        if !widgets_changed
            && (path.starts_with(DOCKBAR_COMMON.user_widgets_path())
                || path.starts_with(DOCKBAR_COMMON.bundled_widgets_path()))
        {
            widgets_changed = true;
        }
    }

    if widgets_changed {
        log::info!("Widgets changed");
        RESOURCES
            .load_all_of_type(ResourceKind::Widget, false)
            .await?;
        RESOURCES.emit_widgets()?;
    }

    if icons_changed {
        log::info!("Icon Packs changed");
        RESOURCES
            .load_all_of_type(ResourceKind::IconPack, false)
            .await?;
        RESOURCES.emit_icon_packs();
    }

    Ok(())
}
