//! The certificate extensions the summary reads.

use super::der::{
    self, BIT_STRING, BOOLEAN, DerError, INTEGER, OBJECT_IDENTIFIER, Reader, SEQUENCE,
};
use super::oid::{
    self, BASIC_CONSTRAINTS, CERTIFICATE_POLICIES, EXTENDED_KEY_USAGE, KEY_USAGE, QC_STATEMENTS,
    SUBJECT_ALT_NAME,
};
use super::san::{self, OtherName};
use super::x509::Extension;
use super::{CertError, KeyUsage, Qualified, malformed};

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
    /// Decodes the known extensions; unknown ones are never looked into.
    ///
    /// A known extension that fails to decode makes the whole certificate
    /// `Malformed`: showing a certificate as signing-capable on the strength
    /// of a half-read KeyUsage would be worse than refusing it. RFC 5280
    /// forbids repeating an extension; if a certificate does, the first copy
    /// is read and the others are ignored.
    pub(super) fn parse(extensions: &[Extension<'_>]) -> Result<Self, CertError> {
        let usage = known(extensions, KEY_USAGE, "KeyUsage", key_usage)?;
        let extended = known(extensions, EXTENDED_KEY_USAGE, "ExtendedKeyUsage", oid_list)?;
        let policies = known(
            extensions,
            CERTIFICATE_POLICIES,
            "CertificatePolicies",
            policy_oids,
        )?;
        let ca = known(extensions, BASIC_CONSTRAINTS, "BasicConstraints", is_ca)?;
        let names = known(
            extensions,
            SUBJECT_ALT_NAME,
            "SubjectAltName",
            san::other_names,
        )?;
        let qualified = known(
            extensions,
            QC_STATEMENTS,
            "qcStatements",
            Qualified::from_extension,
        )?;
        Ok(Self {
            key_usage: usage,
            extended_key_usage: extended.unwrap_or_default(),
            policies: policies.unwrap_or_default(),
            is_ca: ca.unwrap_or(false),
            other_names: names.unwrap_or_default(),
            qualified,
        })
    }
}

