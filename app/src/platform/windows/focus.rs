//! Windows focus: `SetForegroundWindow`, which Windows grants to a process
//! the foreground process started (the browser or calling program), else a
//! taskbar flash.
//!
//! Chrome's `--parent-window` is not used as the owner: it is `0` for
//! requests from an MV3 service worker, and a cross-process owner would tie
//! our window's stacking and minimizing to the browser's.

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    FLASHW_ALL, FLASHW_TIMERNOFG, FLASHWINFO, FlashWindowEx, GetForegroundWindow, HWND_TOPMOST,
    IsIconic, IsWindow, SW_RESTORE, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetForegroundWindow,
    SetWindowPos, ShowWindow,
};

pub fn bring_to_front(handle: isize) -> bool {
    let window = HWND(handle as *mut core::ffi::c_void);
    // SAFETY: `IsWindow` accepts any value and only reads.
    if handle == 0 || !unsafe { IsWindow(Some(window)) }.as_bool() {
        return false;
    }
    // SAFETY: `window` is a live window handle (checked above); these calls
    // change only its state and fail harmlessly if it closed meanwhile.
    unsafe {
        if IsIconic(window).as_bool() {
            let _ = ShowWindow(window, SW_RESTORE);
        }
        let _ = SetWindowPos(
            window,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
        let _ = SetForegroundWindow(window);
    }
    // SAFETY: no arguments; only reads.
    if unsafe { GetForegroundWindow() } == window {
        return true;
    }
    flash(window);
    false
}

/// Flashes the taskbar button until the window comes to the foreground.
fn flash(window: HWND) {
    let info = FLASHWINFO {
        cbSize: size_of::<FLASHWINFO>() as u32,
        hwnd: window,
        dwFlags: FLASHW_ALL | FLASHW_TIMERNOFG,
        uCount: 0,
        dwTimeout: 0,
    };
    // SAFETY: `info` is fully initialized and outlives the call.
    unsafe {
        let _ = FlashWindowEx(&info);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_handles_are_refused() {
        assert!(!bring_to_front(0));
        assert!(!bring_to_front(0x7FFF_FFF0));
    }
}
