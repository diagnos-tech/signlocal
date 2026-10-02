//! PKCS#11 modules: p11-kit registrations, known vendor paths and modules
//! given on the command line. One keystore per loaded module.
//!
//! Discovery finds candidate files ([`discovery`]); each is loaded once
//! ([`module`]) and becomes a [`Pkcs11Keystore`]. A module that fails to load
//! is reported as a [`SourceFailure`] and never stops the others.

mod always_authenticate;
mod ckr;
mod discovery;
mod errors;
mod file_id;
mod finder;
mod keystore;
mod known_paths;
mod listing;
mod locator;
mod login;
mod mechanism;
mod module;
mod objects;
mod p11kit;
mod path_patterns;
mod provider;
mod signing;

use std::path::Path;

use keystore::Pkcs11Keystore;

use super::{Opened, Options, SourceFailure};
use crate::trace::trace;

/// Loads every PKCS#11 module that can be found and adds one keystore per module.
pub fn open(options: &Options, opened: &mut Opened) {
    trace!(
        "PKCS#11 discovery (p11-kit: {}, known paths: {}, --module: {})",
        !options.no_p11_kit,
        !options.no_known_modules,
        options.extra_modules.len()
    );
    let candidates = discovery::discover(options);
    trace!("PKCS#11 candidates: {}", candidates.len());
    for candidate in candidates {
        let file_name = file_name(&candidate.path);
        if !candidate.path.exists() {
            if candidate.reportable {
                opened.failures.push(SourceFailure {
                    source: format!("pkcs11:{file_name}"),
                    error: format!("module file not found: {}", candidate.path.display()),
                });
            }
            continue;
        }
        trace!("loading PKCS#11 module {file_name} (C_Initialize)");
        match module::load(&candidate.path) {
            Ok(pkcs11) => opened.keystores.push(Box::new(Pkcs11Keystore::new(
                pkcs11,
                file_name,
                candidate.path.clone(),
            ))),
            Err(error) => {
                let error = match candidate.known {
                    Some(known) => format!("{error} [{} {}]", known.vendor, known.product),
                    None => error,
                };
                opened.failures.push(SourceFailure {
                    source: format!("pkcs11:{file_name}"),
                    error,
                });
            }
        }
    }
}

/// The name people know the module by, as it was found (not its symlink target).
fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}
