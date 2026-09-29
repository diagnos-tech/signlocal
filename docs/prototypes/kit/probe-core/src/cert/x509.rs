//! Locates the fields of an X.509 certificate (RFC 5280 §4.1) without
//! interpreting them.

use super::der::{
    self, BIT_STRING, BOOLEAN, BitString, DerError, INTEGER, OBJECT_IDENTIFIER, OCTET_STRING,
    Reader, SEQUENCE, Tlv, explicit, implicit,
};
use super::{CertError, malformed};

/// The parts of a certificate the summary reads, borrowed from its DER.
#[derive(Debug)]
pub(super) struct Certificate<'a> {
    /// INTEGER content octets, as encoded.
    pub serial: &'a [u8],
    /// `Name` contents (the RDN sequence).
    pub issuer: &'a [u8],
    pub subject: &'a [u8],
    /// `Time` values, still encoded.
    pub not_before: Tlv<'a>,
    pub not_after: Tlv<'a>,
    pub public_key: SubjectPublicKeyInfo<'a>,
    pub extensions: Vec<Extension<'a>>,
}

#[derive(Debug)]
pub(super) struct SubjectPublicKeyInfo<'a> {
    /// OBJECT IDENTIFIER content of the key algorithm.
    pub algorithm: &'a [u8],
    pub parameters: Option<Tlv<'a>>,
    pub key: BitString<'a>,
}

#[derive(Debug)]
pub(super) struct Extension<'a> {
    /// OBJECT IDENTIFIER content of `extnID`.
    pub id: &'a [u8],
    /// Content of the `extnValue` OCTET STRING: the extension's own DER.
    pub value: &'a [u8],
}

impl<'a> Certificate<'a> {
    /// Splits `der`, which must be exactly one `Certificate`.
    ///
    /// The signature algorithm and value are only checked for shape: the
    /// summary never verifies a certificate's signature.
    pub(super) fn locate(der: &'a [u8]) -> Result<Self, CertError> {
        let at = |what: &'static str| move |error: DerError| malformed(what, error);

        let certificate = der::single(der, SEQUENCE).map_err(at("certificate"))?;
        let mut signed = Reader::new(certificate);
        let tbs = signed.read(SEQUENCE).map_err(at("TBSCertificate"))?;
        signed.read(SEQUENCE).map_err(at("signature algorithm"))?;
        signed.read(BIT_STRING).map_err(at("signature"))?;
        signed.finish().map_err(at("certificate"))?;

        let mut fields = Reader::new(tbs);
        if let Some(version) = fields.read_optional(explicit(0)).map_err(at("version"))? {
            der::single(version, INTEGER).map_err(at("version"))?;
        }
        let serial = fields.read(INTEGER).map_err(at("serial number"))?;
        if serial.is_empty() {
            return Err(malformed("serial number", DerError::Invalid("INTEGER")));
        }
        fields
            .read(SEQUENCE)
            .map_err(at("TBS signature algorithm"))?;
        let issuer = fields.read(SEQUENCE).map_err(at("issuer"))?;
        let (not_before, not_after) = fields
            .read(SEQUENCE)
            .and_then(validity)
            .map_err(at("validity"))?;
        let subject = fields.read(SEQUENCE).map_err(at("subject"))?;
        let public_key = fields
            .read(SEQUENCE)
            .and_then(subject_public_key_info)
            .map_err(at("subject public key info"))?;
        fields
            .read_optional(implicit(1))
            .map_err(at("issuerUniqueID"))?;
        fields
            .read_optional(implicit(2))
            .map_err(at("subjectUniqueID"))?;
        let extensions = match fields
            .read_optional(explicit(3))
            .map_err(at("extensions"))?
        {
            Some(wrapper) => extensions(wrapper).map_err(at("extensions"))?,
            None => Vec::new(),
        };
        fields.finish().map_err(at("TBSCertificate"))?;

        Ok(Self {
            serial,
            issuer,
            subject,
            not_before,
            not_after,
            public_key,
            extensions,
        })
    }
}

fn validity(content: &[u8]) -> Result<(Tlv<'_>, Tlv<'_>), DerError> {
    let mut times = Reader::new(content);
    let not_before = times.read_any()?;
    let not_after = times.read_any()?;
    times.finish()?;
    Ok((not_before, not_after))
}

fn subject_public_key_info(content: &[u8]) -> Result<SubjectPublicKeyInfo<'_>, DerError> {
    let mut fields = Reader::new(content);
    let (algorithm, parameters) = der::oid_and_optional(fields.read(SEQUENCE)?)?;
    let key = der::bit_string(fields.read(BIT_STRING)?)?;
    fields.finish()?;
    Ok(SubjectPublicKeyInfo {
        algorithm,
        parameters,
        key,
    })
}

/// `[3] EXPLICIT SEQUENCE OF Extension`, where
/// `Extension ::= SEQUENCE { extnID OID, critical BOOLEAN DEFAULT FALSE, extnValue OCTET STRING }`.
fn extensions(wrapper: &[u8]) -> Result<Vec<Extension<'_>>, DerError> {
    der::elements(der::single(wrapper, SEQUENCE)?, SEQUENCE)
        .map(|extension| {
            let mut fields = Reader::new(extension?);
            let id = fields.read(OBJECT_IDENTIFIER)?;
            // Criticality does not matter to a summary; only its shape is checked.
            fields
                .read_optional(BOOLEAN)?
                .map(der::boolean)
                .transpose()?;
            let value = fields.read(OCTET_STRING)?;
            fields.finish()?;
            Ok(Extension { id, value })
        })
        .collect()
}
