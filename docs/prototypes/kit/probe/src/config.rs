//! Identifiers from the repository's `project.toml`, injected by `build.rs`.

pub const PRODUCT_NAME: &str = env!("SIGNLOCAL_PRODUCT_NAME");
pub const SLUG: &str = env!("SIGNLOCAL_SLUG");
pub const NATIVE_HOST: &str = env!("SIGNLOCAL_NATIVE_HOST");
pub const FIREFOX_ID: &str = env!("SIGNLOCAL_FIREFOX_ID");
pub const EXTENSION_DEV_ID: &str = env!("SIGNLOCAL_EXTENSION_DEV_ID");
/// Empty until the extension is published.
pub const CHROME_WEB_STORE_ID: &str = env!("SIGNLOCAL_CHROME_WEB_STORE_ID");
/// Empty until the extension is published.
pub const EDGE_ADDONS_ID: &str = env!("SIGNLOCAL_EDGE_ADDONS_ID");
