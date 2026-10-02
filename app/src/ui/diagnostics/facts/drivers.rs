//! Token drivers (PKCS#11 modules) and what loading each gave
//! (`docs/ux.md` §8.4 "Token drivers").
//!
//! The key stores name a loaded module only by its file name, so the full
//! path is known for the ones the person added; a failure message carries
//! the path it tried.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use websign_keystores::SourceFailure;
use websign_keystores::inventory::Inventory;

use super::DriverFact;
use super::home::{anonymize, anonymize_text};

const PREFIX: &str = "pkcs11:";
const NOT_FOUND: &str = "module file not found: ";

/// Every module the scan tried, loaded ones first, in the key stores' order.
pub fn collect(
    inventory: &Inventory,
    user_modules: &[PathBuf],
    home: Option<&Path>,
) -> Vec<DriverFact> {
    let mut drivers: Vec<DriverFact> = inventory
        .opened
        .keystores
        .iter()
        .filter_map(|keystore| keystore.name().strip_prefix(PREFIX).map(str::to_owned))
        .map(|name| {
            let setting = user_module(&name, user_modules);
            DriverFact {
                path: setting
                    .as_deref()
                    .map_or_else(|| name.clone(), |path| display(path, home)),
                result: Ok(tokens_of(inventory, &name)),
                user_added: setting.is_some(),
                setting,
                name,
            }
        })
        .collect();
    drivers.extend(
        inventory
            .opened
            .failures
            .iter()
            .filter_map(|failure| failed(failure, user_modules, home)),
    );
    drivers
}

fn failed(
    failure: &SourceFailure,
    user_modules: &[PathBuf],
    home: Option<&Path>,
) -> Option<DriverFact> {
    let name = failure.source.strip_prefix(PREFIX)?.to_owned();
    let setting = user_module(&name, user_modules);
    let (path, reason) = match failure.error.strip_prefix(NOT_FOUND) {
        Some(path) => (anonymize(path, home), "file not found".to_owned()),
        None => (
            setting
                .as_deref()
                .map_or_else(|| name.clone(), |path| display(path, home)),
            crate::logging::redact_line(&anonymize_text(&failure.error, home)),
        ),
    };
    Some(DriverFact {
        name,
        path,
        result: Err(reason),
        user_added: setting.is_some(),
        setting,
    })
}

/// The added module with this file name, if the person added it.
fn user_module(name: &str, user_modules: &[PathBuf]) -> Option<PathBuf> {
    user_modules
        .iter()
        .find(|path| {
            path.file_name()
                .is_some_and(|file| file.to_string_lossy() == name)
        })
        .cloned()
}

/// Tokens of this module that brought certificates, told apart by their
/// device link (the key stores do not report slot counts).
fn tokens_of(inventory: &Inventory, name: &str) -> u32 {
    let keystore = format!("{PREFIX}{name}");
    let devices: BTreeSet<String> = inventory
        .entries
        .iter()
        .filter(|entry| entry.key.keystore == keystore)
        .map(|entry| format!("{:?}", entry.key.device))
        .collect();
    u32::try_from(devices.len()).unwrap_or(u32::MAX)
}

fn display(path: &Path, home: Option<&Path>) -> String {
    anonymize(&path.to_string_lossy(), home)
}
