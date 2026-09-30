//! Provisional names and identifiers, from the repository's `project.toml`.
//!
//! Renaming the product must only require editing that file (`docs/plan.md`
//! D6). Every crate reads names from here; none spells them out.

#![cfg_attr(
    not(test),
    warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

/// Product name shown to people: `"WebeSign"`.
pub const PRODUCT_NAME: &str = env!("WEBSIGN_PRODUCT_NAME");
/// Lowercase ASCII name for binaries, folders and files: `"websign"`.
pub const SLUG: &str = env!("WEBSIGN_SLUG");
/// One-line English description.
pub const TAGLINE: &str = env!("WEBSIGN_TAGLINE");
/// Source repository URL.
pub const REPOSITORY: &str = env!("WEBSIGN_REPOSITORY");
/// Website (download, privacy, `/activate`, `/test`).
pub const HOMEPAGE: &str = env!("WEBSIGN_HOMEPAGE");
/// Native messaging host name (`[a-z0-9_.]` only).
pub const NATIVE_HOST: &str = env!("WEBSIGN_NATIVE_HOST");
/// URL scheme the app registers (`websign:`), without the colon.
pub const URL_SCHEME: &str = env!("WEBSIGN_URL_SCHEME");
/// macOS bundle identifier.
pub const MACOS_BUNDLE_ID: &str = env!("WEBSIGN_MACOS_BUNDLE_ID");
/// macOS app group shared with the Safari extension.
pub const MACOS_APP_GROUP: &str = env!("WEBSIGN_MACOS_APP_GROUP");
/// Firefox (Gecko) extension ID.
pub const FIREFOX_ID: &str = env!("WEBSIGN_FIREFOX_ID");
/// Chromium extension ID of the development build (pinned by its public key).
pub const EXTENSION_DEV_ID: &str = env!("WEBSIGN_EXTENSION_DEV_ID");
/// Chrome Web Store ID; empty until published.
pub const CHROME_WEB_STORE_ID: &str = env!("WEBSIGN_CHROME_WEB_STORE_ID");
/// Edge Add-ons ID; empty until published.
pub const EDGE_ADDONS_ID: &str = env!("WEBSIGN_EDGE_ADDONS_ID");
/// Oldest app version the extension accepts.
pub const MIN_APP_VERSION: &str = env!("WEBSIGN_MIN_APP_VERSION");

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
