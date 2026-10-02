//! Exposes the identifiers from the repository's `project.toml` as
//! compile-time environment variables, so no name is hard-coded twice.

use std::path::Path;

const PROJECT_TOML: &str = "../../../../project.toml";

fn main() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(PROJECT_TOML);
    println!("cargo:rerun-if-changed={}", path.display());

    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let project: toml::Table = text.parse().expect("project.toml is not valid TOML");

    let exports = [
        ("WEBSIGN_PRODUCT_NAME", "product", "name"),
        ("WEBSIGN_SLUG", "product", "slug"),
        ("WEBSIGN_NATIVE_HOST", "ids", "native_host"),
        ("WEBSIGN_FIREFOX_ID", "extension", "firefox_id"),
        ("WEBSIGN_EXTENSION_DEV_ID", "extension", "dev_id"),
        (
            "WEBSIGN_CHROME_WEB_STORE_ID",
            "extension",
            "chrome_web_store_id",
        ),
        ("WEBSIGN_EDGE_ADDONS_ID", "extension", "edge_addons_id"),
    ];
    for (var, table, key) in exports {
        let value = project[table][key]
            .as_str()
            .unwrap_or_else(|| panic!("project.toml: [{table}].{key} must be a string"));
        println!("cargo:rustc-env={var}={value}");
    }
}