/// Decodes the first extension with OID `id`, if the certificate has one.
fn known<T>(
    extensions: &[Extension<'_>],
    id: &[u8],
    name: &'static str,
    decoder: impl FnOnce(&[u8]) -> Result<T, DerError>,
) -> Result<Option<T>, CertError> {
    extensions
        .iter()
        .find(|extension| extension.id == id)
        .map(|extension| decoder(extension.value))
        .transpose()
        .map_err(|e| malformed(name, e))
}

/// KeyUsage BIT STRING. Bits past its end are zero (DER trims them), and a
/// string with no bit set still means "the extension is present".
fn key_usage(value: &[u8]) -> Result<KeyUsage, DerError> {
    let bits = der::bit_string(der::single(value, BIT_STRING)?)?;
    Ok(KeyUsage {
        digital_signature: bits.bit(0),
        non_repudiation: bits.bit(1),
        key_encipherment: bits.bit(2),
        data_encipherment: bits.bit(3),
        key_agreement: bits.bit(4),
        key_cert_sign: bits.bit(5),
        crl_sign: bits.bit(6),
    })
}

/// `SEQUENCE OF OBJECT IDENTIFIER` as dotted text, in order.
fn oid_list(value: &[u8]) -> Result<Vec<String>, DerError> {
    der::elements(der::single(value, SEQUENCE)?, OBJECT_IDENTIFIER)
        .map(|oid| dotted(oid?))
        .collect()
}

/// Policy identifiers only; qualifiers (CPS URI, user notice) are skipped.
fn policy_oids(value: &[u8]) -> Result<Vec<String>, DerError> {
    der::elements(der::single(value, SEQUENCE)?, SEQUENCE)
        .map(|policy| dotted(der::oid_and_optional(policy?)?.0))
        .collect()
}

/// `BasicConstraints ::= SEQUENCE { cA BOOLEAN DEFAULT FALSE, pathLen INTEGER OPTIONAL }`.
fn is_ca(value: &[u8]) -> Result<bool, DerError> {
    let mut fields = Reader::new(der::single(value, SEQUENCE)?);
    let ca = fields
        .read_optional(BOOLEAN)?
        .map(der::boolean)
        .transpose()?;
    fields.read_optional(INTEGER)?;
    fields.finish()?;
    Ok(ca.unwrap_or(false))
}

/// An OID the summary must print; one it cannot is a malformed extension.
fn dotted(content: &[u8]) -> Result<String, DerError> {
    oid::to_dotted(content).ok_or(DerError::Invalid("OBJECT IDENTIFIER"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_usage_bits_follow_rfc_5280_positions() {
        // 1 unused bit; digitalSignature, nonRepudiation, keyCertSign, cRLSign.
        let usage = key_usage(&[0x03, 0x02, 0x01, 0xc6]).unwrap();
        assert_eq!(
            usage,
            KeyUsage {
                digital_signature: true,
                non_repudiation: true,
                key_cert_sign: true,
                crl_sign: true,
                ..KeyUsage::default()
            }
        );
        // No bit at all, and only decipherOnly (bit 8): present, nothing set.
        assert_eq!(key_usage(&[0x03, 0x01, 0x00]), Ok(KeyUsage::default()));
        assert_eq!(
            key_usage(&[0x03, 0x03, 0x07, 0x00, 0x80]),
            Ok(KeyUsage::default())
        );
        assert!(key_usage(&[0x04, 0x01, 0x00]).is_err());
        assert!(key_usage(&[0x03, 0x02, 0x07, 0x80, 0x00]).is_err());
    }

    #[test]
    fn basic_constraints_variants() {
        assert_eq!(is_ca(&[0x30, 0x00]), Ok(false));
        assert_eq!(is_ca(&[0x30, 0x03, 0x01, 0x01, 0xff]), Ok(true));
        assert_eq!(is_ca(&[0x30, 0x03, 0x01, 0x01, 0x01]), Ok(true), "BER true");
        assert_eq!(
            is_ca(&[0x30, 0x03, 0x01, 0x01, 0x00]),
            Ok(false),
            "explicit default"
        );
        assert_eq!(
            is_ca(&[0x30, 0x06, 0x01, 0x01, 0xff, 0x02, 0x01, 0x00]),
            Ok(true)
        );
        assert_eq!(is_ca(&[0x30, 0x03, 0x02, 0x01, 0x00]), Ok(false));
        assert!(is_ca(&[0x30, 0x03, 0x04, 0x01, 0x00]).is_err());
        assert!(is_ca(&[0x30, 0x04, 0x01, 0x02, 0xff, 0xff]).is_err());
        assert!(is_ca(&[0x04, 0x00]).is_err());
    }

    #[test]
    fn oid_lists_keep_order_and_refuse_unprintable_oids() {
        // clientAuth, emailProtection.
        let eku = [
            0x30, 0x14, 0x06, 0x08, 0x2b, 0x06, 0x01, 0x05, 0x05, 0x07, 0x03, 0x02, 0x06, 0x08,
            0x2b, 0x06, 0x01, 0x05, 0x05, 0x07, 0x03, 0x04,
        ];
        assert_eq!(
            oid_list(&eku).unwrap(),
            ["1.3.6.1.5.5.7.3.2", "1.3.6.1.5.5.7.3.4"]
        );
        assert_eq!(oid_list(&[0x30, 0x00]), Ok(vec![]));
        assert!(oid_list(&[0x30, 0x03, 0x06, 0x01, 0x80]).is_err());
    }

    #[test]
    fn policies_skip_qualifiers() {
        // 2.5.29.32.0 with a qualifier SEQUENCE, then 2.5.29.32.1 alone.
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
