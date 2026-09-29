//! The native messaging host manifest: the small JSON file a browser reads to
//! learn which program to start and which extensions may talk to it.

use std::path::Path;

use anyhow::{Context as _, bail};
use serde_json::json;

use super::browsers::Family;
use crate::config::{
    CHROME_WEB_STORE_ID, EDGE_ADDONS_ID, EXTENSION_DEV_ID, FIREFOX_ID, NATIVE_HOST, PRODUCT_NAME,
};
use crate::nm::is_chromium_extension_id;

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

/// The `chrome-extension://<id>/` origins the manifest allows: the
/// development build, the store builds once they exist, and `extra`.
pub fn allowed_origins(extra: &[String]) -> anyhow::Result<Vec<String>> {
    let mut ids: Vec<&str> = vec![EXTENSION_DEV_ID];
    ids.extend(
        [CHROME_WEB_STORE_ID, EDGE_ADDONS_ID]
            .into_iter()
            .filter(|id| !id.is_empty()),
    );
    ids.extend(extra.iter().map(String::as_str));
    let mut origins = Vec::new();
    for id in ids {
        if !is_chromium_extension_id(id) {
            bail!("{id:?} is not a Chromium extension ID (32 letters from a to p)");
        }
        let origin = format!("chrome-extension://{id}/");
        if !origins.contains(&origin) {
            origins.push(origin);
        }
    }
    Ok(origins)
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
        let origins = allowed_origins(&[]).unwrap();
        let manifest =
            parse(&render(Family::Chromium, &PathBuf::from("/opt/x/probe"), &origins).unwrap());
        assert_eq!(manifest["name"], NATIVE_HOST);
        assert_eq!(manifest["type"], "stdio");
        assert_eq!(manifest["path"], "/opt/x/probe");
        assert_eq!(
            manifest["allowed_origins"][0],
            format!("chrome-extension://{EXTENSION_DEV_ID}/")
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
    fn extra_ids_are_added_once_and_validated() {
        let extra = "abcdefghijklmnopabcdefghijklmnop".to_owned();
        let origins = allowed_origins(&[extra.clone(), extra.clone()]).unwrap();
        assert_eq!(origins.iter().filter(|o| o.contains(&extra)).count(), 1);
        assert!(allowed_origins(&["nope".to_owned()]).is_err());
        assert!(
            allowed_origins(&[EXTENSION_DEV_ID.to_owned()])
                .unwrap()
                .len()
                == origins.len() - 1
        );
    }

    #[test]
    fn origins_never_contain_wildcards() {
        for origin in allowed_origins(&[]).unwrap() {
            assert!(!origin.contains('*'), "Chrome rejects wildcards: {origin}");
            assert!(origin.ends_with('/'));
        }
    }
}
