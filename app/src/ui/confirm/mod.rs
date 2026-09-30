//! The confirmation window (`docs/ux.md` §4–§6), 480 × 600, fixed.
//!
//! Renders `websign_ui_model::confirm::ConfirmView`; converts egui input to
//! `UserInput`; turns `Intent`s into `UiEvent`s (adding the PIN from the
//! field and zeroizing it). The model decides everything (states, arming,
//! which button does what); this module only draws and reports.

mod clock;
mod failure_text;
mod guard;
mod outbox;
mod row_text;
mod run;
mod session;
mod view;
mod viewport;
mod window;
mod words;

#[cfg(test)]
mod tests;

/// Runs the window's event loop on the calling (main) thread, starting with
/// `first` and then every command from `commands`; decisions go to
/// `events`. Returns `Ok` once `commands` is disconnected (the engine
/// ended), after the closing animation. The app creator calls
/// `theme::install(creation, platform::motion::reduce_motion())` and starts
/// the backend of `renderer::choose()`. On `Err` the host keeps `commands`
/// and fails the requests itself (`host_process::ui_thread`).
pub fn run(
    first: websign_ui_model::confirm::UiCommand,
    commands: &std::sync::mpsc::Receiver<websign_ui_model::confirm::UiCommand>,
    events: websign_host::runtime::EventSender,
) -> Result<(), eframe::Error> {
    run::run(first, commands, events)
}
