//! Real threads around the engine: the stdio reader, the key store worker,
//! the device monitor and the tick timer. The app supplies the UI port and
//! the launcher, and runs the UI event loop on the main thread.

mod key_worker;
mod stdio;

use std::sync::mpsc::Sender;

pub use key_worker::spawn_key_worker;
pub use stdio::{StdoutOutbound, spawn_stdin_reader};

use crate::engine::{EngineConfig, EngineEvent};
use crate::ports::{ConfirmUi, Launcher};

/// Channel into the engine, cloned into every producer (the UI included).
pub type EventSender = Sender<EngineEvent>;

/// Runs a host process on stdin/stdout until the client disconnects or the
/// idle limit passes; returns the exit status.
///
/// `ui` is built by the app with the returned [`EventSender`] so the window
/// can post decisions; the engine runs on the calling thread, so the app
/// calls this from a worker thread and keeps the main thread for egui.
pub fn serve_stdio(
    config: EngineConfig,
    make_ui: impl FnOnce(EventSender) -> Box<dyn ConfirmUi>,
    launcher: Box<dyn Launcher>,
) -> i32 {
    let _ = (config, make_ui, launcher);
    todo!("SPEC.md §9")
}
