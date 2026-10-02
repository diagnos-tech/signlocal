//! macOS focus: activate the app, then make the window key.

use super::objc::{AutoreleasePool, Id, ObjcBool, class, on_main_thread, send0, send1};

/// `handle` is the window's `NSView`. Activation is asynchronous: `false`
/// may turn into focus a moment later, which the window sees as an event.
pub fn bring_to_front(handle: isize) -> bool {
    if handle == 0 || !on_main_thread() {
        return false;
    }
    let Some(application) = class(c"NSApplication") else {
        return false;
    };
    let _pool = AutoreleasePool::new();
    let view = handle as Id;
    // SAFETY: on the main thread; `view` is the live `NSView` of a window
    // this process shows (the `platform::focus` contract), and each method
    // below exists with the signature used.
    unsafe {
        let window: Id = send0(view, c"window");
        if window.is_null() {
            return false;
        }
        let app: Id = send0(application, c"sharedApplication");
        // Deprecated in macOS 14 for `activate`, which older versions lack;
        // both request activation, neither forces it.
        send1::<ObjcBool, ()>(app, c"activateIgnoringOtherApps:", 1);
        send1::<Id, ()>(window, c"makeKeyAndOrderFront:", std::ptr::null_mut());
        send0::<ObjcBool>(app, c"isActive") != 0 && send0::<ObjcBool>(window, c"isKeyWindow") != 0
    }
}
