//! Linux implementations of the [`crate::platform`] functions.
//!
//! Each concern gets its own file here (`caller.rs`, `focus.rs`, …) as the
//! platform track implements it; `unsafe` stays in this folder, every block
//! with a `// SAFETY:` comment, every handle in an RAII type.
