//! Public key classification and the raw key material the verifier needs.

use super::der::{self, DerError, INTEGER, OBJECT_IDENTIFIER, Reader, SEQUENCE};
use super::oid::{self, EC_PUBLIC_KEY, RSA_ENCRYPTION, RSASSA_PSS};
use super::x509::SubjectPublicKeyInfo;
use super::{CertError, PublicKeyKind, malformed};
use crate::ecdsa::Curve;

/// A certificate's public key, borrowed from its DER.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PublicKey<'a> {
    Rsa(RsaComponents<'a>),
    Ec { curve: Curve, point: &'a [u8] },
    Unsupported { oid: String },
}

/// The two integers of an `RSAPublicKey`, as unsigned big-endian magnitudes
/// without leading zeros (empty for zero).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RsaComponents<'a> {
    pub modulus: &'a [u8],
    pub exponent: &'a [u8],
}

impl<'a> PublicKey<'a> {
    /// Classifies the key of `spki`.
    ///
    /// Only the RSA and EC keys this crate can use are decoded. The point of
    /// an EC key is not validated here: that needs curve arithmetic, which
    /// only [`crate::verify()`] does.
    pub(super) fn from_spki(spki: &SubjectPublicKeyInfo<'a>) -> Result<Self, CertError> {
        let bytes = || {
            spki.key
                .whole_bytes()
                .ok_or_else(|| malformed("public key", "bit string is not whole bytes"))
        };
        match spki.algorithm {
            RSA_ENCRYPTION | RSASSA_PSS => {
                let components =
                    rsa_components(bytes()?).map_err(|e| malformed("RSA public key", e))?;
                Ok(Self::Rsa(components))
            }
            EC_PUBLIC_KEY => Ok(match named_curve(spki) {
                Ok(curve) => Self::Ec {
                    curve,
                    point: bytes()?,
                },
                Err(oid) => Self::Unsupported { oid },
            }),
            other => {
                let oid = oid::to_dotted(other)
                    .ok_or_else(|| malformed("public key algorithm", "unreadable OID"))?;
                Ok(Self::Unsupported { oid })
            }
        }
    }

    /// The public classification shown in [`super::CertInfo`].
    pub(super) fn kind(&self) -> PublicKeyKind {
        match self {
            Self::Rsa(components) => PublicKeyKind::Rsa {
                bits: components.bits(),
            },
            Self::Ec { curve, .. } => PublicKeyKind::Ec { curve: *curve },
            Self::Unsupported { oid } => PublicKeyKind::Unsupported { oid: oid.clone() },
        }
    }
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
        u32::try_from(full_bytes)
            .ok()
            .and_then(|bytes| bytes.checked_mul(8))
            .map_or(u32::MAX, |bits| bits.saturating_add(top_bits))
    }
}

/// `RSAPublicKey ::= SEQUENCE { modulus INTEGER, publicExponent INTEGER }`.
///
/// The integers are read as unsigned magnitudes: some smart cards drop the
/// sign byte of a modulus whose top bit is set, which DER would read as
/// negative, and pad others with redundant zeros. Neither changes the key.
fn rsa_components(key: &[u8]) -> Result<RsaComponents<'_>, DerError> {
    let mut fields = Reader::new(der::single(key, SEQUENCE)?);
    let modulus = magnitude(fields.read(INTEGER)?)?;
    let exponent = magnitude(fields.read(INTEGER)?)?;
    fields.finish()?;
    Ok(RsaComponents { modulus, exponent })
}

fn magnitude(content: &[u8]) -> Result<&[u8], DerError> {
    if content.is_empty() {
        return Err(DerError::Invalid("INTEGER"));
    }
    let start = content
        .iter()
        .position(|&b| b != 0)
        .unwrap_or(content.len());
    Ok(content.get(start..).unwrap_or_default())
}

/// The curve named by the EC parameters, or the OID to report the key under
/// when there is no supported named curve: the curve's own OID when it is
/// named but unknown, id-ecPublicKey otherwise (explicit parameters,
/// `implicitlyCA`, or none), since there is no curve OID to name.
fn named_curve(spki: &SubjectPublicKeyInfo<'_>) -> Result<Curve, String> {
    let algorithm = || "1.2.840.10045.2.1".to_owned();
    let parameters = spki
        .parameters
        .filter(|p| p.tag == OBJECT_IDENTIFIER)
        .ok_or_else(algorithm)?;
    let curve_oid = oid::to_dotted(parameters.content).ok_or_else(algorithm)?;
    Curve::from_oid(&curve_oid).ok_or(curve_oid)
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
        assert_eq!(components(&[]).bits(), 0);
        let mut m2047 = vec![0x7f];
        m2047.extend([0xff; 255]);
        assert_eq!(components(&m2047).bits(), 2047);
    }

    #[test]
    fn rsa_integers_are_read_as_unsigned_magnitudes() {
        // Modulus 00 00 c1 (redundant zeros), exponent 81 (no sign byte).
        let key = [0x30, 0x08, 0x02, 0x03, 0x00, 0x00, 0xc1, 0x02, 0x01, 0x81];
        assert_eq!(
            rsa_components(&key),
            Ok(RsaComponents {
                modulus: &[0xc1],
                exponent: &[0x81]
            })
        );
        assert_eq!(magnitude(&[0x00]), Ok(&[][..]));
        assert!(magnitude(&[]).is_err());
        let three_integers = [
            0x30, 0x09, 0x02, 0x01, 0x05, 0x02, 0x01, 0x03, 0x02, 0x01, 0x01,
        ];
        assert!(rsa_components(&three_integers).is_err());
    }
}
