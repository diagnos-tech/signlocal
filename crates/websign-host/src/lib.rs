//! The session engine of the app: everything between a transport and the
//! windows, with no OS or UI code of its own.
//!
//! A host process serves exactly one connection: a browser's native
//! messaging port or one `websign connect` child. The pieces:
//!
//! * [`launch`] recognizes a browser start from `argv` (promoted from the
//!   Phase-0 kit);
//! * [`caller`] is who asks: a web origin (from the browser) or a desktop
//!   program (from the OS);
//! * [`session`] parses frames, negotiates the version and enforces the
//!   per-transport rules;
//! * [`flow`] holds the per-request state machines (sign, choose);
//! * [`queue`] keeps one request on screen and up to ten waiting;
//! * [`store`] persists consent, connection records, settings and recent
//!   errors, without personal data;
//! * [`engine`] ties them together as a deterministic event loop over
//!   [`ports`] — the UI, the key store worker, the clock, the launcher;
//! * [`runtime`] wires real threads, stdio and the key store worker;
//! * `testing` (feature `testing`) holds fakes of every port.
//!
//! `SPEC.md` is the contract; `docs/architecture/protocol.md` the wire.

#![cfg_attr(
    not(test),
    warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod caller;
pub mod engine;
pub mod flow;
pub mod launch;
pub mod ports;
pub mod queue;
pub mod runtime;
pub mod session;
pub mod store;
#[cfg(any(test, feature = "testing"))]
pub mod testing;

pub use caller::Caller;
pub use engine::{Control, Engine, EngineConfig, EngineEvent};
pub use launch::{
    BrowserFamily, BrowserLaunch, SAFARI_FLAG, detect_browser_launch, is_appex_executable,
    parse_launch, parse_launch_at, running_in_appex,
};
