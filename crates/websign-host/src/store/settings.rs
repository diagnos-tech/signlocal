//! Preferences the person set in diagnostics.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::StoreError;

/// The settings document.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// PKCS#11 modules added with "Add driver…".
    pub user_modules: Vec<PathBuf>,
    /// "Getting started" hidden.
    pub onboarding_dismissed: bool,
    /// The test signature page reported success once.
    pub test_signature_done: bool,
}

/// Read and change settings.
pub trait SettingsStore {
    fn get(&mut self) -> Result<Settings, StoreError>;
    fn update(&mut self, change: &mut dyn FnMut(&mut Settings)) -> Result<(), StoreError>;
}
