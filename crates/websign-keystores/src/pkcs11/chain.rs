//! `chain`: the CA certificates stored on the same token as the key
//! (`SPEC.md` §3.4). ICP-Brasil and eIDAS tokens usually ship the issuing CA
//! and the root next to the holder's certificate, which saves the caller a
//! network fetch.
//!
//! Only what the token holds is used, and nothing is validated: the result is
//! a hint for the caller's own path building.

use websign_core::CertInfo;

use super::cert_names::issuer_and_subject;

/// More than any real hierarchy (ICP-Brasil has three levels), and a bound
/// on what a token with a certificate loop could make us walk.
pub const MAX_ISSUERS: usize = 8;

/// Issuers of `leaf` among `stored`, nearest first, the leaf itself excluded.
/// Only CA certificates count (`basicConstraints cA`); the walk stops at a
/// self-signed certificate, a missing issuer or [`MAX_ISSUERS`].
pub fn issuers(leaf: &[u8], stored: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let authorities: Vec<&Vec<u8>> = stored
        .iter()
        .filter(|der| der.as_slice() != leaf && is_authority(der))
        .collect();
    let mut chain: Vec<Vec<u8>> = Vec::new();
    let mut current = leaf;
    while chain.len() < MAX_ISSUERS {
        let Some((issuer, subject)) = issuer_and_subject(current) else {
            break;
        };
        if issuer == subject {
            break;
        }
        let next = authorities.iter().find(|candidate| {
            issuer_and_subject(candidate).is_some_and(|(_, name)| name == issuer)
                && !chain.contains(candidate)
        });
        let Some(next) = next else {
            break;
        };
        chain.push((*next).clone());
        current = next.as_slice();
    }
    chain
}

fn is_authority(der: &[u8]) -> bool {
    CertInfo::from_der(der).is_ok_and(|info| info.is_ca)
}
