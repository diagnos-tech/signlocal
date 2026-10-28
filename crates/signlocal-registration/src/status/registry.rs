//! Status on Windows: the first `HKCU` key the browser finds decides, and its
//! value must name an existing manifest.

use std::fs;
use std::path::{Path, PathBuf};

use super::RegistrationState;
use super::evaluate::evaluate;
use crate::browsers::Browser;
use crate::registry::{Hive, Registry};
use crate::windows;

pub(super) fn state(browser: Browser, registry: &dyn Registry, host: &Path) -> RegistrationState {
    for subkey in windows::lookup_order(browser) {
        let value = match registry.get_string(Hive::CurrentUser, &subkey, "") {
            Ok(Some(value)) => value,
            Ok(None) => continue,
            Err(error) => {
                return RegistrationState::Broken {
                    reason: format!("cannot read HKCU\\{subkey}: {error}"),
                };
            }
        };
        let manifest = PathBuf::from(value);
        return match fs::read_to_string(&manifest) {
            Ok(text) => evaluate(&text, browser.family(), &manifest, host),
            Err(_) => RegistrationState::Broken {
                reason: format!(
                    "HKCU\\{subkey} points to {}, which cannot be read",
                    manifest.display()
                ),
            },
        };
    }
    RegistrationState::Missing
}

#[cfg(test)]
mod tests;
