//! The meaning a colored element carries. Color is never the only signal
//! (`docs/ux.md` §14): every tone also comes with an icon shape and text.

use egui::Color32;

use crate::ui::icons::{self, Icon};
use crate::ui::theme::Colors;

/// What an element is saying.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// Plain status ("New site", badges).
    Neutral,
    /// Information worth reading ("The site will receive…").
    Info,
    Success,
    /// Needs attention; signing still works.
    Warning,
    /// Prevents signing.
    Danger,
}

/// The colors of one tone in one theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TonePalette {
    /// Chip text and inline status text.
    pub text: Color32,
    /// Chip and notice background.
    pub fill: Color32,
    pub border: Color32,
    pub icon: Color32,
}

impl Tone {
    pub fn palette(self, c: &Colors) -> TonePalette {
        let (text, fill, border, icon) = match self {
            Tone::Neutral => (c.fg_muted, c.bg_sunken, c.border, c.fg_muted),
            Tone::Info => (c.fg_muted, c.bg_surface, c.border, c.accent_fg),
            Tone::Success => (c.success, c.success_soft, c.success_border, c.success),
            Tone::Warning => (c.warning, c.warning_soft, c.warning_border, c.warning),
            Tone::Danger => (c.danger, c.danger_soft, c.danger_border, c.danger),
        };
        TonePalette {
            text,
            fill,
            border,
            icon,
        }
    }

    /// The status icon of this tone (§12: fill weight for status).
    pub fn icon(self) -> Icon {
        match self {
            Tone::Neutral | Tone::Info => icons::INFO,
            Tone::Success => icons::SUCCESS,
            Tone::Warning => icons::ATTENTION,
            Tone::Danger => icons::ERROR,
        }
    }

    /// How urgently a screen reader announces a change (§14: errors are
    /// assertive, everything else polite).
    pub fn live(self) -> egui::accesskit::Live {
        match self {
            Tone::Danger => egui::accesskit::Live::Assertive,
            _ => egui::accesskit::Live::Polite,
        }
    }
}
