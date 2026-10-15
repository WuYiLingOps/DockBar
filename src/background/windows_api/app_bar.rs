use dockbar_core::system_state::AppBarEdge;
use std::sync::LazyLock;
use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT},
    UI::Shell::{
        ABE_BOTTOM, ABE_LEFT, ABE_RIGHT, ABE_TOP, ABM_NEW, ABM_REMOVE, ABM_SETPOS, ABM_SETSTATE,
        ABS_ALWAYSONTOP, ABS_AUTOHIDE, APPBARDATA, SHAppBarMessage,
    },
};

use crate::{error::Result, utils::lock_free::TracedMutex};

static REGISTERED_BARS: LazyLock<TracedMutex<Vec<isize>>> =
    LazyLock::new(|| TracedMutex::new(Vec::new()));

/// Last rect+edge each bar was registered with, so repeated register/unregister
/// calls (the dock re-runs them on every position/settings recompute) become
/// no-ops instead of hammering the shell with ABM_SETPOS/ABM_REMOVE cycles that
/// force explorer to recompute the work area and disrupt flyout placement.
static REGISTERED_BAR_STATE: LazyLock<TracedMutex<Vec<(isize, RECT, u32)>>> =
    LazyLock::new(|| TracedMutex::new(Vec::new()));

fn rects_equal(a: &RECT, b: &RECT) -> bool {
    a.left == b.left && a.top == b.top && a.right == b.right && a.bottom == b.bottom
}

/// https://learn.microsoft.com/en-us/windows/win32/shell/abm-setstate#parameters
#[derive(Debug, Clone, Copy)]
pub enum AppBarDataState {
    BothOff = 0,
    AutoHide = ABS_AUTOHIDE as isize,
    AlwaysOnTop = ABS_ALWAYSONTOP as isize,
    BothOn = 3,
}

impl From<AppBarDataState> for LPARAM {
    fn from(val: AppBarDataState) -> Self {
        LPARAM(val as isize)
    }
}

impl From<u32> for AppBarDataState {
    fn from(state: u32) -> Self {
        match state {
            0 => AppBarDataState::BothOff,
            ABS_AUTOHIDE => AppBarDataState::AutoHide,
            ABS_ALWAYSONTOP => AppBarDataState::AlwaysOnTop,
            3 => AppBarDataState::BothOn,
            _ => unreachable!(),
        }
    }
}

pub struct AppBarData(pub APPBARDATA);

#[allow(dead_code)]
impl AppBarData {
    pub fn from_handle(hwnd: HWND) -> Self {
        Self(APPBARDATA {
            cbSize: std::mem::size_of::<APPBARDATA>() as u32,
            hWnd: hwnd,
            ..Default::default()
        })
    }

    pub fn set_state(&self, state: AppBarDataState) {
        let mut data = self.0;
        data.lParam = state.into();
        unsafe { SHAppBarMessage(ABM_SETSTATE, &mut data) };
    }

    pub fn set_edge(&mut self, edge: AppBarEdge) {
        self.0.uEdge = match edge {
            AppBarEdge::Left => ABE_LEFT,
            AppBarEdge::Top => ABE_TOP,
            AppBarEdge::Right => ABE_RIGHT,
            AppBarEdge::Bottom => ABE_BOTTOM,
        };
    }

    pub fn set_rect(&mut self, rect: RECT) {
        self.0.rc = rect;
    }

    pub fn register_as_new_bar(&mut self) -> Result<()> {
        let mut data = self.0;
        let addr = data.hWnd.0 as isize;
        let mut guard = REGISTERED_BARS.lock();

        if !guard.contains(&addr) {
            let ok = unsafe { SHAppBarMessage(ABM_NEW, &mut data) };
            if ok == 0 {
                return Err("Failed to register App Bar".into());
            }
            guard.push(addr);
        }

        // Skip the ABM_SETPOS when edge+rect didn't change; the shell recomputes
        // the work area on every SETPOS, and redundant calls destabilize system
        // flyouts (Win+P etc.) while they read the work area.
        let mut state = REGISTERED_BAR_STATE.lock();
        if let Some((_, last_rect, last_edge)) = state.iter().find(|(a, _, _)| *a == addr)
            && rects_equal(last_rect, &data.rc)
            && *last_edge == data.uEdge
        {
            return Ok(());
        }
        unsafe { SHAppBarMessage(ABM_SETPOS, &mut data) };
        if let Some(entry) = state.iter_mut().find(|(a, _, _)| *a == addr) {
            *entry = (addr, data.rc, data.uEdge);
        } else {
            state.push((addr, data.rc, data.uEdge));
        }
        Ok(())
    }

    pub fn unregister_bar(&mut self) -> Result<()> {
        let mut data = self.0;
        let addr = data.hWnd.0 as isize;
        let mut guard = REGISTERED_BARS.lock();

        // Not registered (anymore): nothing to remove, and an ABM_REMOVE for an
        // unknown hwnd is shell-undefined behavior we'd rather avoid.
        if !guard.contains(&addr) {
            REGISTERED_BAR_STATE.lock().retain(|(a, _, _)| *a != addr);
            return Ok(());
        }

        unsafe { SHAppBarMessage(ABM_REMOVE, &mut data) };
        guard.retain(|x| *x != addr);
        REGISTERED_BAR_STATE.lock().retain(|(a, _, _)| *a != addr);
        Ok(())
    }
}
