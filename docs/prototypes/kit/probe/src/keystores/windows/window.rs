//! Which window owns the OS PIN dialog.

use std::ffi::c_void;

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Console::GetConsoleWindow;

/// The window PIN dialogs should belong to: the caller's, or else this
/// process's console. A dialog without an owner may open behind the
/// browser, where users never find it.
///
/// Chromium starts native hosts with a hidden console, so for a host the
/// console is an invisible window: it still ties the dialog to this process,
/// but whether the dialog then comes to the front is only known from real
/// tokens (docs/prototypes/1-windows.md, step 7).
pub fn owner(requested: Option<isize>) -> Option<HWND> {
    let window = match requested {
        Some(handle) => HWND(handle as *mut c_void),
        // SAFETY: takes no arguments; returns null when the process has no
        // console.
        None => unsafe { GetConsoleWindow() },
    };
    (!window.is_invalid()).then_some(window)
}
