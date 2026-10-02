//! Which certificates never reach the list (`docs/ux.md` §5.8).

use websign_core::{CertInfo, PublicKeyKind};

use super::candidate::CertCandidate;
use super::row::HiddenReason;

const EKU_SERVER_AUTH: &str = "1.3.6.1.5.5.7.3.1";
const EKU_CODE_SIGNING: &str = "1.3.6.1.5.5.7.3.3";
const EKU_TIME_STAMPING: &str = "1.3.6.1.5.5.7.3.8";
const EKU_OCSP_SIGNING: &str = "1.3.6.1.5.5.7.3.9";

/// Purposes that say nothing about signing documents.
const FOREIGN_PURPOSES: [&str; 4] = [
    EKU_SERVER_AUTH,
    EKU_CODE_SIGNING,
    EKU_TIME_STAMPING,
    EKU_OCSP_SIGNING,
];

/// The first reason `candidate` must stay out of the list, if any. `all` is
/// every candidate of the listing, to spot login siblings.
pub(super) fn hidden_reason(
    candidate: &CertCandidate,
    all: &[CertCandidate],
) -> Option<HiddenReason> {
    let Ok(info) = &candidate.info else {
        return Some(HiddenReason::Unparseable);
    };
    if !candidate.has_private_key {
        return Some(HiddenReason::NoPrivateKey);
    }
    if info.is_ca {
        return Some(HiddenReason::CertificateAuthority);
    }
    if !info.can_sign() {
        return Some(HiddenReason::KeyUsage);
    }
    if only_foreign_purposes(info) {
        return Some(HiddenReason::ExtendedKeyUsage);
    }
    if matches!(info.key, PublicKeyKind::Unsupported { .. }) {
        return Some(HiddenReason::UnsupportedKey);
    }
    is_login_sibling(candidate, info, all).then_some(HiddenReason::LoginSibling)
}

fn only_foreign_purposes(info: &CertInfo) -> bool {
    !info.extended_key_usage.is_empty()
        && info
            .extended_key_usage
            .iter()
            .all(|oid| FOREIGN_PURPOSES.contains(&oid.as_str()))
}

/// A login certificate (digitalSignature only) that shares holder and device
/// with a certificate carrying nonRepudiation: choosing it would produce a
/// non-qualified signature by mistake.
fn is_login_sibling(candidate: &CertCandidate, info: &CertInfo, all: &[CertCandidate]) -> bool {
    let login_only = info
        .key_usage
        .is_some_and(|usage| usage.digital_signature && !usage.non_repudiation);
    login_only
        && all.iter().any(|other| {
            other.fingerprint != candidate.fingerprint
                && other.device == candidate.device
                && other.info.as_ref().is_ok_and(|other_info| {
                    other_info.subject == info.subject
                        && other_info
                            .key_usage
                            .is_some_and(|usage| usage.non_repudiation)
                })
        })
}
