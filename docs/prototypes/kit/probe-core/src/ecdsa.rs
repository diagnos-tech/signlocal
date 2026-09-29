//! ECDSA curves and conversion between the two signature encodings.
//!
//! Signatures leave the app as raw `r || s` (IEEE P1363): that is what the SDK
//! promises and what CNG and PKCS#11 return. macOS returns DER instead.

/// Named curves accepted for ECDSA keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Curve {
    P256,
    P384,
    P521,
}

impl Curve {
    /// Size in bytes of one coordinate or scalar: 32, 48 or 66.
    pub const fn field_len(self) -> usize {
        todo!()
    }

    /// Size in bytes of a raw `r || s` signature.
    pub const fn signature_len(self) -> usize {
        todo!()
    }

    /// NIST name: `"P-256"`, `"P-384"`, `"P-521"`.
    pub const fn name(self) -> &'static str {
        todo!()
    }

    /// Curve for a dotted `namedCurve` OID, if supported.
    pub fn from_oid(oid: &str) -> Option<Curve> {
        todo!()
    }
}

/// Why an ECDSA signature could not be re-encoded.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EcdsaEncodingError {
    #[error("malformed ECDSA signature")]
    Malformed,
    #[error("ECDSA integer does not fit the curve")]
    IntegerTooLarge,
    #[error("raw ECDSA signature must be {expected} bytes, got {actual}")]
    WrongLength { expected: usize, actual: usize },
}

/// DER `ECDSA-Sig-Value` → raw `r || s`, left-padded to the curve size.
///
/// Tolerates non-minimal integers (extra leading zeros), which some tokens
/// emit; rejects anything else that is not strict DER.
pub fn der_to_raw(der: &[u8], curve: Curve) -> Result<Vec<u8>, EcdsaEncodingError> {
    todo!()
}

/// Raw `r || s` → minimal DER `ECDSA-Sig-Value`.
pub fn raw_to_der(raw: &[u8], curve: Curve) -> Result<Vec<u8>, EcdsaEncodingError> {
    todo!()
}
