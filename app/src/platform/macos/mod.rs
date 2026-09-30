//! macOS implementations of the [`crate::platform`] functions.
//!
//! `unsafe` stays in this folder, every block with a `// SAFETY:` comment;
//! Core Foundation objects are held in `core-foundation`'s RAII types.

mod bundle;
pub mod caller;
pub mod channel;
mod code_signature;
pub mod file_picker;
pub mod focus;
mod objc;
mod peer;
pub mod secure_input;
pub mod settings;
pub mod system_ui;
pub mod url_events;
