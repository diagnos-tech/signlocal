//! Windows implementations of the [`crate::platform`] functions.
//!
//! `unsafe` stays in this folder, every block with a `// SAFETY:` comment;
//! kernel handles are held in `std`'s `OwnedHandle`, other resources in
//! small RAII guards next to their use.

mod authenticode;
pub mod caller;
mod catalog;
pub mod channel;
pub mod focus;
pub mod settings;
pub mod system_ui;
mod trust;
mod version_info;
mod wide;
