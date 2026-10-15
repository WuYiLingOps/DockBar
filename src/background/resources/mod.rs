pub mod cli;
pub mod commands;
mod emitters;
mod system_icon_pack;
pub mod user_icon_pack;

use std::{
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

use dockbar_core::{
    resource::{DockBarResource, IconPackId, ResourceId, ResourceKind, WidgetId},
    state::{IconPack, Widget},
};

use crate::{
    error::{Result, ResultLogExt},
    state::application::FULL_STATE,
    utils::{constants::DOCKBAR_COMMON, lock_free::TracedMutex},
};

pub static RESOURCES: LazyLock<Arc<ResourceManager>> =
    LazyLock::new(|| Arc::new(ResourceManager::default()));

#[derive(Default)]
pub struct ResourceManager {
    pub widgets: scc::HashMap<WidgetId, Arc<Widget>>,
    /// this list doesn't include the system icon pack, this is managed by the system_icon_pack field.
    pub icon_packs: scc::HashMap<IconPackId, Arc<IconPack>>,
    /// current system icon pack
    pub system_icon_pack: Arc<TracedMutex<Option<IconPack>>>,
    /// list of manual loaded resources
    pub manual: scc::HashSet<PathBuf>,
}

impl ResourceManager {
    #[rustfmt::skip]
    pub async fn initialize(&self) {
        // measuring here doesn't make sense because all are loaded in parallel
        // using all available cores/threads acting as single task, so all finish at the same time.
        tokio::try_join!(
            tokio::spawn(async move {
                RESOURCES.load_all_of_type(ResourceKind::Widget, true).await.log_error()
            }),
            tokio::spawn(async move {
                RESOURCES.load_all_of_type(ResourceKind::IconPack, true).await.log_error()
            }),
        ).log_error();
    }

    /// Returns the id of the resource that was loaded, if any (e.g. the system icon pack
    /// is loaded but doesn't produce an activatable resource id).
    pub async fn load(&self, kind: &ResourceKind, path: &Path) -> Result<Option<ResourceId>> {
        let id = match kind {
            ResourceKind::Widget => {
                let mut widget = Widget::load(path).await?;
                // flags the widget as bundled so the permissions system grants it
                // everything without prompting (the metadata file can't carry this:
                // `internal` is skipped on deserialization)
                widget.metadata.internal.bundled =
                    path.starts_with(DOCKBAR_COMMON.bundled_widgets_path());
                let id = (*widget.id).clone();
                self.widgets
                    .upsert_async(widget.id.clone(), Arc::new(widget))
                    .await;
                id
            }
            ResourceKind::IconPack => {
                let is_system = path == DOCKBAR_COMMON.system_icon_pack_path();
                if is_system {
                    // Check before awaiting — never hold MutexGuard across an await point
                    let needs_load = self.system_icon_pack.lock().is_none();
                    if needs_load {
                        let mut icon_pack = IconPack::load(path).await?;
                        icon_pack.metadata.internal.bundled = true;
                        // we only read the system icon pack once, after that it is entirely runtime managed
                        *self.system_icon_pack.lock() = Some(icon_pack);
                    }
                    return Ok(None);
                }

                let icon_pack = IconPack::load(path).await?;
                let id = (*icon_pack.id).clone();
                self.icon_packs
                    .upsert_async(icon_pack.id.clone(), Arc::new(icon_pack))
                    .await;
                id
            }
        };
        Ok(Some(id))
    }

    pub fn unload(&self, kind: &ResourceKind, path: &Path) {
        match kind {
            ResourceKind::Widget => {
                self.widgets
                    .retain_sync(|_, v| v.metadata.internal.path != path);
            }
            ResourceKind::IconPack => {
                self.icon_packs
                    .retain_sync(|_, v| v.metadata.internal.path != path);
            }
        }
    }

    pub fn unload_all(&self, kind: &ResourceKind) {
        match kind {
            ResourceKind::Widget => self
                .widgets
                .retain_sync(|_, v| self.manual.contains_sync(&v.metadata.internal.path)),
            ResourceKind::IconPack => self
                .icon_packs
                .retain_sync(|_, v| self.manual.contains_sync(&v.metadata.internal.path)),
        }
    }

    /// returns a flat list of paths to be loaded for this kind
    async fn get_entries_for_type(kind: &ResourceKind) -> Result<Vec<PathBuf>> {
        let dirs: Vec<PathBuf> = match kind {
            ResourceKind::Widget => vec![
                DOCKBAR_COMMON.bundled_widgets_path().to_path_buf(),
                DOCKBAR_COMMON.user_widgets_path().to_path_buf(),
            ],
            ResourceKind::IconPack => vec![DOCKBAR_COMMON.user_icons_path().to_path_buf()],
        };

        async fn read_dir_entries(dir: PathBuf) -> Result<Vec<PathBuf>> {
            let mut rd = match tokio::fs::read_dir(&dir).await {
                Ok(rd) => rd,
                // missing directories (e.g. an emptied user folder) are just empty
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
                Err(e) => return Err(e.into()),
            };
            let mut entries = Vec::new();
            while let Some(entry) = rd.next_entry().await? {
                entries.push(entry.path());
            }
            Ok(entries)
        }

        let paths = futures::future::try_join_all(dirs.into_iter().map(read_dir_entries))
            .await?
            .into_iter()
            .flatten()
            .collect();
        Ok(paths)
    }

    /// `on_mount` should be true only when called during app startup. It only affects
    /// `ResourceKind::IconPack`: when true, bundled default icons overwrite existing files
    /// on disk (so an updated app version replaces stale cached icons); otherwise they're
    /// only written if missing, avoiding redundant writes on the frequent icon-pack-changed
    /// reload.
    pub async fn load_all_of_type(&self, kind: ResourceKind, on_mount: bool) -> Result<()> {
        log::trace!("Loading {kind:?}s");

        let paths = Self::get_entries_for_type(&kind).await?;
        self.unload_all(&kind);

        let handles = paths.into_iter().map(|path| {
            tokio::spawn(async move {
                if let Err(e) = RESOURCES.load(&kind, &path).await {
                    log::error!("Failed to load {kind:?}, error: {e}");
                }
            })
        });
        futures::future::join_all(handles).await;

        if kind == ResourceKind::IconPack {
            // try load system icon pack
            let _ = self
                .load(&kind, DOCKBAR_COMMON.system_icon_pack_path())
                .await;
            // creates the system icon pack if not loaded
            std::thread::spawn(move || RESOURCES.ensure_system_icon_pack(on_mount).log_error());
        }
        Ok(())
    }

    /// Activates a loaded resource the same way the "Enable" button does after installing
    /// a resource (via `dockbar.uri:` or a dropped resource file).
    pub fn enable_resource(&self, kind: ResourceKind, id: ResourceId) {
        std::thread::spawn(move || {
            FULL_STATE.rcu(move |state| {
                let mut state = state.cloned();
                match kind {
                    ResourceKind::IconPack => {
                        state
                            .settings
                            .active_icon_packs
                            .push(IconPackId::from(id.clone()));
                    }
                    ResourceKind::Widget => {
                        state
                            .settings
                            .set_widget_enabled(&WidgetId::from(id.clone()), true);
                    }
                }
                state
            });

            FULL_STATE.load().write_settings().log_error();
        });
    }
}
