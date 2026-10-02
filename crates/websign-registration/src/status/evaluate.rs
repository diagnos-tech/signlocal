//! Judges one manifest's text against what this app would have written.

use std::path::Path;

use serde_json::Value;
use websign_project::{FIREFOX_ID, NATIVE_HOST, PRODUCT_NAME};

use super::RegistrationState;
use crate::browsers::Family;
use crate::manifest;

/// `manifest_path` resolves a relative `path` (allowed on Windows); `host` is
/// the program the manifest must start.
pub fn evaluate(
    text: &str,
    family: Family,
    manifest_path: &Path,
    host: &Path,
) -> RegistrationState {
    let broken = |reason: &str| RegistrationState::Broken {
        reason: reason.to_owned(),
    };
    let Ok(json) = serde_json::from_str::<Value>(text) else {
        return broken("the manifest is not valid JSON");
    };
    if json.get("name").and_then(Value::as_str) != Some(NATIVE_HOST) {
        return broken("the manifest names another host");
    }
    if !allows_our_extension(&json, family) {
        return broken(&format!(
            "the manifest does not allow the {PRODUCT_NAME} extension"
        ));
    }
    let Some(path) = json.get("path").and_then(Value::as_str) else {
        return broken("the manifest has no program path");
    };
    let resolved = match manifest_path.parent() {
        Some(folder) if !Path::new(path).has_root() => folder.join(path),
        _ => Path::new(path).to_owned(),
    };
    if same_path(&resolved, host) {
        RegistrationState::Registered
    } else {
        RegistrationState::PointsElsewhere {
            path: path.to_owned(),
        }
    }
}

fn allows_our_extension(json: &Value, family: Family) -> bool {
    let (key, ours): (&str, Vec<String>) = match family {
        Family::Chromium => ("allowed_origins", manifest::allowed_origins()),
        Family::Firefox => ("allowed_extensions", vec![FIREFOX_ID.to_owned()]),
    };
    json.get(key)
        .and_then(Value::as_array)
        .is_some_and(|listed| {
            listed
                .iter()
                .filter_map(Value::as_str)
                .any(|entry| ours.iter().any(|id| id == entry))
        })
}

/// Windows paths compare without case and accept either separator, as the
/// file system does.
fn same_path(a: &Path, b: &Path) -> bool {
    if cfg!(windows) {
        let normal = |path: &Path| path.to_string_lossy().replace('/', "\\").to_lowercase();
        normal(a) == normal(b)
    } else {
        a == b
    }
}

#[cfg(test)]
mod tests;
