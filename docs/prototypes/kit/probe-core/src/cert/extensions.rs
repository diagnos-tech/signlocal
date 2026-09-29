//! The certificate extensions the summary reads.

use der::asn1::{AnyRef, BitStringRef, ObjectIdentifier};
use der::{Decode, Reader, Tag, Tagged};
use x509_cert::ext::Extension;

use super::asn1::{malformed, oid_with_optional_info};
use super::san::{self, OtherName};
use super::{CertError, KeyUsage, Qualified};

const KEY_USAGE: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.29.15");
const SUBJECT_ALT_NAME: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.29.17");
const BASIC_CONSTRAINTS: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.29.19");
const CERTIFICATE_POLICIES: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.29.32");
const EXTENDED_KEY_USAGE: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.29.37");
const QC_STATEMENTS: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.3.6.1.5.5.7.1.3");

/// Everything read from the extensions block, before it is folded into a
/// `CertInfo`.
#[derive(Debug, Default)]
pub(super) struct Extensions {
    pub key_usage: Option<KeyUsage>,
    pub extended_key_usage: Vec<String>,
    pub policies: Vec<String>,
    pub is_ca: bool,
    pub other_names: Vec<OtherName>,
    pub qualified: Option<Qualified>,
}

impl Extensions {
    /// Decodes the known extensions; unknown ones are ignored.
    ///
    /// A known extension that fails to decode makes the whole certificate
    /// `Malformed`: showing a certificate as signing-capable on the strength
    /// of a half-read KeyUsage would be worse than refusing it. If an
    /// extension is repeated (which RFC 5280 forbids) the first copy wins.
    ///
    /// SPEC: repeated extensions are not an error; the first copy is used.
    pub(super) fn parse(extensions: Option<&[Extension]>) -> Result<Self, CertError> {
        let extensions = extensions.unwrap_or_default();
        let find = |oid: ObjectIdentifier| {
            extensions
                .iter()
                .find(|ext| ext.extn_id == oid)
                .map(|ext| ext.extn_value.as_bytes())
        };

        let mut parsed = Self::default();
        if let Some(value) = find(KEY_USAGE) {
            parsed.key_usage = Some(key_usage(value).map_err(|e| malformed("KeyUsage", e))?);
        }
        if let Some(value) = find(EXTENDED_KEY_USAGE) {
            parsed.extended_key_usage =
                extended_key_usage(value).map_err(|e| malformed("ExtendedKeyUsage", e))?;
        }
        if let Some(value) = find(CERTIFICATE_POLICIES) {
            parsed.policies =
                policy_oids(value).map_err(|e| malformed("CertificatePolicies", e))?;
        }
        if let Some(value) = find(BASIC_CONSTRAINTS) {
            parsed.is_ca = is_ca(value).map_err(|e| malformed("BasicConstraints", e))?;
        }
        if let Some(value) = find(SUBJECT_ALT_NAME) {
            parsed.other_names =
                san::other_names(value).map_err(|e| malformed("SubjectAltName", e))?;
        }
        if let Some(value) = find(QC_STATEMENTS) {
            parsed.qualified =
                Some(Qualified::from_extension(value).map_err(|e| malformed("qcStatements", e))?);
        }
        Ok(parsed)
    }
}

/// Reads the KeyUsage bit string. Bits past the end of a (DER-minimal, hence
/// trimmed) bit string are zero.
fn key_usage(value: &[u8]) -> der::Result<KeyUsage> {
    let bits = BitStringRef::from_der(value)?;
    let bit = |position: usize| bits.get(position).unwrap_or(false);
    Ok(KeyUsage {
        digital_signature: bit(0),
        non_repudiation: bit(1),
        key_encipherment: bit(2),
        data_encipherment: bit(3),
        key_agreement: bit(4),
        key_cert_sign: bit(5),
        crl_sign: bit(6),
    })
}

fn extended_key_usage(value: &[u8]) -> der::Result<Vec<String>> {
    Ok(Vec::<ObjectIdentifier>::from_der(value)?
        .iter()
        .map(ToString::to_string)
        .collect())
}

/// Policy identifiers only; qualifiers (CPS URI, user notice) are skipped.
fn policy_oids(value: &[u8]) -> der::Result<Vec<String>> {
    Vec::<AnyRef>::from_der(value)?
        .into_iter()
        .map(|policy| Ok(oid_with_optional_info(policy)?.0.to_string()))
        .collect()
}

