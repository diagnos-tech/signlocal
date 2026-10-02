//! Provisional names and identifiers, from the repository's `project.toml`.
//!
//! Renaming the product must only require editing that file (`docs/plan.md`
//! D6). Every crate reads names from here; none spells them out. The values
//! live in `generated.rs`, written by `cargo xtask gen` and checked by
//! `cargo xtask check generated`, so the crate is self-contained when
//! packaged for crates.io.

#![cfg_attr(
    not(test),
    warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

mod generated;

pub use generated::*;

/// Every Chromium extension ID the native host must allow: the development
/// build and the store builds that exist.
pub fn chromium_extension_ids() -> Vec<&'static str> {
    [EXTENSION_DEV_ID, CHROME_WEB_STORE_ID, EDGE_ADDONS_ID]
        .into_iter()
        .filter(|id| !id.is_empty())
        .collect()
}

/// Chromium extension IDs are 32 letters from `a` to `p` (the SHA-256 of the
/// extension's public key, hex-encoded with an `a`-based alphabet). Promoted
/// from the Phase-0 kit (`probe/src/nm/launch.rs`).
pub fn is_chromium_extension_id(id: &str) -> bool {
    id.len() == 32 && id.bytes().all(|b| (b'a'..=b'p').contains(&b))
}
