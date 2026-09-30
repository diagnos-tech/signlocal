//! A candidate as the caller receives it.

use websign_core::present::holder::display_name;
use websign_core::present::wire::{algorithm_name, certificate_profile, key_description};
use websign_protocol::limits::MAX_CHAIN_LEN;
use websign_protocol::types::{Base64Bytes, Certificate, FingerprintHex};
use websign_ui_model::certs::CertCandidate;

use crate::ports::KeySnapshot;

/// The wire certificate of `candidate`. `None` when its DER is missing from
/// the snapshot, it did not parse, or its key type has no wire description:
/// none of those can be offered for signing.
pub(super) fn wire_certificate(
    snapshot: &KeySnapshot,
    candidate: &CertCandidate,
    chain: &[Vec<u8>],
) -> Option<Certificate> {
    let der = snapshot.certificates.get(&candidate.fingerprint)?;
    let info = candidate.info.as_ref().ok()?;
    let issuer_name = info
        .issuer
        .common_name
        .as_ref()
        .or(info.issuer.organization.as_ref())
        .cloned()
        .unwrap_or_default();
    Some(Certificate {
        der: Base64Bytes::new(der.clone()),
        chain: wire_chain(chain),
        fingerprint: FingerprintHex::new(candidate.fingerprint.to_hex()).ok()?,
        display_name: display_name(info),
        issuer_name,
        not_before: info.not_before,
        not_after: info.not_after,
        key: key_description(&info.key)?,
        algorithms: candidate
            .algorithms
            .iter()
            .copied()
            .map(algorithm_name)
            .collect(),
        profile: certificate_profile(info, candidate.hardware),
    })
}

/// An issuer chain as the wire carries it: at most `MAX_CHAIN_LEN`
/// certificates, the rest dropped (it is best effort anyway).
pub(super) fn wire_chain(chain: &[Vec<u8>]) -> Vec<Base64Bytes> {
    chain
        .iter()
        .take(MAX_CHAIN_LEN)
        .cloned()
        .map(Base64Bytes::new)
        .collect()
}
