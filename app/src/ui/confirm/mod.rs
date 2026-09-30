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
