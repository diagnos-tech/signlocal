//! What is checked before a PKCS#11 key is asked to sign, so that a request
//! the key cannot serve fails with `Unsupported` instead of a token error.

use cryptoki::context::Pkcs11;
use cryptoki::error::{Error, RvError};
use cryptoki::mechanism::MechanismType;
use cryptoki::object::KeyType;
use cryptoki::slot::Slot;
use websign_core::{CertInfo, Curve, PublicKeyKind, SignatureAlgorithm};

use crate::KeystoreError;

/// For ECDSA, the curve from the certificate (needed to validate the raw
/// signature's size); `None` for RSA.
pub fn expected_curve(
    algorithm: SignatureAlgorithm,
    cert_der: &[u8],
) -> Result<Option<Curve>, KeystoreError> {
    if algorithm != SignatureAlgorithm::Ecdsa {
        return Ok(None);
    }
    let info =
        CertInfo::from_der(cert_der).map_err(|error| KeystoreError::Other(error.to_string()))?;
    match info.key {
        PublicKeyKind::Ec { curve } => Ok(Some(curve)),
        other => Err(KeystoreError::Unsupported(format!(
            "ECDSA needs an EC key, the certificate has {other:?}"
        ))),
    }
}

/// `Unsupported` when the token says it cannot sign with the mechanism. A
/// token that cannot even describe it is given the benefit of the doubt: some
/// modules implement mechanisms they do not report.
pub fn require_mechanism(
    pkcs11: &Pkcs11,
    slot: Slot,
    mechanism: MechanismType,
) -> Result<(), KeystoreError> {
    match pkcs11.get_mechanism_info(slot, mechanism) {
        Ok(info) if info.sign() => Ok(()),
        Ok(_) | Err(Error::Pkcs11(RvError::MechanismInvalid, _)) => Err(
            KeystoreError::Unsupported(format!("the token cannot sign with {mechanism}")),
        ),
        Err(_) => Ok(()),
    }
}

/// `Unsupported` when the private key's type does not fit the algorithm.
/// A key that does not state its type is let through; the token decides.
pub fn require_key_type(
    key_type: Option<KeyType>,
    algorithm: SignatureAlgorithm,
) -> Result<(), KeystoreError> {
    let (expected, article) = if algorithm.is_rsa() {
        (KeyType::RSA, "an RSA")
    } else {
        (KeyType::EC, "an EC")
    };
    match key_type {
        Some(actual) if actual != expected => Err(KeystoreError::Unsupported(format!(
            "{algorithm} needs {article} key"
        ))),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rsa_algorithms_refuse_ec_keys_and_the_other_way_round() {
        let error = require_key_type(Some(KeyType::EC), SignatureAlgorithm::RsaPss);
        assert!(matches!(error, Err(KeystoreError::Unsupported(text)) if text.contains("an RSA")));
        let error = require_key_type(Some(KeyType::RSA), SignatureAlgorithm::Ecdsa);
        assert!(matches!(error, Err(KeystoreError::Unsupported(text)) if text.contains("an EC")));
    }

    #[test]
    fn matching_or_unstated_key_types_pass() {
        assert!(require_key_type(Some(KeyType::RSA), SignatureAlgorithm::RsaPkcs1v15).is_ok());
        assert!(require_key_type(None, SignatureAlgorithm::Ecdsa).is_ok());
    }

    #[test]
    fn rsa_needs_no_curve_and_garbage_cannot_give_one() {
        assert!(matches!(
            expected_curve(SignatureAlgorithm::RsaPss, b"garbage"),
            Ok(None)
        ));
        assert!(expected_curve(SignatureAlgorithm::Ecdsa, b"garbage").is_err());
    }
}
