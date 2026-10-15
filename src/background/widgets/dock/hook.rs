use windows::Win32::Foundation::HWND;

use crate::error::Result;

use super::DockBar;

impl DockBar {
    // move this to independent function as this should work independently if dock is enabled or not
    pub fn process_raw_win_event(event: u32, _origin_hwnd: HWND) -> Result<()> {
        // Upstream Seelen UI hid native shell flyouts here (notification center via
        // ShellExperienceHost, volume/brightness via empty-titled
        // XamlExplorerHostIslandWindow) to make room for its own replacement UI.
        // This trimmed fork ships neither replacement, so hiding those windows only
        // broke system features: the Win+P project flyout lives inside an
        // XamlExplorerHostIslandWindow and died instantly (transparent window
        // appearing and vanishing), and notifications were swallowed. All the
        // hiding logic is therefore removed.
        let _ = event;
        Ok(())
    }
}
