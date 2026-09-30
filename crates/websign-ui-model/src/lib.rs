//! What the windows show and how they react, as pure functions and state
//! machines (`docs/ux.md` §4–§8, test vectors §16).
//!
//! The app's egui code only renders what this crate decides; the host only
//! talks to the window through [`confirm::port`]. Keeping the decisions here
//! lets blind-TDD agents test every rule without a GPU, a display or an OS
//! key store. Nothing here is localized: views carry structured values and
//! message keys are chosen by the renderer.

#![cfg_attr(
    not(test),
    warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod certs;
pub mod confirm;
pub mod diagnostics;
pub mod possible;
pub mod time;

#[cfg(test)]
mod fixtures;
