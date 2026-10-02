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

    /// Shows the OS certificate viewer for `der`, owned by the confirmation
    /// window ("View in system", `docs/ux.md` §5.12). Must not block: the
    /// viewers of Windows and macOS are modal and belong on the UI thread,
    /// so the implementation hands the bytes over and returns. The default
    /// has no viewer (scripted windows).
    fn view_certificate(&mut self, der: Vec<u8>) {
        let _ = der;
        log::debug!("this window has no certificate viewer");
    }
}

/// Starts other processes of this app.
pub trait Launcher {
    /// Opens the diagnostics window in a new process, so it survives this
    /// connection closing.
    fn open_diagnostics(&mut self, tab: Option<DiagnosticsTab>) -> Result<(), String>;
}
