//! ECDSA curves and conversion between the two signature encodings.
//!
//! Signatures leave the app as raw `r || s` (IEEE P1363): that is what the SDK
//! promises and what CNG and PKCS#11 return. macOS returns DER instead.

mod sig_der;
#[cfg(test)]
mod tests;

use sig_der::{push_length, read_tlv};

const TAG_SEQUENCE: u8 = 0x30;
const TAG_INTEGER: u8 = 0x02;

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
        match self {
            Self::P256 => 32,
            Self::P384 => 48,
            Self::P521 => 66,
        }
    }

    /// Size in bytes of a raw `r || s` signature.
    pub const fn signature_len(self) -> usize {
        2 * self.field_len()
    }

    /// NIST name: `"P-256"`, `"P-384"`, `"P-521"`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::P256 => "P-256",
            Self::P384 => "P-384",
            Self::P521 => "P-521",
        }
    }

    /// Curve for a dotted `namedCurve` OID, if supported.
    pub fn from_oid(oid: &str) -> Option<Curve> {
        match oid {
            "1.2.840.10045.3.1.7" => Some(Self::P256),
            "1.3.132.0.34" => Some(Self::P384),
            "1.3.132.0.35" => Some(Self::P521),
            _ => None,
        }
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
    use EcdsaEncodingError::{IntegerTooLarge, Malformed};

    let (body, trailing) = read_tlv(der, TAG_SEQUENCE).ok_or(Malformed)?;
    if !trailing.is_empty() {
        return Err(Malformed);
    }
    let (r, rest) = read_tlv(body, TAG_INTEGER).ok_or(Malformed)?;
    let (s, rest) = read_tlv(rest, TAG_INTEGER).ok_or(Malformed)?;
    if !rest.is_empty() {
        return Err(Malformed);
    }

    // Both integers are validated before their sizes are compared, so a
    // structurally broken signature is always `Malformed`, never "too large".
    let r = positive_magnitude(r)?;
    let s = positive_magnitude(s)?;
    let field_len = curve.field_len();
    if r.len() > field_len || s.len() > field_len {
        return Err(IntegerTooLarge);
    }

    let mut raw = Vec::with_capacity(curve.signature_len());
    push_left_padded(&mut raw, r, field_len);
    push_left_padded(&mut raw, s, field_len);
    Ok(raw)
}

/// Raw `r || s` → minimal DER `ECDSA-Sig-Value`.
///
/// SPEC: an all-zero half is encoded as `02 01 00` rather than rejected;
/// `der_to_raw` refuses such a value, so only that direction is lossy.
pub fn raw_to_der(raw: &[u8], curve: Curve) -> Result<Vec<u8>, EcdsaEncodingError> {
    let wrong_length = || EcdsaEncodingError::WrongLength {
        expected: curve.signature_len(),
        actual: raw.len(),
    };
    if raw.len() != curve.signature_len() {
        return Err(wrong_length());
    }
    let (r, s) = raw
        .split_at_checked(curve.field_len())
        .ok_or_else(wrong_length)?;

    let mut body = Vec::with_capacity(raw.len() + 6);
    push_integer(&mut body, r);
    push_integer(&mut body, s);

    let mut der = Vec::with_capacity(body.len() + 3);
    der.push(TAG_SEQUENCE);
    push_length(&mut der, body.len());
    der.extend_from_slice(&body);
    Ok(der)
}

/// The value of a DER INTEGER without leading zeros, if it is a valid
/// non-negative, non-zero integer.
fn positive_magnitude(content: &[u8]) -> Result<&[u8], EcdsaEncodingError> {
    let first = content.first().ok_or(EcdsaEncodingError::Malformed)?;
    if first & 0x80 != 0 {
        return Err(EcdsaEncodingError::Malformed);
    }
    // All zeros is the value zero, which is not a valid `r` or `s`.
    let start = content
        .iter()
        .position(|&b| b != 0)
        .ok_or(EcdsaEncodingError::Malformed)?;
    content.get(start..).ok_or(EcdsaEncodingError::Malformed)
}

fn push_left_padded(out: &mut Vec<u8>, value: &[u8], len: usize) {
    out.resize(out.len() + len.saturating_sub(value.len()), 0);
    out.extend_from_slice(value);
}

/// Appends `value` (big-endian, any leading zeros) as a minimal DER INTEGER.
fn push_integer(out: &mut Vec<u8>, value: &[u8]) {
    let significant = match value.iter().position(|&b| b != 0) {
        Some(start) => value.get(start..).unwrap_or_default(),
        // Zero is still one byte long.
        None => &[0],
    };
    let needs_sign_byte = significant.first().is_some_and(|b| b & 0x80 != 0);

    out.push(TAG_INTEGER);
    push_length(out, significant.len() + usize::from(needs_sign_byte));
    if needs_sign_byte {
        out.push(0);
    }
    out.extend_from_slice(significant);
}
