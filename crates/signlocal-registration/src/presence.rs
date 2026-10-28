//! When a browser counts as installed for registration. A manifest goes
//! into a browser's folder only when that folder exists, so a machine
//! without the browser is not littered, or when the browser is detected
//! (its app bundle on macOS, its executable on Linux): a browser installed
//! but never started has no folder yet, and a person who installs the app
//! first must still find it working when they start the browser.

use std::path::PathBuf;

use crate::destination::{Location, Target};

/// Lifts the folder requirement of every target that waits for one of
/// `roots` (the folders of the browsers detected as installed).
pub fn waive(mut targets: Vec<Target>, roots: &[PathBuf]) -> Vec<Target> {
    for target in &mut targets {
        if let Location::File { requires, .. } = &mut target.location
            && requires.as_ref().is_some_and(|root| roots.contains(root))
        {
            *requires = None;
        }
    }
    targets
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::browsers::Family;

    fn requirement(target: &Target) -> Option<&Path> {
        match &target.location {
            Location::File { requires, .. } => requires.as_deref(),
            Location::Registry { .. } => None,
        }
    }

    #[test]
    fn only_targets_waiting_for_a_detected_root_are_freed() {
        let brave = Path::new("/u/.config/BraveSoftware/Brave-Browser");
        let beta = Path::new("/u/.config/BraveSoftware/Brave-Browser-Beta");
        let hosts = Path::new("/u/.config/x/NativeMessagingHosts");
        let targets = vec![
            Target::file("Brave", Family::Chromium, hosts).requiring(brave),
            Target::file("Brave Beta", Family::Chromium, hosts).requiring(beta),
            Target::file("unconditional", Family::Chromium, hosts),
        ];
        let waived = waive(targets, &[brave.to_owned()]);
        let required: Vec<_> = waived.iter().map(requirement).collect();
        assert_eq!(required, [None, Some(beta), None]);
    }
}
