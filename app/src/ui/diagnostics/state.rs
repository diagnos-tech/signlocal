//! What the window remembers between frames, and what the person asked for
//! this frame. Views read the state and emit [`Action`]s; the window applies
//! them after drawing, so drawing never changes stores or starts work.

use std::path::PathBuf;

use websign_core::Fingerprint;
use websign_protocol::messages::DiagnosticsTab;

/// Seconds "Revoke" stays turned into "Confirm revoke" (§8.3).
pub const REVOKE_CONFIRM_SECONDS: f64 = 4.0;
/// Seconds a short confirmation ("Diagnostics copied…") stays on screen.
pub const NOTICE_SECONDS: f64 = 4.0;

/// The four tabs, in sidebar order (Ctrl/⌘+1…4).
pub const TABS: [DiagnosticsTab; 4] = [
    DiagnosticsTab::Browsers,
    DiagnosticsTab::Devices,
    DiagnosticsTab::Certificates,
    DiagnosticsTab::Help,
];

/// Window state.
#[derive(Debug, Clone, Default)]
pub struct UiState {
    /// `None` until the first scan decides it (the first tab with a problem).
    pub tab: Option<DiagnosticsTab>,
    /// A consent key whose "Revoke" was pressed once, until this time.
    pub revoke_armed: Option<(String, f64)>,
    pub notice: Option<(Notice, f64)>,
    /// Open questions of the Help tab, by index; the first starts open.
    pub faq_open: [bool; 4],
    /// "Can't sign (n)" expanded.
    pub hidden_open: bool,
    /// The path being typed after "Add driver…", when this computer has no
    /// file picker.
    pub driver_input: Option<String>,
}

impl UiState {
    pub fn new(tab: Option<DiagnosticsTab>) -> UiState {
        UiState {
            tab,
            faq_open: [true, false, false, false],
            ..UiState::default()
        }
    }

    /// The notice still on screen at `now`.
    pub fn notice_at(&self, now: f64) -> Option<&Notice> {
        self.notice
            .as_ref()
            .filter(|(_, until)| now < *until)
            .map(|(notice, _)| notice)
    }

    /// Whether `key`'s "Revoke" is waiting for confirmation at `now`.
    pub fn revoke_pending(&self, key: &str, now: f64) -> bool {
        self.revoke_armed
            .as_ref()
            .is_some_and(|(armed, until)| armed == key && now < *until)
    }
}

/// A short confirmation, announced as a live region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notice {
    Copied,
    /// The site or program no longer allowed, as shown.
    Revoked(String),
    /// Registration rewritten; this browser (as shown) needs a restart.
    Repaired(String),
    /// Registration could not be rewritten.
    RepairFailed,
}

/// Something the person did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    SelectTab(DiagnosticsTab),
    Rescan,
    CopyReport,
    /// Copies a text shown on screen (masked ATR, the `pcscd` command).
    CopyText(String),
    OpenUrl(String),
    /// "Repair" on a browser's row (its name as shown).
    Repair(String),
    /// First press of "Revoke".
    ArmRevoke(String),
    /// "Confirm revoke".
    Revoke {
        key: String,
        shown: String,
    },
    /// "Add driver…": the OS file picker, else the path field.
    PickDriver,
    /// The path typed so far (no file picker on this computer).
    EditDriverInput(String),
    CancelDriverInput,
    AddDriver(PathBuf),
    RemoveDriver(PathBuf),
    ImportPfx,
    Details(Fingerprint),
    ToggleFaq(usize),
    ToggleHidden,
    DismissOnboarding,
    Close,
}
