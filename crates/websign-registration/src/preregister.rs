//! Extension pre-registration on Windows: `HKCU\Software\Google\Chrome\
//! Extensions\<id>` (and the Edge/Brave equivalents) with the store's
//! `update_url`, so the browser offers "New extension added — Enable" on its
//! next start. Only for extensions already published in that store; a no-op
//! elsewhere.
//!
//! Chrome documents only `HKLM` for this, but its code also reads `HKCU`,
//! which is all a per-user install (or an MSIX package) can write; whether
//! every Chrome build honours it is still unproven
//! (`docs/research/native-messaging.md` §4). Brave reads Chrome's key.

use websign_project::{CHROME_WEB_STORE_ID, EDGE_ADDONS_ID, is_chromium_extension_id};

use crate::destination::Outcome;
use crate::registry::{Hive, Registry};

mod linux;

pub use linux::linux_system_files;

const CHROME_UPDATE_URL: &str = "https://clients2.google.com/service/update2/crx";
const EDGE_UPDATE_URL: &str = "https://edge.microsoft.com/extensionwebstorebase/v1/crx";

/// One pre-registration key.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry {
    label: &'static str,
    subkey: String,
    update_url: &'static str,
}

/// The keys for the given store IDs; an empty ID (not yet published) has
/// none. An ID that is not a Chromium extension ID is ignored too: it becomes
/// part of a registry path, which a stray `\` would redirect.
fn entries(chrome_web_store_id: &str, edge_addons_id: &str) -> Vec<Entry> {
    let mut entries = Vec::new();
    if is_chromium_extension_id(chrome_web_store_id) {
        entries.push(Entry {
            label: "Google Chrome, Brave",
            subkey: format!(r"Software\Google\Chrome\Extensions\{chrome_web_store_id}"),
            update_url: CHROME_UPDATE_URL,
        });
    }
    if is_chromium_extension_id(edge_addons_id) {
        entries.push(Entry {
            label: "Microsoft Edge",
            subkey: format!(r"Software\Microsoft\Edge\Extensions\{edge_addons_id}"),
            update_url: EDGE_UPDATE_URL,
        });
    }
    entries
}

/// Writes (or, with `remove`, deletes) the pre-registration keys for every
/// published store ID.
pub fn apply(remove: bool, dry_run: bool) -> Vec<(String, Outcome)> {
    if !cfg!(windows) {
        return Vec::new();
    }
    let entries = entries(CHROME_WEB_STORE_ID, EDGE_ADDONS_ID);
    apply_entries(&*crate::registry::system(), &entries, remove, dry_run)
}

fn apply_entries(
    registry: &dyn Registry,
    entries: &[Entry],
    remove: bool,
    dry_run: bool,
) -> Vec<(String, Outcome)> {
    entries
        .iter()
        .map(|entry| {
            let what = format!("{}: HKCU\\{}", entry.label, entry.subkey);
            (what, apply_entry(registry, entry, remove, dry_run))
        })
        .collect()
}

fn apply_entry(registry: &dyn Registry, entry: &Entry, remove: bool, dry_run: bool) -> Outcome {
    let result = if remove {
        if !registry.key_exists(Hive::CurrentUser, &entry.subkey) {
            return Outcome::NotPresent;
        }
        if dry_run {
            return Outcome::DryRun;
        }
        registry
            .remove_tree(&entry.subkey)
            .map(|()| Outcome::Removed)
    } else {
        if dry_run {
            return Outcome::DryRun;
        }
        registry
            .set_string(&entry.subkey, "update_url", entry.update_url)
            .map(|()| Outcome::Written)
    };
    result.unwrap_or_else(|error| Outcome::Failed(error.to_string()))
}

#[cfg(test)]
mod tests;
