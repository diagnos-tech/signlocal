//! Identifiers from the repository's `project.toml`, injected by `build.rs`.

pub const PRODUCT_NAME: &str = env!("WEBSIGN_PRODUCT_NAME");
pub const SLUG: &str = env!("WEBSIGN_SLUG");
pub const NATIVE_HOST: &str = env!("WEBSIGN_NATIVE_HOST");
pub const URL_SCHEME: &str = env!("WEBSIGN_URL_SCHEME");
pub const FIREFOX_ID: &str = env!("WEBSIGN_FIREFOX_ID");
pub const EXTENSION_DEV_ID: &str = env!("WEBSIGN_EXTENSION_DEV_ID");
pub const EXTENSION_DEV_KEY: &str = env!("WEBSIGN_EXTENSION_DEV_KEY");
/// Empty until the extension is published.
pub const CHROME_WEB_STORE_ID: &str = env!("WEBSIGN_CHROME_WEB_STORE_ID");
/// Empty until the extension is published.
pub const EDGE_ADDONS_ID: &str = env!("WEBSIGN_EDGE_ADDONS_ID");
