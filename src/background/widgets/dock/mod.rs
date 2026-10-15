pub mod cli;
pub mod handler;
pub mod hook;
pub mod native_taskbar;

use crate::{error::Result, state::application::FULL_STATE, windows_api::monitor::Monitor};

/// Class names of the shell's own taskbar windows; kept for native-taskbar
/// interop work.
#[allow(dead_code)]
pub static TASKBAR_CLASS: [&str; 2] = ["Shell_TrayWnd", "Shell_SecondaryTrayWnd"];

pub struct DockBar {}

impl DockBar {
    #[allow(dead_code)]
    pub fn get_dock_size_on_monitor(monitor: &Monitor) -> Result<i32> {
        let state = FULL_STATE.load();
        let settings: &dockbar_core::state::DockBarSettings = &state.settings.by_widget.dock;
        let total_size = (settings.total_size() as f64 * monitor.scale_factor()?) as i32;
        Ok(total_size)
    }
}