/// `BasicConstraints ::= SEQUENCE { cA BOOLEAN DEFAULT FALSE, pathLen INTEGER OPTIONAL }`.
fn is_ca(value: &[u8]) -> der::Result<bool> {
    AnyRef::from_der(value)?.sequence(|reader| {
        let ca = if !reader.is_finished() && Tag::peek(reader)? == Tag::Boolean {
            bool::decode(reader)?
        } else {
            false
        };
        if !reader.is_finished() {
            AnyRef::decode(reader)?.tag().assert_eq(Tag::Integer)?;
        }
        Ok(ca)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_usage_bits_follow_rfc_5280_positions() {
        // BIT STRING, 1 unused bit: digitalSignature + nonRepudiation + keyCertSign + cRLSign.
        // bits 0,1,5,6 -> 1100 0110 = 0xc6 (bit 7 unused).
        let usage = key_usage(&[0x03, 0x02, 0x01, 0xc6]).unwrap();
        assert!(usage.digital_signature);
        assert!(usage.non_repudiation);
        assert!(!usage.key_encipherment);
        assert!(!usage.data_encipherment);
        assert!(!usage.key_agreement);
        assert!(usage.key_cert_sign);
        assert!(usage.crl_sign);
    }

    #[test]
    fn key_usage_short_bit_strings_read_missing_bits_as_zero() {
        // Only digitalSignature, bit string trimmed to one bit.
        let usage = key_usage(&[0x03, 0x02, 0x07, 0x80]).unwrap();
        assert_eq!(
            usage,
            KeyUsage {
                digital_signature: true,
                ..KeyUsage::default()
            }
        );
        // Two-byte bit string with decipherOnly (bit 8) set: none of ours.
        let usage = key_usage(&[0x03, 0x03, 0x07, 0x00, 0x80]).unwrap();
        assert_eq!(usage, KeyUsage::default());
    }

    #[test]
    fn key_usage_rejects_other_types() {
        assert!(key_usage(&[0x04, 0x01, 0x00]).is_err());
        assert!(key_usage(&[]).is_err());
    }

    #[test]
    fn basic_constraints_variants() {
        assert!(!is_ca(&[0x30, 0x00]).unwrap());
        assert!(is_ca(&[0x30, 0x03, 0x01, 0x01, 0xff]).unwrap());
        assert!(is_ca(&[0x30, 0x06, 0x01, 0x01, 0xff, 0x02, 0x01, 0x00]).unwrap());
        // Only a path length: cA stays false.
        assert!(!is_ca(&[0x30, 0x03, 0x02, 0x01, 0x00]).unwrap());
        assert!(is_ca(&[0x30, 0x03, 0x01, 0x01, 0x05]).is_err());
        assert!(is_ca(&[0x30, 0x03, 0x04, 0x01, 0x00]).is_err());
        assert!(is_ca(&[0x04, 0x00]).is_err());
    }

    #[test]
    fn eku_and_policies_keep_order() {
        // clientAuth (1.3.6.1.5.5.7.3.2), emailProtection (1.3.6.1.5.5.7.3.4)
        let eku = [
            0x30, 0x14, 0x06, 0x08, 0x2b, 0x06, 0x01, 0x05, 0x05, 0x07, 0x03, 0x02, 0x06, 0x08,
            0x2b, 0x06, 0x01, 0x05, 0x05, 0x07, 0x03, 0x04,
        ];
        assert_eq!(
            extended_key_usage(&eku).unwrap(),
            ["1.3.6.1.5.5.7.3.2", "1.3.6.1.5.5.7.3.4"]
        );

        // Two policies: 2.5.29.32.0 with a qualifier SEQUENCE, then 2.5.29.32.1 alone.
        let policies = [
            0x30, 0x15, 0x30, 0x0b, 0x06, 0x04, 0x55, 0x1d, 0x20, 0x00, 0x30, 0x03, 0x02, 0x01,
            0x01, 0x30, 0x06, 0x06, 0x04, 0x55, 0x1d, 0x20, 0x01,
        ];
        assert_eq!(
            policy_oids(&policies).unwrap(),
            ["2.5.29.32.0", "2.5.29.32.1"]
        );
    }
}
