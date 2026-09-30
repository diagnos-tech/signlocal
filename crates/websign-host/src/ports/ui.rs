//! The windows, seen from the engine.

use websign_protocol::messages::DiagnosticsTab;
use websign_ui_model::confirm::UiCommand;

/// The confirmation window. Commands are queued to the UI thread and never
/// block; decisions come back as [`crate::EngineEvent::Ui`].
pub trait ConfirmUi {
    fn command(&mut self, command: UiCommand);

    /// The native handle of the confirmation window (HWND on Windows) once
    /// it exists, so OS PIN dialogs are owned by it and open in front.
    fn parent_window(&self) -> Option<isize>;
}

/// Starts other processes of this app.
pub trait Launcher {
    /// Opens the diagnostics window in a new process, so it survives this
    /// connection closing.
    fn open_diagnostics(&mut self, tab: Option<DiagnosticsTab>) -> Result<(), String>;
}
