//! Public key classification and raw key material from `SubjectPublicKeyInfo`.

use der::Decode;
use der::asn1::{AnyRef, ObjectIdentifier, UintRef};
use spki::SubjectPublicKeyInfoOwned;

use super::asn1::malformed;
use super::{CertError, PublicKeyKind};
use crate::ecdsa::Curve;

const RSA_ENCRYPTION: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.1");
const RSA_PSS: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.10");
const EC_PUBLIC_KEY: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.10045.2.1");

/// The two integers of an `RSAPublicKey`, without leading zeros.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RsaComponents<'a> {
    pub modulus: &'a [u8],
    pub exponent: &'a [u8],
}

impl RsaComponents<'_> {
    /// Size of the modulus in bits, so a 2047-bit modulus reports 2047 and
    /// not the 2048 its byte length would suggest.
    fn bits(&self) -> u32 {
        let Some(first) = self.modulus.first() else {
            return 0;
        };
        let full_bytes = self.modulus.len().saturating_sub(1);
        let top_bits = 8 - first.leading_zeros();
        u32::try_from(full_bytes * 8).map_or(u32::MAX, |bits| bits.saturating_add(top_bits))
    }
}

impl PublicKeyKind {
    /// Classifies a certificate's key.
    ///
    /// An EC key whose parameters are not a named curve (explicit curve
    /// parameters, or none) is reported under the algorithm OID, since there
    /// is no curve OID to name.
    ///
    /// SPEC: `Unsupported { oid }` carries the id-ecPublicKey OID in that
    /// case; the SPEC only defines the OID for a named but unknown curve.
    pub(super) fn from_spki(spki: &SubjectPublicKeyInfoOwned) -> Result<Self, CertError> {
        let algorithm = spki.algorithm.oid;
        if algorithm == RSA_ENCRYPTION || algorithm == RSA_PSS {
            let key = rsa_components(spki)?;
            return Ok(Self::Rsa { bits: key.bits() });
        }
        if algorithm == EC_PUBLIC_KEY {
            let Some(curve_oid) = named_curve(spki) else {
                return Ok(Self::Unsupported {
                    oid: algorithm.to_string(),
                });
            };
            let curve_oid = curve_oid.to_string();
            return Ok(match Curve::from_oid(&curve_oid) {
                Some(curve) => Self::Ec { curve },
                None => Self::Unsupported { oid: curve_oid },
            });
        }
        Ok(Self::Unsupported {
            oid: algorithm.to_string(),
        })
    }
}

/// Decodes the `RSAPublicKey` inside `spki`.
pub(crate) fn rsa_components(
    spki: &SubjectPublicKeyInfoOwned,
) -> Result<RsaComponents<'_>, CertError> {
    let key = spki
        .subject_public_key
        .as_bytes()
        .ok_or_else(|| malformed("RSA public key", "bit string has unused bits"))?;
    let (modulus, exponent) = AnyRef::from_der(key)
        .and_then(|any| {
            any.sequence(|reader| {
                let modulus = UintRef::decode(reader)?;
                let exponent = UintRef::decode(reader)?;
                Ok((modulus.as_bytes(), exponent.as_bytes()))
            })
        })
        .map_err(|e| malformed("RSA public key", e))?;
    Ok(RsaComponents { modulus, exponent })
}

/// The SEC1-encoded point of an EC key, or `None` if the bit string is not a
/// whole number of bytes.
pub(crate) fn ec_point(spki: &SubjectPublicKeyInfoOwned) -> Option<&[u8]> {
    spki.subject_public_key.as_bytes()
}

fn named_curve(spki: &SubjectPublicKeyInfoOwned) -> Option<ObjectIdentifier> {
    spki.algorithm
        .parameters
        .as_ref()?
        .decode_as::<ObjectIdentifier>()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn components(modulus: &[u8]) -> RsaComponents<'_> {
        RsaComponents {
            modulus,
            exponent: &[1, 0, 1],
        }
    }

    #[test]
    fn bit_length_counts_from_the_highest_set_bit() {
        assert_eq!(components(&[0x80, 0x00]).bits(), 16);
        assert_eq!(components(&[0x40, 0x00]).bits(), 15);
        assert_eq!(components(&[0x01]).bits(), 1);
        assert_eq!(components(&[0x00]).bits(), 0);
        assert_eq!(components(&[]).bits(), 0);
        let mut m2047 = vec![0x7f];
        m2047.extend([0xff; 255]);
        assert_eq!(components(&m2047).bits(), 2047);
    }
}
