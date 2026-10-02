//! Color tokens (`docs/ux.md` §11.1), one field per `--ws-*` color of
//! `design/tokens.css`. Field names are the token names in `snake_case`.

use egui::Color32;

/// One theme's colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Colors {
    /// Window background (body).
    pub bg_canvas: Color32,
    /// Header, footer, cards, list, sidebar.
    pub bg_surface: Color32,
    /// Fields, identicon well, badges, diagnostics box.
    pub bg_sunken: Color32,
    /// Hover of rows and secondary buttons.
    pub bg_hover: Color32,
    /// Selected row or tab.
    pub accent_soft: Color32,
    /// Dividers and card outlines (decorative, no contrast requirement).
    pub border: Color32,
    /// Outline of fields, radios and checkboxes (≥ 3:1).
    pub border_strong: Color32,
    pub fg: Color32,
    pub fg_muted: Color32,
    /// Metadata, placeholders, scheme and subdomains.
    pub fg_subtle: Color32,
    /// Primary button background and checked radio.
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_pressed: Color32,
    /// Links, accent icons, selected tab text.
    pub accent_fg: Color32,
    /// Text over `accent`.
    pub on_accent: Color32,
    /// The 2 px focus ring.
    pub focus: Color32,
    pub success: Color32,
    pub success_soft: Color32,
    pub success_border: Color32,
    pub warning: Color32,
    pub warning_soft: Color32,
    pub warning_border: Color32,
    pub danger: Color32,
    pub danger_soft: Color32,
    pub danger_border: Color32,
    /// Popover and tooltip shadow (the only shadow, §11.3).
    pub shadow_color: Color32,
}

/// Identicon colors `id-0` … `id-7`, indexed by
/// `websign_protocol::code::VerificationCode::color_index`. The same in both
/// themes so the app and the website draw the same picture.
pub const IDENTICON: [Color32; 8] = [
    rgb(0xD6453A),
    rgb(0xC4610A),
    rgb(0x3B8A3B),
    rgb(0x12857B),
    rgb(0x2D78D2),
    rgb(0x5A55D6),
    rgb(0x9549C4),
    rgb(0xC63E7B),
];

/// Light theme colors.
pub fn light() -> Colors {
    Colors {
        bg_canvas: rgb(0xF5F6F9),
        bg_surface: rgb(0xFFFFFF),
        bg_sunken: rgb(0xEDEFF4),
        bg_hover: rgb(0xF1F3F8),
        accent_soft: rgb(0xEEF0FD),
        border: rgb(0xDCE0E8),
        border_strong: rgb(0x848D9F),
        fg: rgb(0x141722),
        fg_muted: rgb(0x545C6D),
        fg_subtle: rgb(0x636B7D),
        accent: rgb(0x3346D1),
        accent_hover: rgb(0x2A3BB8),
        accent_pressed: rgb(0x2332A0),
        accent_fg: rgb(0x3346D1),
        on_accent: rgb(0xFFFFFF),
        focus: rgb(0x3346D1),
        success: rgb(0x127A41),
        success_soft: rgb(0xE6F5EC),
        success_border: rgb(0xA8DCBE),
        warning: rgb(0x935300),
        warning_soft: rgb(0xFFF2D9),
        warning_border: rgb(0xEFCB8A),
        danger: rgb(0xBE242B),
        danger_soft: rgb(0xFDEBEC),
        danger_border: rgb(0xF2B3B6),
        shadow_color: rgba(0x141722, 0x1F),
    }
}

/// Dark theme colors.
pub fn dark() -> Colors {
    Colors {
        bg_canvas: rgb(0x0E1016),
        bg_surface: rgb(0x161922),
        bg_sunken: rgb(0x0B0D12),
        bg_hover: rgb(0x1E222D),
        accent_soft: rgb(0x1A1F3C),
        border: rgb(0x2A2F3C),
        border_strong: rgb(0x6B7488),
        fg: rgb(0xE7E9EF),
        fg_muted: rgb(0xA3AAB9),
        fg_subtle: rgb(0x8C94A6),
        accent: rgb(0x4E5EE4),
        accent_hover: rgb(0x4555DA),
        accent_pressed: rgb(0x3E4ECC),
        accent_fg: rgb(0x9DA8FF),
        on_accent: rgb(0xFFFFFF),
        focus: rgb(0x9DA8FF),
        success: rgb(0x5BCB8D),
        success_soft: rgb(0x10281B),
        success_border: rgb(0x1F5C3B),
        warning: rgb(0xEFB35E),
        warning_soft: rgb(0x2A1F0E),
        warning_border: rgb(0x5C4418),
        danger: rgb(0xFF868B),
        danger_soft: rgb(0x321519),
        danger_border: rgb(0x6B2429),
        shadow_color: rgba(0x000000, 0x8C),
    }
}

/// The colors of `dark` or light.
pub fn of(dark: bool) -> Colors {
    if dark { self::dark() } else { light() }
}

/// `0xRRGGBB` as written in `tokens.css`, so values can be compared by eye.
const fn rgb(hex: u32) -> Color32 {
    let [_, r, g, b] = hex.to_be_bytes();
    Color32::from_rgb(r, g, b)
}

/// `0xRRGGBB` plus the CSS alpha byte (`#RRGGBBAA`), unmultiplied as in CSS.
fn rgba(hex: u32, alpha: u8) -> Color32 {
    let [_, r, g, b] = hex.to_be_bytes();
    Color32::from_rgba_unmultiplied(r, g, b, alpha)
}
