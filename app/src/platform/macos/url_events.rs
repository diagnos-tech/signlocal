//! `websign:` URLs on macOS, which Launch Services delivers as a `kAEGetURL`
//! Apple Event, never as an argument: when it starts the app for a URL the
//! process gets no URL in `argv`, and a copy of the app that already has a
//! window (diagnostics, a confirmation window kept hidden) receives the
//! event instead of a new process starting.
//!
//! **When the handler is installed.** AppKit installs its own `kAEGetURL`
//! handler while it finishes launching and would replace one installed
//! earlier; the documented point to install ours is "will finish launching",
//! before AppKit dispatches the event that launched the app. That moment
//! belongs to winit's application delegate, which eframe owns, so instead of
//! being the delegate we observe `NSApplicationWillFinishLaunchingNotification`,
//! which AppKit posts at the same moment. The observer is registered in
//! `main` before any window exists; it costs nothing in a process that never
//! starts AppKit (a host that never shows a window, the command line).

use std::ffi::{CStr, c_char, c_void};
use std::sync::OnceLock;

use super::objc::{
    AutoreleasePool, Id, Method, Sel, class, define_class, selector, send0, send1, send4,
};

#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {
    static NSApplicationWillFinishLaunchingNotification: Id;
}

/// `kInternetEventClass` and `kAEGetURL`: both the four-char code `'GURL'`.
const GET_URL: u32 = u32::from_be_bytes(*b"GURL");
/// `keyDirectObject` (`'----'`): the URL inside the event.
const DIRECT_OBJECT: u32 = u32::from_be_bytes(*b"----");

/// Selectors of the receiver class, prefixed so they never collide with a
/// method AppKit or winit might add to `NSObject`.
const WILL_FINISH: &CStr = c"websignWillFinishLaunching:";
const GOT_URL: &CStr = c"websignHandleGetURLEvent:withReplyEvent:";

static HANDLER: OnceLock<fn(&str)> = OnceLock::new();

/// Registers the observer that installs the `kAEGetURL` handler. The first
/// call wins; later calls do nothing.
pub fn listen(handler: fn(&str)) {
    if HANDLER.set(handler).is_err() {
        return;
    }
    let _pool = AutoreleasePool::new();
    let Some(receiver) = new_receiver() else {
        log::warn!("url events: the receiver class could not be defined");
        return;
    };
    let Some(center_class) = class(c"NSNotificationCenter") else {
        return;
    };
    // SAFETY: `+defaultCenter` returns the process's center;
    // `-addObserver:selector:name:object:` takes an object, a selector, a
    // notification name and an optional sender (nil = any). The receiver is
    // never released (see `new_receiver`), as the center does not retain it.
    unsafe {
        let center: Id = send0(center_class, c"defaultCenter");
        if center.is_null() {
            return;
        }
        send4::<Id, Sel, Id, Id, ()>(
            center,
            c"addObserver:selector:name:object:",
            receiver,
            selector(WILL_FINISH),
            NSApplicationWillFinishLaunchingNotification,
            std::ptr::null_mut(),
        );
    }
}

/// An instance of our receiver class, alive until the process ends: neither
/// the notification center nor the Apple Event manager retains it.
fn new_receiver() -> Option<Id> {
    let methods = [
        Method {
            name: WILL_FINISH,
            imp: will_finish_launching as *const c_void,
            types: c"v@:@",
        },
        Method {
            name: GOT_URL,
            imp: handle_get_url as *const c_void,
            types: c"v@:@@",
        },
    ];
    // SAFETY: both implementations below take `(Id, Sel, …)` exactly as
    // their encodings declare; `listen` runs this once (`HANDLER`).
    let receiver_class = unsafe { define_class(c"SignLocalURLEventReceiver", &methods) }?;
    // SAFETY: `+alloc`/`-init` of an `NSObject` subclass return an object.
    let receiver: Id = unsafe { send0(send0::<Id>(receiver_class, c"alloc"), c"init") };
    (!receiver.is_null()).then_some(receiver)
}

/// "Will finish launching": the moment to install Apple Event handlers.
extern "C" fn will_finish_launching(this: Id, _: Sel, _: Id) {
    let Some(manager_class) = class(c"NSAppleEventManager") else {
        return;
    };
    // SAFETY: on the main thread (AppKit posts this notification there);
    // `this` is our live receiver, which implements `GOT_URL` with the
    // signature `-setEventHandler:andSelector:forEventClass:andEventID:`
    // requires (`AEEventClass` and `AEEventID` are 32-bit codes).
    unsafe {
        let manager: Id = send0(manager_class, c"sharedAppleEventManager");
        if !manager.is_null() {
            send4::<Id, Sel, u32, u32, ()>(
                manager,
                c"setEventHandler:andSelector:forEventClass:andEventID:",
                this,
                selector(GOT_URL),
                GET_URL,
                GET_URL,
            );
        }
    }
}

/// A `kAEGetURL` event: hands its URL text to the handler. Runs on the main
/// thread inside the event loop, so the handler must return quickly.
extern "C" fn handle_get_url(_: Id, _: Sel, event: Id, _reply: Id) {
    let _pool = AutoreleasePool::new();
    // SAFETY: `event` is the live `NSAppleEventDescriptor` AppKit passes.
    let Some(url) = (unsafe { url_of(event) }) else {
        log::warn!("url events: an event without a URL");
        return;
    };
    if let Some(handler) = HANDLER.get() {
        // A panic must not unwind into AppKit (that aborts the process).
        let _ = std::panic::catch_unwind(|| handler(&url));
    }
}

/// The direct object of a `kAEGetURL` event as text.
///
/// # Safety
/// `event` is a live `NSAppleEventDescriptor` (or nil).
unsafe fn url_of(event: Id) -> Option<String> {
    if event.is_null() {
        return None;
    }
    // SAFETY: `-paramDescriptorForKeyword:` takes an `AEKeyword` (32-bit)
    // and returns a descriptor or nil; `-stringValue` an `NSString` or nil;
    // `-UTF8String` a pointer valid while the autorelease pool lives.
    unsafe {
        let descriptor: Id = send1(event, c"paramDescriptorForKeyword:", DIRECT_OBJECT);
        if descriptor.is_null() {
            return None;
        }
        let text: Id = send0(descriptor, c"stringValue");
        if text.is_null() {
            return None;
        }
        let utf8: *const c_char = send0(text, c"UTF8String");
        if utf8.is_null() {
            return None;
        }
        CStr::from_ptr(utf8).to_str().ok().map(str::to_owned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_char_codes_match_the_headers() {
        assert_eq!(GET_URL, 0x4755_524C);
        assert_eq!(DIRECT_OBJECT, 0x2D2D_2D2D);
    }

    #[test]
    fn a_missing_event_has_no_url() {
        // SAFETY: nil is allowed.
        assert_eq!(unsafe { url_of(std::ptr::null_mut()) }, None);
    }
}
