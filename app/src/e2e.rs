//! Test-only behavior (feature `e2e`, never in release builds; CI fails the
//! release job if the release binary contains the marker string
//! `WEBSIGN_E2E_BUILD`).
//!
//! * `WEBSIGN_E2E_CONFIRM=sign|choose|remember|cancel` — the confirmation
//!   window acts by itself, through the same code path as a person: an
//!   accessibility click on its own widgets, which the window only counts
//!   once armed (after the real 600 ms). `sign` and `choose` press the
//!   primary button at every step (Continue, Use certificate, Sign);
//!   `remember` ticks "Remember this site" first; `cancel` presses Cancel.
//!   Unset, the window waits for the page (cancel from the page, tab
//!   closed).
//! * `WEBSIGN_E2E_PIN` — the PIN typed into our field for PKCS#11 keys.
//! * `WEBSIGN_E2E_SCREENSHOTS=<dir>` — every window state shown is saved as
//!   `<dir>/<window>-<state>-<theme>.png`, light and dark (egui viewport
//!   screenshot); the diagnostics window saves its tab and closes.
//!
//! A driven window counts as focused: Xvfb has no window manager to give it
//! focus, and desktop runners may keep a new window in the background.

mod capture;
mod confirm_driver;
mod diagnostics_driver;
mod png;
mod shadow;
mod tree;

use std::path::PathBuf;

use websign_host::ports::ConfirmUi;

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
        E2eConfig::from_vars(|name| std::env::var(name).ok())
    }

    /// Reads the variables through `get`; empty values count as unset.
    fn from_vars(get: impl Fn(&str) -> Option<String>) -> E2eConfig {
        let var = |name: &str| get(name).filter(|value| !value.trim().is_empty());
        E2eConfig {
            confirm: var("WEBSIGN_E2E_CONFIRM"),
            pin: var("WEBSIGN_E2E_PIN"),
            screenshots: var("WEBSIGN_E2E_SCREENSHOTS").map(PathBuf::from),
        }
    }

    /// Whether the window has anything to do by itself.
    fn is_active(&self) -> bool {
        self.confirm.is_some() || self.screenshots.is_some()
    }
}

/// The window an e2e hook drives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Window {
    Confirm,
    Diagnostics,
}

/// Installs the e2e driver on a window's egui context, when the environment
/// asks for one. Called from the window's app creator.
pub fn attach(ctx: &egui::Context, window: Window) {
    let config = E2eConfig::from_env();
    if !config.is_active() {
        return;
    }
    log::warn!("{MARKER}: {window:?} window driven by the e2e hook");
    // The driver reads the widgets from the accessibility tree.
    ctx.enable_accesskit();
    match window {
        Window::Confirm => ctx.add_plugin(confirm_driver::ConfirmDriver::new(config)),
        Window::Diagnostics => ctx.add_plugin(diagnostics_driver::DiagnosticsDriver::new(config)),
    }
}

/// Wraps the window's port so the driver knows which state the window
/// shows (it names the screenshots after it).
pub fn watch(window: Box<dyn ConfirmUi>) -> Box<dyn ConfirmUi> {
    Box::new(shadow::Watched::new(window))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_variables_and_ignores_empty_ones() {
        let config = E2eConfig::from_vars(|name| match name {
            "WEBSIGN_E2E_CONFIRM" => Some("sign".to_owned()),
            "WEBSIGN_E2E_PIN" => Some(" ".to_owned()),
            "WEBSIGN_E2E_SCREENSHOTS" => Some("/tmp/shots".to_owned()),
            _ => None,
        });
        assert_eq!(config.confirm.as_deref(), Some("sign"));
        assert_eq!(config.pin, None);
        assert_eq!(config.screenshots, Some(PathBuf::from("/tmp/shots")));
        assert!(config.is_active());
        assert!(!E2eConfig::default().is_active());
    }
}
