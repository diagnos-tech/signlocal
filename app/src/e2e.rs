//! Test-only behavior (feature `e2e`, never in release builds; CI fails the
//! release job if the release binary contains the marker string
//! `WEBSIGN_E2E_BUILD`).
//!
//! * `WEBSIGN_E2E_CONFIRM=sign|cancel|choose` — the confirmation window
//!   performs that action by itself as soon as it is armed (after the real
//!   600 ms), through the same code path as a click.
//! * `WEBSIGN_E2E_PIN` — the PIN typed into our field for PKCS#11 keys.
//! * `WEBSIGN_E2E_SCREENSHOTS=<dir>` — every window state shown is saved as
//!   `<dir>/<window>-<state>-<theme>.png` (egui viewport screenshot).

/// Marker the release job searches for in the binary.
pub const MARKER: &str = "WEBSIGN_E2E_BUILD";

/// What the e2e environment asks for, read once at start.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct E2eConfig {
    pub confirm: Option<String>,
    pub pin: Option<String>,
    pub screenshots: Option<std::path::PathBuf>,
}

impl E2eConfig {
    /// Reads the `WEBSIGN_E2E_*` variables.
    pub fn from_env() -> E2eConfig {
        todo!("testing.md §E2E")
    }
}
