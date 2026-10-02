//! Helpers shared by the contract checks.

use secrecy::SecretString;
use websign_core::{Fingerprint, HashAlgorithm, SignatureAlgorithm};

use super::Fixture;
use crate::{FoundKey, Keystore, KeystoreError, PinPrompt, SignRequest, Signature};

/// The bytes every contract signature covers.
const MESSAGE: &[u8] = b"websign keystore contract";

/// How a key is named in violations: a fingerprint prefix, never the
/// holder's name.
pub fn short_name(key: &FoundKey) -> String {
    let hex = fingerprint_hex(key);
    format!("{}… ({})", hex.get(..12).unwrap_or(&hex), key.keystore)
}

pub fn fingerprint_hex(key: &FoundKey) -> String {
    Fingerprint::of(&key.cert_der).to_hex()
}

/// Which PIN a signature needs: the fixture's for keys whose PIN the app
/// collects, none when the OS or a PIN pad collects it.
pub fn fixture_pin(key: &FoundKey, fixture: &Fixture) -> Option<String> {
    match key.pin {
        PinPrompt::App {
            protected_path: false,
        } => fixture.pin.clone(),
        _ => None,
    }
}

/// A signature of [`MESSAGE`] hashed with `hash`; returns the digest too so
/// the caller can verify.
pub fn sign(
    keystore: &mut dyn Keystore,
    key: &FoundKey,
    hash: HashAlgorithm,
    algorithm: SignatureAlgorithm,
    pin: Option<&str>,
) -> (Vec<u8>, Result<Signature, KeystoreError>) {
    let digest = hash.digest(MESSAGE);
    let result = sign_digest(keystore, key, hash, algorithm, &digest, pin);
    (digest, result)
}

pub fn sign_digest(
    keystore: &mut dyn Keystore,
    key: &FoundKey,
    hash: HashAlgorithm,
    algorithm: SignatureAlgorithm,
    digest: &[u8],
    pin: Option<&str>,
) -> Result<Signature, KeystoreError> {
    let pin = pin.map(|pin| SecretString::from(pin.to_owned()));
    let request = SignRequest {
        hash,
        algorithm,
        digest,
        pin: pin.as_ref(),
        parent_window: None,
    };
    keystore.sign(key, &request)
}

/// Any algorithm the certificate's key supports, for checks that need just one.
pub fn any_algorithm(key: &FoundKey) -> Option<SignatureAlgorithm> {
    let info = websign_core::CertInfo::from_der(&key.cert_der).ok()?;
    SignatureAlgorithm::ALL
        .into_iter()
        .find(|&algorithm| algorithm != SignatureAlgorithm::RsaPss && info.key.supports(algorithm))
}
