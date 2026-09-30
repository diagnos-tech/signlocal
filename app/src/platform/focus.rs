//! Bringing the confirmation window to the front (`docs/ux.md` §4.1).
//!
//! The host is a child of the browser, which has the focus, but Windows may
//! refuse foreground to a process that received no input; then the window
//! stays topmost and flashes (`FlashWindowEx`), and arming starts only when
//! the person clicks it.

/// Raises the window with native handle `handle`; `true` when it got focus.
pub fn bring_to_front(handle: isize) -> bool {
    let _ = handle;
    todo!("ux.md §4.1")
}
