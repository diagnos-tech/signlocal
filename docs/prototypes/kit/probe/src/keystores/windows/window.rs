//! Which window owns the OS PIN dialog.

use std::ffi::c_void;

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Console::GetConsoleWindow;

/// The window PIN dialogs should belong to: the caller's, or else this
/// process's console. A dialog without an owner may open behind the
/// browser, where users never find it.
pub fn owner(requested: Option<isize>) -> Option<HWND> {
    let window = match requested {
        Some(handle) => HWND(handle as *mut c_void),
        // SAFETY: no arguments; returns null when there is no console (a
        // browser starts native hosts without one).
        None => unsafe { GetConsoleWindow() },
    };
    (!window.is_invalid()).then_some(window)
}
