//! OS facts and actions the app needs outside key stores and registration.
//! One function per concern, one implementation per OS behind `cfg`.

pub mod appearance;
pub mod caller;
pub mod channel;
pub mod file_picker;
pub mod focus;
pub mod motion;
pub mod os_version;
pub mod secure_input;
pub mod system_ui;
pub mod url_events;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

#[cfg(target_os = "linux")]
use linux as os;
#[cfg(target_os = "macos")]
use macos as os;
#[cfg(windows)]
use windows as os;
