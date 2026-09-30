//! Light or dark, as the OS is set (`docs/ux.md` §4.1 "Theme").
//!
//! egui follows the OS theme through winit where winit knows it; this answers
//! where it does not (several Linux desktops) and for anything drawn before
//! a window exists.

/// `Some(true)` when the OS asks apps to be dark, `Some(false)` for light,
/// `None` when it states no preference or cannot be read.
///
/// Windows: `AppsUseLightTheme`; macOS: `AppleInterfaceStyle`; Linux: the
/// XDG settings portal's `color-scheme`, else GNOME's `color-scheme`.
pub fn dark_mode() -> Option<bool> {
    super::os::settings::dark_mode()
}
