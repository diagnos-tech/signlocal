//! Certificate to wire summary: what `list` tells the extension.

use probe_core::{CertInfo, SignatureAlgorithm};

use super::backend::CertificateSummary;
use crate::commands::view::key_label;
use crate::keystores::{FoundKey, PinPrompt};

/// Describes `info`, reached through `found`, for the extension.
pub fn summarize(info: &CertInfo, found: &FoundKey, paths: usize) -> CertificateSummary {
    let (kind, level) = classify(info);
    CertificateSummary {
        fingerprint: info.fingerprint.to_hex(),
        display_name: info.display_name(),
        kind: kind.to_owned(),
        level,
        key: key_label(&info.key),
        origin: found.keystore.clone(),
        provider: found.provider.clone(),
        hardware: found.hardware,
        pin: match found.pin {
            PinPrompt::System => "system",
            PinPrompt::App {
                protected_path: true,
            } => "pin-pad",
            PinPrompt::App {
                protected_path: false,
            } => "app",
        },
        can_sign: info.can_sign(),
        algorithms: SignatureAlgorithm::ALL
            .into_iter()
            .filter(|&algorithm| info.key.supports(algorithm))
            .map(SignatureAlgorithm::name)
            .collect(),
        not_before: info.not_before,
        not_after: info.not_after,
        paths,
    }
}

/// `(kind, ICP-Brasil level)`. ICP-Brasil wins over eIDAS when both apply,
/// because the level is what a Brazilian relying party checks.
fn classify(info: &CertInfo) -> (&'static str, Option<String>) {
    if let Some(icp) = &info.icp_brasil {
        return ("icp-brasil", icp.level.map(|level| level.to_string()));
    }
    if info.qualified.as_ref().is_some_and(|q| q.compliance) {
        return ("eidas-qualified", None);
    }
    ("certificate", None)
}
