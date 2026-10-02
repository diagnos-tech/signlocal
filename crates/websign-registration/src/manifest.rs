//! The native messaging host manifest: the small JSON file a browser reads to
//! learn which program to start and which extensions may talk to it.

use std::path::Path;

use anyhow::Context as _;
use serde_json::json;

use super::browsers::Family;
use websign_project::{FIREFOX_ID, NATIVE_HOST, PRODUCT_NAME, chromium_extension_ids};

/// File name browsers expect on Linux and macOS.
pub fn file_name() -> String {
    format!("{NATIVE_HOST}.json")
}

/// The manifest text for `family`, starting `host_path`.
pub fn render(family: Family, host_path: &Path, origins: &[String]) -> anyhow::Result<String> {
    let host_path = host_path
        .to_str()
        .with_context(|| format!("{} is not valid UTF-8", host_path.display()))?;
    let mut manifest = json!({
        "name": NATIVE_HOST,
        "description": format!("{PRODUCT_NAME} native messaging host"),
        "path": host_path,
        "type": "stdio",
    });
    match family {
        Family::Chromium => manifest["allowed_origins"] = json!(origins),
        Family::Firefox => manifest["allowed_extensions"] = json!([FIREFOX_ID]),
    }
    let mut text = serde_json::to_string_pretty(&manifest)?;
    text.push('\n');
    Ok(text)
}

/// The `chrome-extension://<id>/` origins the manifest allows: exactly the
/// `project.toml` IDs (the development build and the store builds once they
/// exist). No other ID is ever listed: the app refuses to serve an extension
/// whose ID is not in `project.toml` anyway (`app/src/launch.rs`).
pub fn allowed_origins() -> Vec<String> {
    chromium_extension_ids()
        .into_iter()
        .map(|id| format!("chrome-extension://{id}/"))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::Value;

    use super::*;

    fn parse(text: &str) -> Value {
        serde_json::from_str(text).unwrap()
    }

    #[test]
    fn chromium_manifest_lists_origins_and_no_firefox_key() {
        let origins = allowed_origins();
        let manifest =
            parse(&render(Family::Chromium, &PathBuf::from("/opt/x/probe"), &origins).unwrap());
        assert_eq!(manifest["name"], NATIVE_HOST);
        assert_eq!(manifest["type"], "stdio");
        assert_eq!(manifest["path"], "/opt/x/probe");
        assert_eq!(
            manifest["allowed_origins"][0],
            format!("chrome-extension://{}/", websign_project::EXTENSION_DEV_ID)
        );
        assert!(manifest.get("allowed_extensions").is_none());
    }

    #[test]
    fn firefox_manifest_allows_only_the_gecko_id() {
        let manifest =
            parse(&render(Family::Firefox, &PathBuf::from("/opt/x/probe"), &[]).unwrap());
        assert_eq!(manifest["allowed_extensions"], json!([FIREFOX_ID]));
        assert!(manifest.get("allowed_origins").is_none());
    }

    #[test]
    fn origins_are_exactly_the_project_ids() {
        let origins = allowed_origins();
        assert_eq!(origins.len(), chromium_extension_ids().len());
        for id in chromium_extension_ids() {
            assert!(websign_project::is_chromium_extension_id(id));
            assert!(origins.contains(&format!("chrome-extension://{id}/")));
        }
    }

    #[test]
    fn origins_never_contain_wildcards() {
        for origin in allowed_origins() {
            assert!(!origin.contains('*'), "Chrome rejects wildcards: {origin}");
            assert!(origin.ends_with('/'));
        }
    }
}
