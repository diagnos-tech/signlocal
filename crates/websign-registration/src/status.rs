//! Reads registrations back, for diagnostics ("Chrome can't find the app")
//! and for the repair button.

use crate::browsers::Browser;

/// The state of one browser's registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistrationState {
    /// A manifest exists and starts this app.
    Registered,
    /// A manifest exists but points elsewhere (an old install, another copy).
    PointsElsewhere { path: String },
    /// A manifest exists but is not valid JSON or lacks our extension IDs.
    Broken { reason: String },
    /// No manifest where this browser looks.
    Missing,
}

/// The registration state of `browser` for the current user (and, on Linux,
/// system-wide).
pub fn registration_state(browser: Browser) -> RegistrationState {
    let _ = browser;
    todo!("SPEC.md §3")
}
