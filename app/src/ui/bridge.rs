//! The [`websign_host::ports::ConfirmUi`] implementation: forwards
//! `UiCommand`s to the UI thread and wakes egui; the window posts
//! `UiEvent`s back into the engine's channel.
//!
//! The engine and the window live on different threads and are built apart
//! (`host_process`), so they meet here: the window registers its egui
//! context and native handle in [`LINK`] once it exists, and the bridge uses
//! them to wake the UI thread and to parent OS PIN dialogs. A process shows
//! at most one confirmation window (winit allows one event loop per
//! process), so one link per process is enough.

use std::sync::mpsc::Sender;
use std::sync::{Mutex, MutexGuard, PoisonError};

use websign_host::ports::ConfirmUi;
use websign_ui_model::confirm::UiCommand;

/// What the engine thread knows about the live window.
#[derive(Default)]
struct Link {
    ctx: Option<egui::Context>,
    handle: Option<isize>,
    /// A certificate waiting for the OS viewer: the viewers of Windows and
    /// macOS are modal and must run on the UI thread, never on the engine's.
    viewer: Option<Vec<u8>>,
}

static LINK: Mutex<Link> = Mutex::new(Link {
    ctx: None,
    handle: None,
    viewer: None,
});

/// A poisoned lock only means another thread panicked mid-update of two
/// plain values; they are still usable.
fn link() -> MutexGuard<'static, Link> {
    LINK.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Called by the window once its event loop runs: commands sent from now on
/// wake it at once instead of waiting for its next scheduled frame.
pub fn connect(ctx: &egui::Context) {
    link().ctx = Some(ctx.clone());
}

/// Records the window's native handle (`None` while it has none).
pub fn set_native_handle(handle: Option<isize>) {
    link().handle = handle;
}

/// Called when the event loop ends.
pub fn disconnect() {
    *link() = Link::default();
}

/// The certificate the engine asked to show in the OS viewer, once.
pub fn take_certificate() -> Option<Vec<u8>> {
    link().viewer.take()
}

/// The engine-side handle of the window.
#[derive(Debug)]
pub struct WindowBridge {
    commands: Sender<UiCommand>,
}

impl WindowBridge {
    /// A bridge sending to the UI thread's receiver.
    pub fn new(commands: Sender<UiCommand>) -> WindowBridge {
        WindowBridge { commands }
    }
}

impl ConfirmUi for WindowBridge {
    fn command(&mut self, command: UiCommand) {
        if self.commands.send(command).is_err() {
            // The UI thread is gone (the window could not open); the host
            // already answers every request on screen with `Internal`.
            log::warn!("the confirmation window is gone; command dropped");
            return;
        }
        if let Some(ctx) = &link().ctx {
            ctx.request_repaint();
        }
    }

    fn parent_window(&self) -> Option<isize> {
        link().handle
    }

    /// Hands `der` to the UI thread, which opens the viewer owned by the
    /// window between frames ([`take_certificate`]). A second request before
    /// that replaces the first: the person asked for the latest.
    fn view_certificate(&mut self, der: Vec<u8>) {
        let mut link = link();
        link.viewer = Some(der);
        if let Some(ctx) = &link.ctx {
            ctx.request_repaint();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;

    use super::*;

    /// One test touches the process-wide link, so no other test races it.
    #[test]
    fn forwards_commands_wakes_the_window_and_reports_its_handle() {
        let (sender, receiver) = channel();
        let mut bridge = WindowBridge::new(sender);
        bridge.command(UiCommand::Hide);
        assert_eq!(receiver.try_recv(), Ok(UiCommand::Hide));
        assert_eq!(bridge.parent_window(), None);

        let ctx = egui::Context::default();
        connect(&ctx);
        set_native_handle(Some(42));
        // The first frames request their own repaints (fonts, layout).
        for _ in 0..3 {
            let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
            output.textures_delta.clear();
        }
        assert!(!ctx.has_requested_repaint());
        bridge.command(UiCommand::Hide);
        assert!(ctx.has_requested_repaint(), "a command wakes the window");
        assert_eq!(bridge.parent_window(), Some(42));

        for _ in 0..3 {
            let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
            output.textures_delta.clear();
        }
        bridge.view_certificate(vec![0x30, 0x03]);
        assert!(
            ctx.has_requested_repaint(),
            "a certificate wakes the window"
        );
        assert_eq!(take_certificate(), Some(vec![0x30, 0x03]));
        assert_eq!(take_certificate(), None, "shown once");

        disconnect();
        assert_eq!(bridge.parent_window(), None);
        drop(receiver);
        bridge.command(UiCommand::Hide);
    }
}
