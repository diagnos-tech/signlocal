//! The confirmation window: its contract with the host ([`port`]) and its
//! state machine ([`machine`], `docs/ux.md` §4.8), with the anti-accident
//! rules ([`arming`], §4.7) and the cancel-code rule ([`cancel`], §15).

pub mod arming;
pub mod cancel;
pub mod machine;
pub mod port;
pub mod view;

pub use machine::{ConfirmModel, ConfirmState, Intent, UserInput};
pub use port::{UiCommand, UiEvent};
pub use view::ConfirmView;
