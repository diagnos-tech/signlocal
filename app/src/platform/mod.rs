//! OS facts and actions the app needs outside key stores and registration.
//! One function per concern, one implementation per OS behind `cfg`.

pub mod caller;
pub mod channel;
pub mod focus;
pub mod motion;
pub mod system_ui;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;
