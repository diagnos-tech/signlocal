//! The confirmation window (`docs/ux.md` §4–§6), 480 × 600, fixed.
//!
//! Renders `websign_ui_model::confirm::ConfirmView`; converts egui input to
//! `UserInput`; turns `Intent`s into `UiEvent`s (adding the PIN from the
//! field and zeroizing it).

/// The eframe app state of the confirmation window.
#[derive(Debug)]
pub struct ConfirmWindow {
    _private: (),
}

/// Runs the window's event loop on the calling (main) thread, starting with
/// `first` and then every command from `commands`; decisions go to
/// `events`. Returns once `commands` is disconnected (the engine ended),
/// after the closing animation.
pub fn run(
    first: websign_ui_model::confirm::UiCommand,
    commands: std::sync::mpsc::Receiver<websign_ui_model::confirm::UiCommand>,
    events: websign_host::runtime::EventSender,
) {
    let _ = (first, commands, events);
    todo!("ui/confirm track: the window event loop")
}
