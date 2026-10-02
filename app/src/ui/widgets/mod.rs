//! Widgets egui lacks, one per file, shared by both windows: buttons with
//! arming, chip, badge, banner, identicon and code card, certificate row,
//! list, text and PIN fields, spinner, skeleton, tabs and the focus ring
//! (`docs/ux.md` §11.5).
//!
//! Widgets take texts already localized and formatted: they know how things
//! look, never what they say. Each exposes a role and a name to AccessKit.

pub mod badge;
pub mod banner;
pub mod button;
pub mod cert_row;
pub mod chip;
pub mod code_card;
pub mod field;
pub mod focus;
pub mod identicon;
pub mod keys;
pub mod link;
pub mod list;
pub mod pin_buffer;
pub mod pin_field;
pub mod pin_keys;
pub mod radio;
pub mod skeleton;
pub mod spinner;
pub mod tabs;
pub mod text;
pub mod text_field;
pub mod tone;

#[cfg(test)]
mod tests;
