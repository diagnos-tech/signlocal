//! Linux external-extension files, for the deb/rpm packages to install.
//!
//! Chrome, Chromium and Edge read `<id>.json` from their system
//! `extensions` folders and install the store extension on their next start.
//! Writing there needs root, so the packaging places these files; the app
//! only provides their paths and contents.

use std::path::PathBuf;

use serde_json::json;
use websign_project::{CHROME_WEB_STORE_ID, EDGE_ADDONS_ID, is_chromium_extension_id};

use super::{CHROME_UPDATE_URL, EDGE_UPDATE_URL};

/// `(path, contents)` of every external-extension file for the published
/// store IDs; empty while none is published. Malformed IDs are ignored, as
/// they would name a file outside the `extensions` folder.
pub fn linux_system_files() -> Vec<(PathBuf, String)> {
    files(CHROME_WEB_STORE_ID, EDGE_ADDONS_ID)
}

pub(super) fn files(chrome_web_store_id: &str, edge_addons_id: &str) -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();
    let mut add = |dir: &str, id: &str, update_url: &str| {
        let text = format!("{}\n", json!({ "external_update_url": update_url }));
        files.push((PathBuf::from(dir).join(format!("{id}.json")), text));
    };
    if is_chromium_extension_id(chrome_web_store_id) {
        add(
            "/usr/share/google-chrome/extensions",
            chrome_web_store_id,
            CHROME_UPDATE_URL,
        );
        add(
            "/usr/share/chromium/extensions",
            chrome_web_store_id,
            CHROME_UPDATE_URL,
        );
    }
    if is_chromium_extension_id(edge_addons_id) {
        add(
            "/usr/share/microsoft-edge/extensions",
            edge_addons_id,
            EDGE_UPDATE_URL,
        );
    }
    files
}
