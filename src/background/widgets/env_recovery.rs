//! In-process recovery from a dead WebView2 environment.
//!
//! When the Evergreen WebView2 Runtime updates itself while DockBar is running,
//! the already-running browser process keeps serving the old version, while
//! every NEW webview creation fails with `RPC_E_DISCONNECTED` (0x80010108).
//! That failure is environment-wide, not pod-wide: per-pod creation retries can
//! never succeed. This module detects that signature and recovers in place —
//! stale browser processes are terminated and every widget webview is rebuilt —
//! so the user never has to restart the app.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use sysinfo::{ProcessesToUpdate, System};

use crate::{
    error::ResultLogExt,
    utils::spawn_named_thread,
    widgets::manager::{GAME_MODE_ACTIVE, WIDGET_MANAGER},
};

/// `RPC_E_DISCONNECTED`: the COM connection to the WebView2 browser process is gone.
const RPC_E_DISCONNECTED: &str = "0x80010108";

/// Two creation failures within this window confirm that the environment is
/// dead. A lone failure may be a transient COM hiccup, and a false positive
/// costs every widget window a rebuild.
const PAIRING_WINDOW: Duration = Duration::from_secs(15);
/// Minimum delay between two recovery runs, so a recovery that cannot fix the
/// environment (e.g. a corrupted user-data folder) degrades into a slow retry
/// instead of a rebuild loop. The next trigger on a wedged pod re-arms the
/// detection, so user interaction keeps retrying recovery after the cooldown.
const RECOVERY_COOLDOWN: Duration = Duration::from_secs(60);
/// Grace period after killing the browser processes, letting the OS release
/// their user-data folder locks before any new webview launches a replacement.
const BROWSER_EXIT_GRACE: Duration = Duration::from_millis(300);

const BROWSER_PROCESS_NAME: &str = "msedgewebview2.exe";
/// The webview user-data dirs all live under the app cache dir
/// (`%LOCALAPPDATA%\com.dockbar.app\gpu`, `...\no-gpu`), so the app folder name
/// matches only OUR browser processes — other apps' WebView2 processes never
/// match and are never touched.
const USER_DATA_DIR_MARKER: &str = "com.dockbar.app";

static LAST_DISCONNECT: Mutex<Option<Instant>> = Mutex::new(None);
static LAST_RECOVERY: Mutex<Option<Instant>> = Mutex::new(None);
static RECOVERY_RUNNING: AtomicBool = AtomicBool::new(false);

/// Feed a failed `WidgetWebview::create` error here. Recovery starts once two
/// `RPC_E_DISCONNECTED` failures arrive close together; the creation retry
/// backoff guarantees the confirming second failure ~1s after the first.
pub fn notify_webview_creation_failure(error: impl std::fmt::Display) {
    // `AppError` keeps errors as pre-rendered strings only, so the HRESULT has
    // to be matched on the formatted message: the hex code is locale-independent,
    // unlike the message text.
    if !is_env_disconnected(&error.to_string()) {
        return;
    }

    let now = Instant::now();
    let confirmed = {
        let mut last = LAST_DISCONNECT.lock();
        let paired = matches!(*last, Some(at) if now.duration_since(at) <= PAIRING_WINDOW);
        *last = if paired { None } else { Some(now) };
        paired
    };

    if !confirmed {
        log::warn!(
            "WebView2 environment looks disconnected ({RPC_E_DISCONNECTED}); waiting for a confirming failure before recovering."
        );
        return;
    }

    log::warn!(
        "WebView2 environment is confirmed disconnected (usually the Evergreen runtime auto-updated while DockBar was running); starting in-place recovery."
    );
    spawn_recovery();
}

fn spawn_recovery() {
    if RECOVERY_RUNNING
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        log::info!("WebView2 environment recovery already running; skipping.");
        return;
    }

    {
        let mut last = LAST_RECOVERY.lock();
        if last.is_some_and(|at| at.elapsed() < RECOVERY_COOLDOWN) {
            log::warn!(
                "WebView2 environment recovery is on cooldown; the next failed creation will retry it."
            );
            RECOVERY_RUNNING.store(false, Ordering::Release);
            return;
        }
        *last = Some(Instant::now());
    }

    spawn_named_thread("WebView2EnvRecovery", || {
        execute_recovery();
        RECOVERY_RUNNING.store(false, Ordering::Release);
    });
}

fn execute_recovery() {
    let started = Instant::now();

    if GAME_MODE_ACTIVE.load(Ordering::Acquire) {
        // suspend_all() already removed every pod; resume_all() rebuilds them
        // when the game ends, so only the browser processes need cleaning up.
        let killed = kill_stale_webview_browsers();
        log::info!(
            "Game mode is active: killed {killed} stale browser process(es); widgets will be rebuilt when the game ends."
        );
        return;
    }

    // 1) Kill the stale browser processes first: every webview created after
    //    this point launches a fresh browser with the updated runtime, so the
    //    Destroyed-event respawn race below already rebuilds against a healthy
    //    environment.
    let killed = kill_stale_webview_browsers();
    log::info!("Killed {killed} stale {BROWSER_PROCESS_NAME} browser process(es).");
    std::thread::sleep(BROWSER_EXIT_GRACE);

    // 2) Drop every pod. WidgetWebview's Drop destroys the native windows, and
    //    their Destroyed events respawn their pods (see WidgetPod::run), racing
    //    with the reconcile below — both paths are idempotent.
    WIDGET_MANAGER.close_all_pods();

    // 3) Recreate pods from current settings and start the non-lazy widgets
    //    (dock, wallpaper...). Lazy widgets (settings, context menu) start on
    //    their next trigger.
    WIDGET_MANAGER.reconcile().log_error();

    log::info!(
        "WebView2 environment recovery finished in {:?}.",
        started.elapsed()
    );
}

/// Terminates every `msedgewebview2.exe` bound to DockBar's user-data folders.
/// Returns how many processes were killed.
fn kill_stale_webview_browsers() -> usize {
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);

    let mut killed = 0;
    for process in system.processes().values() {
        if !process.name().eq_ignore_ascii_case(BROWSER_PROCESS_NAME) {
            continue;
        }
        let is_ours = process.cmd().iter().any(|arg| {
            arg.to_string_lossy()
                .to_ascii_lowercase()
                .contains(USER_DATA_DIR_MARKER)
        });
        if is_ours && process.kill() {
            killed += 1;
        }
    }
    killed
}

fn is_env_disconnected(rendered_error: &str) -> bool {
    rendered_error.contains(RPC_E_DISCONNECTED)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_disconnected_environment_in_tauri_error_chain() {
        let rendered = "Tauri(Runtime(CreateWebview(WebView2Error(WindowsError(Error { code: HRESULT(0x80010108), message: \"The object invoked has disconnected from its clients.\" })))))";
        assert!(is_env_disconnected(rendered));
    }

    #[test]
    fn ignores_unrelated_webview_creation_errors() {
        let invalid_handle = "Tauri(Runtime(CreateWebview(WebView2Error(WindowsError(Error { code: HRESULT(0x80070578), message: \"Invalid window handle.\" })))))";
        assert!(!is_env_disconnected(invalid_handle));
        assert!(!is_env_disconnected("Widget not found"));
    }
}
