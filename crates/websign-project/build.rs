//! Exposes the identifiers of the repository's `project.toml` as compile-time
//! environment variables read by `src/lib.rs`. Promoted from the Phase-0 kit
//! (`probe/build.rs`), reviewed.

use std::path::Path;

const PROJECT_TOML: &str = "../../project.toml";

/// `(environment variable, table, key)` for every exported string.
const EXPORTS: [(&str, &str, &str); 14] = [
    ("WEBSIGN_PRODUCT_NAME", "product", "name"),
    ("WEBSIGN_SLUG", "product", "slug"),
    ("WEBSIGN_TAGLINE", "product", "tagline"),
    ("WEBSIGN_REPOSITORY", "product", "repository"),
    ("WEBSIGN_HOMEPAGE", "product", "homepage"),
    ("WEBSIGN_NATIVE_HOST", "ids", "native_host"),
    ("WEBSIGN_URL_SCHEME", "ids", "url_scheme"),
    ("WEBSIGN_MACOS_BUNDLE_ID", "ids", "macos_bundle_id"),
    ("WEBSIGN_MACOS_APP_GROUP", "ids", "macos_app_group"),
    ("WEBSIGN_FIREFOX_ID", "extension", "firefox_id"),
    ("WEBSIGN_EXTENSION_DEV_ID", "extension", "dev_id"),
    (
        "WEBSIGN_CHROME_WEB_STORE_ID",
        "extension",
        "chrome_web_store_id",
    ),
    ("WEBSIGN_EDGE_ADDONS_ID", "extension", "edge_addons_id"),
    ("WEBSIGN_MIN_APP_VERSION", "extension", "min_app_version"),
];

// A build script reports failure by panicking; there is no caller to return to.
#[allow(clippy::panic, clippy::expect_used)]
fn main() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(PROJECT_TOML);
    println!("cargo:rerun-if-changed={}", path.display());

    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let project: toml::Table = text.parse().expect("project.toml is not valid TOML");

    for (var, table, key) in EXPORTS {
        let value = project
            .get(table)
            .and_then(|t| t.get(key))
            .and_then(toml::Value::as_str)
            .unwrap_or_else(|| panic!("project.toml: [{table}].{key} must be a string"));
        println!("cargo:rustc-env={var}={value}");
    }
}
