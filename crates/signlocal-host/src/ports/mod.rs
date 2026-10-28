//! What the engine needs from the outside world, as traits.
//!
//! Production implementations live in [`crate::runtime`] (stdio, the key store
//! worker thread, the system clock) and in the app (the egui window, process
//! launching). Tests use fakes and a manual clock, so every scenario of
//! `SPEC.md` §8 runs in milliseconds.

mod clock;
mod keys;
mod outbound;
mod ui;

pub use clock::{Clock, SystemClock, unix_seconds};
pub use keys::{KeyCommand, KeyReply, KeyService, KeySnapshot, SLOW_LISTING};
pub use outbound::{Outbound, OutboundError};
pub use ui::{ConfirmUi, Launcher};
