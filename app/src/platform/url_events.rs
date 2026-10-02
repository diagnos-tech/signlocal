//! `websign:` URLs that arrive as OS events instead of arguments.
//!
//! Windows and Linux start a new process with the URL as its only argument,
//! which [`crate::launch`] reads. macOS never does: Launch Services starts
//! the app without the URL and delivers it as an Apple Event (`kAEGetURL`),
//! also to a copy of the app that is already running with a window.
//!
//! The event is only dispatched while an AppKit event loop runs (the
//! diagnostics or confirmation window's), so the windows need no code of
//! their own for it: [`listen`] hooks into AppKit's launch, before winit or
//! eframe dispatch the first event.

/// Calls `handler` on the main thread with the text of every URL the OS
/// delivers as an event. Call once, at the start of `main`, before any
/// window exists; the handler must return quickly (it runs inside the event
/// loop) and must treat the URL as untrusted text. Does nothing off macOS.
pub fn listen(handler: fn(&str)) {
    #[cfg(target_os = "macos")]
    super::os::url_events::listen(handler);
    #[cfg(not(target_os = "macos"))]
    let _ = handler;
}
