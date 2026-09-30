//! Linux focus: the window manager or compositor decides (see
//! [`crate::platform::focus`]).

/// Always `false`: X11 window managers and Wayland compositors apply their
/// own focus-stealing prevention, and the window already asks through winit.
pub fn bring_to_front(handle: isize) -> bool {
    let _ = handle;
    false
}
