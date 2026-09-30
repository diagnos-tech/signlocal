//! Status from manifest files (Linux and macOS).

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use super::evaluate::evaluate;
use super::{Candidate, RegistrationState, combine};

pub(super) fn state(candidates: &[Candidate], host: &Path) -> RegistrationState {
    combine(candidates.iter().map(|candidate| {
        let expected = candidate.host_copy.as_deref().unwrap_or(host);
        read(&candidate.manifest).map_or_else(
            |state| state,
            |text| evaluate(&text, candidate.family, &candidate.manifest, expected),
        )
    }))
}

/// The manifest's text, or the state its absence or unreadability means.
fn read(path: &Path) -> Result<String, RegistrationState> {
    fs::read_to_string(path).map_err(|error| match error.kind() {
        ErrorKind::NotFound => RegistrationState::Missing,
        _ => RegistrationState::Broken {
            reason: format!("cannot read {}: {error}", path.display()),
        },
    })
}

#[cfg(test)]
mod tests;
