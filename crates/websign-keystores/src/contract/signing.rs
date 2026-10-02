//! Checks on `sign`, `capabilities` and `chain` for one key:
//! `signs-every-combination`, `advertised-algorithms-sign`,
//! `rejects-wrong-length`, `vanished-key`, `chain-best-effort`.

use std::panic::{self, AssertUnwindSafe};

use websign_core::{CertInfo, HashAlgorithm, PublicKeyKind, SignatureAlgorithm};

use super::support::{any_algorithm, fixture_pin, sign, sign_digest};
use super::{Fixture, Report};
use crate::{FoundKey, Keystore, KeystoreError};

/// More issuers than this is a loop or a bug (`SPEC.md` §3.4, §4.4, §5.3).
const MAX_CHAIN: usize = 8;

pub fn check(keystore: &mut dyn Keystore, key: &FoundKey, fixture: &Fixture, report: &mut Report) {
    let pin = fixture_pin(key, fixture);
    every_combination(keystore, key, pin.as_deref(), report);
    wrong_length(keystore, key, pin.as_deref(), report);
    vanished(keystore, key, pin.as_deref(), report);
    chain(keystore, key, report);
}

fn every_combination(
    keystore: &mut dyn Keystore,
    key: &FoundKey,
    pin: Option<&str>,
    report: &mut Report,
) {
    const CHECK: &str = "signs-every-combination";
    let info = match CertInfo::from_der(&key.cert_der) {
        Ok(info) => info,
        Err(error) => {
            return report.fail(CHECK, Some(key), format!("unreadable certificate: {error}"));
        }
    };
    let algorithms: Vec<SignatureAlgorithm> = SignatureAlgorithm::ALL
        .into_iter()
        .filter(|&algorithm| info.key.supports(algorithm))
        .collect();
    if algorithms.is_empty() {
        return report.fail(
            CHECK,
            Some(key),
            format!("no algorithm for key {:?}", info.key),
        );
    }
    let advertised = keystore.capabilities(key);
    if !algorithms
        .iter()
        .any(|&algorithm| advertised.supports(algorithm))
    {
        report.fail(
            "advertised-algorithms-sign",
            Some(key),
            "the store advertises nothing this key can do",
        );
    }
    let verifiable = match info.key {
        PublicKeyKind::Ec { curve } => curve.has_verifier(),
        _ => true,
    };
    for hash in HashAlgorithm::ALL {
        for &algorithm in &algorithms {
            let (digest, signed) = sign(keystore, key, hash, algorithm, pin);
            let label = format!("{hash} {algorithm}");
            match signed {
                Ok(signature) if verifiable => {
                    let verified = websign_core::verify(
                        &key.cert_der,
                        hash,
                        algorithm,
                        &digest,
                        &signature.bytes,
                    );
                    if let Err(error) = verified {
                        report.fail(
                            CHECK,
                            Some(key),
                            format!("{label}: does not verify: {error}"),
                        );
                    }
                }
                Ok(_) => {}
                // What the host offers must sign (`SPEC.md` §1 rule 10).
                Err(error) if advertised.supports(algorithm) => report.fail(
                    "advertised-algorithms-sign",
                    Some(key),
                    format!("{label} is advertised but fails: {error}"),
                ),
                // A store may refuse what it does not advertise (PSS through
                // legacy CAPI, `SPEC.md` §4.2), but only as `Unsupported`.
                Err(KeystoreError::Unsupported(_)) => {}
                Err(error) => report.fail(CHECK, Some(key), format!("{label}: {error}")),
            }
        }
    }
}

fn wrong_length(
    keystore: &mut dyn Keystore,
    key: &FoundKey,
    pin: Option<&str>,
    report: &mut Report,
) {
    let Some(algorithm) = any_algorithm(key) else {
        return;
    };
    let short = [0x5a; 31];
    let signed = sign_digest(keystore, key, HashAlgorithm::Sha256, algorithm, &short, pin);
    if signed.is_ok() {
        report.fail(
            "rejects-wrong-length",
            Some(key),
            "a 31-byte SHA-256 digest was signed",
        );
    }
}

/// A key whose certificate left the source (simulated by altering the
/// certificate the key was listed with) must fail as missing.
fn vanished(keystore: &mut dyn Keystore, key: &FoundKey, pin: Option<&str>, report: &mut Report) {
    let Some(algorithm) = any_algorithm(key) else {
        return;
    };
    let mut gone = key.clone();
    if let Some(last) = gone.cert_der.last_mut() {
        *last ^= 0xff;
    }
    let (_, signed) = sign(keystore, &gone, HashAlgorithm::Sha256, algorithm, pin);
    match signed {
        Err(KeystoreError::NotFound | KeystoreError::TokenRemoved) => {}
        Ok(_) => report.fail(
            "vanished-key",
            Some(key),
            "signed for a certificate it does not hold",
        ),
        Err(error) => report.fail(
            "vanished-key",
            Some(key),
            format!("expected NotFound, got {error}"),
        ),
    }
}

fn chain(keystore: &mut dyn Keystore, key: &FoundKey, report: &mut Report) {
    const CHECK: &str = "chain-best-effort";
    let chain = panic::catch_unwind(AssertUnwindSafe(|| keystore.chain(key)));
    let Ok(chain) = chain else {
        return report.fail(CHECK, Some(key), "chain panicked");
    };
    if chain.contains(&key.cert_der) {
        report.fail(CHECK, Some(key), "the leaf is part of its own chain");
    }
    if chain.len() > MAX_CHAIN {
        report.fail(
            CHECK,
            Some(key),
            format!("{} issuers (at most {MAX_CHAIN})", chain.len()),
        );
    }
}
