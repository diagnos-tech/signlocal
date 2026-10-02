//! Which PKCS#11 mechanism signs an already-computed digest, and how to
//! bring the token's answer into the format the SDK promises.
//!
//! The digest is hashed by the caller (a PDF signer), so only "raw" mechanisms
//! qualify: `CKM_SHA256_RSA_PKCS` and friends would hash it a second time and
//! produce a valid signature over the wrong bytes.

use cryptoki::mechanism::rsa::{PkcsMgfType, PkcsPssParams};
use cryptoki::mechanism::{Mechanism, MechanismType};
use cryptoki::types::Ulong;
use probe_core::ecdsa::{self, Curve};
use probe_core::{HashAlgorithm, SignatureAlgorithm, pkcs1};

use crate::keystores::KeystoreError;

/// A mechanism plus the exact bytes to hand to `C_Sign`.
#[derive(Debug)]
pub struct SignPlan {
    pub mechanism: Mechanism<'static>,
    pub input: Vec<u8>,
}

impl SignPlan {
    pub fn mechanism_type(&self) -> MechanismType {
        self.mechanism.mechanism_type()
    }
}

/// Chooses the mechanism for `algorithm` over `digest` (already hashed with `hash`).
pub fn plan(
    hash: HashAlgorithm,
    algorithm: SignatureAlgorithm,
    digest: &[u8],
) -> Result<SignPlan, KeystoreError> {
    let wrong_length =
        |error: probe_core::DigestLengthError| KeystoreError::Other(error.to_string());
    match algorithm {
        SignatureAlgorithm::RsaPkcs1v15 => Ok(SignPlan {
            mechanism: Mechanism::RsaPkcs,
            input: pkcs1::digest_info(hash, digest).map_err(wrong_length)?,
        }),
        SignatureAlgorithm::RsaPss => {
            hash.check_digest(digest).map_err(wrong_length)?;
            let params = PkcsPssParams {
                hash_alg: hash_mechanism(hash),
                mgf: mgf(hash),
                // Salt as long as the digest: what PAdES/CMS profiles expect
                // and what every RSA-PSS implementation accepts.
                s_len: Ulong::new(hash.digest_len() as _),
            };
            Ok(SignPlan {
                mechanism: Mechanism::RsaPkcsPss(params),
                input: digest.to_vec(),
            })
        }
        SignatureAlgorithm::Ecdsa => {
            hash.check_digest(digest).map_err(wrong_length)?;
            Ok(SignPlan {
                mechanism: Mechanism::Ecdsa,
                input: digest.to_vec(),
            })
        }
    }
}

fn hash_mechanism(hash: HashAlgorithm) -> MechanismType {
    match hash {
        HashAlgorithm::Sha256 => MechanismType::SHA256,
        HashAlgorithm::Sha384 => MechanismType::SHA384,
        HashAlgorithm::Sha512 => MechanismType::SHA512,
    }
}

fn mgf(hash: HashAlgorithm) -> PkcsMgfType {
    match hash {
        HashAlgorithm::Sha256 => PkcsMgfType::MGF1_SHA256,
        HashAlgorithm::Sha384 => PkcsMgfType::MGF1_SHA384,
        HashAlgorithm::Sha512 => PkcsMgfType::MGF1_SHA512,
    }
}

/// `CKM_ECDSA` must answer raw `r || s`. A few modules answer DER instead;
/// that is converted, and anything else is rejected rather than passed on.
pub fn ecdsa_signature(raw: Vec<u8>, curve: Curve) -> Result<Vec<u8>, KeystoreError> {
    if raw.len() == curve.signature_len() {
        return Ok(raw);
    }
    if raw.first() == Some(&0x30) {
        return ecdsa::der_to_raw(&raw, curve).map_err(|error| {
            KeystoreError::Other(format!("unreadable DER ECDSA signature: {error}"))
        });
    }
    Err(KeystoreError::Other(format!(
        "the token returned a {}-byte ECDSA signature; {} expected for {}",
        raw.len(),
        curve.signature_len(),
        curve.name()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(hash: HashAlgorithm) -> Vec<u8> {
        hash.digest(b"payload")
    }

    #[test]
    fn pkcs1_signs_the_digest_info_with_the_raw_mechanism() {
        for hash in HashAlgorithm::ALL {
            let digest = digest(hash);
            let plan = plan(hash, SignatureAlgorithm::RsaPkcs1v15, &digest).unwrap();
            assert!(matches!(plan.mechanism, Mechanism::RsaPkcs));
            assert_eq!(plan.input, pkcs1::digest_info(hash, &digest).unwrap());
            assert!(plan.input.len() > digest.len());
        }
    }

    #[test]
    fn pss_signs_the_bare_digest_with_matching_hash_and_salt() {
        for hash in HashAlgorithm::ALL {
            let digest = digest(hash);
            let plan = plan(hash, SignatureAlgorithm::RsaPss, &digest).unwrap();
            assert_eq!(plan.input, digest);
            let Mechanism::RsaPkcsPss(params) = plan.mechanism else {
                panic!("expected CKM_RSA_PKCS_PSS");
            };
            assert_eq!(params.hash_alg, hash_mechanism(hash));
            assert_eq!(params.mgf, mgf(hash));
            assert_eq!(*params.s_len as usize, hash.digest_len());
        }
    }

    #[test]
    fn ecdsa_signs_the_bare_digest() {
        let digest = digest(HashAlgorithm::Sha384);
        let plan = plan(HashAlgorithm::Sha384, SignatureAlgorithm::Ecdsa, &digest).unwrap();
        assert!(matches!(plan.mechanism, Mechanism::Ecdsa));
        assert_eq!(plan.input, digest);
    }

    #[test]
    fn a_digest_of_the_wrong_size_is_refused_for_every_algorithm() {
        for algorithm in SignatureAlgorithm::ALL {
            let error = plan(HashAlgorithm::Sha256, algorithm, &[0; 31]).unwrap_err();
            assert!(error.to_string().contains("must be 32 bytes"), "{error}");
        }
    }

    #[test]
    fn raw_ecdsa_signatures_pass_through() {
        let raw = vec![7; Curve::P384.signature_len()];
        assert_eq!(ecdsa_signature(raw.clone(), Curve::P384).unwrap(), raw);
    }

    #[test]
    fn der_ecdsa_signatures_are_converted() {
        let raw: Vec<u8> = (1..=64).collect();
        let der = ecdsa::raw_to_der(&raw, Curve::P256).unwrap();
        assert_eq!(ecdsa_signature(der, Curve::P256).unwrap(), raw);
    }

    #[test]
    fn other_lengths_are_rejected() {
        let error = ecdsa_signature(vec![1; 63], Curve::P256).unwrap_err();
        assert!(error.to_string().contains("63-byte"), "{error}");
    }
}
