//! How a button looks: its weight, height, padding and colors per pointer
//! state (`docs/ux.md` §11.3; the mockups' `.btn`, `.md`, `.sm`, `.ghost`).

use egui::{Color32, Response};

use crate::ui::theme::{Colors, metrics};

/// Visual weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The one main action (Sign, Use this certificate).
    Primary,
    Secondary,
    /// Text-like action (Details, Revoke in a row).
    Ghost,
}

/// Height: `control-sm` 28, `control-md` 32, `control-lg` 36 (footer).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Small,
    Medium,
    Large,
}

/// The control height of `size`.
pub fn height(size: Size) -> f32 {
    match size {
        Size::Small => metrics::CONTROL_SM,
        Size::Medium => metrics::CONTROL_MD,
        Size::Large => metrics::CONTROL_LG,
    }
}

/// Horizontal padding: 16 on footer buttons, 8 on ghost ones, else 12.
pub fn padding(kind: Kind, size: Size) -> f32 {
    match (kind, size) {
        (Kind::Ghost, _) => metrics::SPACE_2,
        (_, Size::Large) => metrics::SPACE_4,
        _ => metrics::SPACE_3,
    }
}

/// `(fill, border, text)` for the pointer state.
pub fn colors(kind: Kind, c: &Colors, response: &Response) -> (Color32, Color32, Color32) {
    let pressed = response.is_pointer_button_down_on();
    let hovered = response.hovered();
    match kind {
        Kind::Primary => {
            let fill = if pressed {
                c.accent_pressed
            } else if hovered {
                c.accent_hover
            } else {
                c.accent
            };
            (fill, fill, c.on_accent)
        }
        Kind::Secondary => {
            let fill = if hovered { c.bg_hover } else { c.bg_surface };
            (fill, c.border_strong, c.fg)
        }
        Kind::Ghost => {
            let fill = if hovered {
                c.bg_hover
            } else {
                Color32::TRANSPARENT
            };
            (fill, Color32::TRANSPARENT, c.accent_fg)
        }
    }
}

/// Opacity of a disabled or unarmed button (the mockups' `.42`).
pub const DISABLED_OPACITY: f32 = 0.42;
