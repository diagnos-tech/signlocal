//! Bringing the confirmation window to the front (`docs/ux.md` §4.1).
//!
//! The host is a child of the browser (or of the program that ran
//! `websign connect`), which has the focus, so the OS normally lets it take
//! the foreground. When it refuses (Windows gives foreground only to
//! processes the person is interacting with), the window stays topmost and
//! flashes, and nothing more is attempted: no simulated input, no attaching
//! to another thread's input queue. Arming never relies on this function:
//! it starts from the window's real focus events (ui-model `Focus(true)`),
//! so a window that did not get focus stays unarmed until the person
//! clicks it.

/// Raises the window with native handle `handle` and reports whether it has
/// the keyboard focus now.
///
/// `handle` is the `HWND` on Windows, the `NSView` pointer on macOS (as
/// `raw-window-handle` gives it) and the X11 window ID on Linux (ignored:
/// see the Linux notes below). Call it on the UI thread. `false` does not
/// mean failure for good: the OS may grant focus a moment later (macOS
/// activates asynchronously), which the window then sees as a focus event.
///
/// On Linux, X11 window managers and Wayland compositors decide stacking
/// and focus themselves (focus-stealing prevention); the window asks for
/// attention through winit (`ViewportCommand::Focus`,
/// `RequestUserAttention`), so this returns `false`.
pub fn bring_to_front(handle: isize) -> bool {
    super::os::focus::bring_to_front(handle)
}
