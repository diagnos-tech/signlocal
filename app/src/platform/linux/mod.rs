//! Linux implementations of the [`crate::platform`] functions.
//!
//! Nothing here needs `unsafe` beyond one `getuid`: the facts come from
//! `/proc`, the file system and the desktop's own command-line tools
//! (`gsettings`, `gdbus`, `xdg-open`), each run with a deadline.

pub mod caller;
pub mod channel;
mod command;
mod desktop_entry;
pub mod file_picker;
pub mod focus;
mod private_file;
pub mod settings;
pub mod system_ui;
