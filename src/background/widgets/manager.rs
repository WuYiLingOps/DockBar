use std::sync::{
    Arc, LazyLock,
    atomic::{AtomicBool, Ordering},
};

use dockbar_core::{
    resource::WidgetId,
    state::{WidgetLoader, WidgetStatus},
};

use crate::{
    error::{Result, ResultLogExt},
    modules::monitors::MonitorManager,
    resources::RESOURCES,
    state::application::FULL_STATE,
    utils::lock_free::SyncHashMap,
    widgets::{WidgetWebviewLabel, loader::WidgetDeployment},
};

pub static WIDGET_MANAGER: LazyLock<WidgetManager> = LazyLock::new(WidgetManager::create);
pub static GAME_MODE_ACTIVE: AtomicBool = AtomicBool::new(false);

pub struct WidgetManager {
    /// group of widgets instances by widget resource id.
    ///
    /// Deployments are shared as `Arc`s so the map lock is never held while a
    /// webview is created or destroyed: those operations dispatch to the
    /// event-loop thread and can take seconds (specially in debug builds),
    /// which used to stall every concurrent trigger/liveness/destroy handler
    /// until the `TracedMutex` timeout.
    pub deployments: SyncHashMap<WidgetId, Arc<WidgetDeployment>>,
}

impl WidgetManager {
    fn create() -> Self {
        let sub_id = MonitorManager::subscribe(|_event| {
            WIDGET_MANAGER.reconcile().log_error();
        });
        MonitorManager::set_event_handler_priority(&sub_id, 1);
        Self {
            deployments: SyncHashMap::new(),
        }
    }

    /// Snapshot of the deployment for `id`, releasing the map lock immediately.
    /// Heavy work must run on the returned `Arc`, not inside a `deployments.get`.
    pub fn deployment(&self, id: &WidgetId) -> Option<Arc<WidgetDeployment>> {
        self.deployments.get(id, |deploy| deploy.clone())
    }

    /// Whether `deploy` is still the deployment registered for `id` (it may have
    /// been replaced or removed while we were working on the snapshot).
    fn is_current(&self, id: &WidgetId, deploy: &Arc<WidgetDeployment>) -> bool {
        self.deployments
            .get(id, |current| Arc::ptr_eq(current, deploy))
            .unwrap_or(false)
    }

    pub fn is_ready(&self, label: &WidgetWebviewLabel) -> bool {
        self.deployment(&label.widget_id)
            .is_some_and(|deploy| deploy.pods.any(|(key, pod)| key == label && pod.is_ready()))
    }

    pub fn set_status(&self, label: &WidgetWebviewLabel, status: WidgetStatus) {
        if let Some(deploy) = self.deployment(&label.widget_id) {
            deploy.pods.get(label, |instance| {
                instance.set_status(status);
            });
        }
    }

    pub fn suspend_all(&self) {
        GAME_MODE_ACTIVE.store(true, Ordering::Release);
        for (_, deploy) in self
            .deployments
            .map(|(key, deploy)| (key.clone(), deploy.clone()))
        {
            deploy.pods.clear();
        }
    }

    pub fn resume_all(&self) -> Result<()> {
        GAME_MODE_ACTIVE.store(false, Ordering::Release);
        self.reconcile()
    }

    /// Drops every pod of every deployment. `WidgetWebview`'s `Drop` destroys
    /// the native windows, giving the caller a clean slate to `reconcile()`
    /// against (used by the WebView2 environment recovery).
    pub fn close_all_pods(&self) {
        self.deployments.for_each(|(_, deploy)| {
            deploy.pods.clear();
        });
    }

    /// Destroys the pod for `label` (if any) and rebuilds a fresh one, starting
    /// it unless the widget is lazy or game mode is active. This is the recovery
    /// path for wedged windows (used by the liveness give-up and the
    /// `WindowEvent::Destroyed` handler).
    pub fn respawn_pod(&self, label: &WidgetWebviewLabel) {
        let Some(deploy) = self.deployment(&label.widget_id) else {
            return;
        };

        deploy.kill_pod(label);
        deploy.reconcile();
        // suspend_all() destroys every widget when game mode starts; respawning
        // here would bring them back on top of the game. resume_all() starts
        // them again once game mode ends.
        if !deploy.definition.lazy && !GAME_MODE_ACTIVE.load(Ordering::Acquire) {
            deploy.start_all_webviews();
        }
    }

    pub fn reconcile(&self) -> Result<()> {
        // remove deleted resources
        self.deployments
            .retain(|(key, _)| RESOURCES.widgets.contains_sync(key));

        let mut filtered = Vec::new();
        RESOURCES.widgets.iter_sync(|k, w| {
            if w.loader != WidgetLoader::Legacy {
                filtered.push((k.clone(), w.clone()));
            }
            true
        });

        let state = FULL_STATE.load();
        for (id, widget) in filtered {
            if !state.is_widget_enabled(&id) {
                self.deployments.remove(&id);
                continue;
            }

            if !self.deployments.contains_key(&id) {
                self.deployments
                    .upsert(id.clone(), Arc::new(WidgetDeployment::new(widget)));
            }
        }

        // lazy creation of webviews to reduce startup time
        std::thread::spawn(|| {
            fn reconcile_if_current(manager: &WidgetManager, id: &WidgetId) {
                // Re-check currency before acting: the deployment could have been
                // removed (widget disabled) or replaced while we were queued.
                let Some(deploy) = manager.deployment(id) else {
                    return;
                };
                if !manager.is_current(id, &deploy) {
                    return;
                }
                deploy.reconcile();
                if !deploy.definition.lazy && !GAME_MODE_ACTIVE.load(Ordering::Acquire) {
                    deploy.start_all_webviews();
                }
            }

            // More visual widgets load first
            for priority in [
                WidgetId::known_wall(),
                WidgetId::known_toolbar(),
                WidgetId::known_dock(),
            ] {
                reconcile_if_current(&WIDGET_MANAGER, &priority);
            }

            // All other widgets
            for (id, _) in WIDGET_MANAGER
                .deployments
                .map(|(key, deploy)| (key.clone(), deploy.clone()))
            {
                reconcile_if_current(&WIDGET_MANAGER, &id);
            }
        });

        Ok(())
    }
}
