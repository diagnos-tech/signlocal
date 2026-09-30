//! Spacing, sizes and radii (`docs/ux.md` §11.3), in logical points.
//!
//! Radii are `u8` because egui's `CornerRadius` is; everything else is `f32`.

pub const SPACE_1: f32 = 4.0;
pub const SPACE_2: f32 = 8.0;
pub const SPACE_3: f32 = 12.0;
pub const SPACE_4: f32 = 16.0;
pub const SPACE_5: f32 = 20.0;
pub const SPACE_6: f32 = 24.0;
pub const SPACE_8: f32 = 32.0;

/// Badges, identicon.
pub const RADIUS_SM: u8 = 4;
/// Buttons, fields, rows, tabs.
pub const RADIUS_MD: u8 = 6;
/// Cards, list, notices.
pub const RADIUS_LG: u8 = 10;

/// Checkbox, row buttons.
pub const CONTROL_SM: f32 = 28.0;
/// Fields, regular buttons; also the minimum target (§14: 32 × 32).
pub const CONTROL_MD: f32 = 32.0;
/// Footer buttons.
pub const CONTROL_LG: f32 = 36.0;

pub const ROW_CERT: f32 = 72.0;
pub const ROW_COMPACT: f32 = 40.0;
pub const ROW_DIAG: f32 = 56.0;

/// Icons inside a line of text.
pub const ICON_SM: f32 = 16.0;
/// Icon buttons and tabs.
pub const ICON_MD: f32 = 20.0;
/// Status cards.
pub const ICON_LG: f32 = 32.0;

pub const IDENTICON: f32 = 40.0;
pub const SIDEBAR: f32 = 200.0;
pub const POPUP_WIDTH: f32 = 320.0;
pub const WINDOW_CONFIRM: [f32; 2] = [480.0, 600.0];
pub const WINDOW_DIAGNOSTICS: [f32; 2] = [760.0, 540.0];

/// Focus ring stroke and its gap from the widget (§14).
pub const FOCUS_WIDTH: f32 = 2.0;
pub const FOCUS_OFFSET: f32 = 2.0;

/// `shadow-popover`: offset 0/4, blur 16, spread 0.
pub const SHADOW_POPOVER_OFFSET: [i8; 2] = [0, 4];
pub const SHADOW_POPOVER_BLUR: u8 = 16;

/// Primary buttons are never narrower than this, so "Sign" and its
/// translations keep one target size (§4.2).
pub const PRIMARY_MIN_WIDTH: f32 = 112.0;
/// Chip height (§4.2 "permission chip … (22 px)"); radius is half of it.
pub const CHIP_HEIGHT: f32 = 22.0;
