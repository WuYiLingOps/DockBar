use std::sync::atomic::{AtomicBool, Ordering};

use dockbar_core::{handlers::DockBarEvent, state::shortcuts::ResolvedShortcut};

use crate::app::emit_to_webviews;

pub static SHORTCUTS_PAUSED: AtomicBool = AtomicBool::new(false);

/// Stores the last resolved shortcut list. Global hotkeys are no longer registered
/// anywhere (the privileged service was removed), this only keeps the resolved set
/// available for debugging purposes.
pub fn set_resolved_shortcuts(resolved: Vec<ResolvedShortcut>) {
    let paused = SHORTCUTS_PAUSED.load(Ordering::Acquire);
    emit_to_webviews(
        DockBarEvent::ShortcutsPaused,
        paused && !resolved.is_empty(),
    );
}

pub fn toggle_pause() {
    let was_paused = SHORTCUTS_PAUSED.fetch_xor(true, Ordering::SeqCst);
    let now_paused = !was_paused;
    emit_to_webviews(DockBarEvent::ShortcutsPaused, now_paused);
}
