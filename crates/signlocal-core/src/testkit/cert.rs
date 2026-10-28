//! The certificate builder.

use super::{CN, UTF8, name, oid, rdn, sequence, spki_rsa, tlv};

/// Builder for a v3 certificate; every field has a sensible default.
#[derive(Debug, Clone)]
pub(crate) struct TestCert {
    version: Option<u8>,
    serial: Vec<u8>,
    issuer: Vec<u8>,
    subject: Vec<u8>,
    not_before: String,
    not_after: String,
    spki: Vec<u8>,
    unique_ids: Vec<Vec<u8>>,
    extensions: Vec<Vec<u8>>,
}

impl TestCert {
    pub(crate) fn new() -> Self {
        Self {
            version: Some(2),
            serial: vec![0x01],
            issuer: name(&[rdn(CN, UTF8, b"Test CA")]),
            subject: name(&[rdn(CN, UTF8, b"Test Subject")]),
            not_before: "240101000000Z".into(),
            not_after: "250101000000Z".into(),
            spki: spki_rsa(2048),
            unique_ids: Vec::new(),
            extensions: Vec::new(),
        }
    }

    /// The `[0] EXPLICIT` version INTEGER, or `None` to omit it (v1).
    pub(crate) fn version(mut self, version: Option<u8>) -> Self {
        self.version = version;
        self
    }

    /// Serial number as INTEGER content octets (include the sign byte yourself).
    pub(crate) fn serial(mut self, content: &[u8]) -> Self {
        self.serial = content.to_vec();
        self
    }

    pub(crate) fn subject(mut self, rdns: &[Vec<u8>]) -> Self {
        self.subject = name(rdns);
        self
    }

    /// The subject as a complete `Name` TLV, for encodings `name` will not produce.
    pub(crate) fn subject_der(mut self, name: Vec<u8>) -> Self {
        self.subject = name;
        self
    }

    pub(crate) fn issuer(mut self, rdns: &[Vec<u8>]) -> Self {
        self.issuer = name(rdns);
        self
    }

    /// `UTCTime` (`YYMMDDHHMMSSZ`) or `GeneralizedTime` (`YYYYMMDDHHMMSSZ`) text.
    pub(crate) fn validity(mut self, not_before: &str, not_after: &str) -> Self {
        self.not_before = not_before.into();
        self.not_after = not_after.into();
        self
    }

    pub(crate) fn spki(mut self, spki: Vec<u8>) -> Self {
        self.spki = spki;
        self
    }

    pub(crate) fn extension(mut self, extension: Vec<u8>) -> Self {
        self.extensions.push(extension);
        self
    }

    /// A raw field between the SPKI and the extensions (issuerUniqueID,
    /// subjectUniqueID, or something that does not belong there).
    pub(crate) fn unique_id(mut self, field: Vec<u8>) -> Self {
        self.unique_ids.push(field);
        self
    }

    pub(crate) fn build(&self) -> Vec<u8> {
        let algorithm = sequence(&[oid("1.2.840.113549.1.1.11"), tlv(0x05, &[])]);
        let mut fields: Vec<Vec<u8>> = self
            .version
            .map(|version| tlv(0xa0, &tlv(0x02, &[version])))
            .into_iter()
            .collect();
        fields.extend([
            tlv(0x02, &self.serial),
            algorithm.clone(),
            self.issuer.clone(),
            sequence(&[time(&self.not_before), time(&self.not_after)]),
            self.subject.clone(),
            self.spki.clone(),
        ]);
        fields.extend(self.unique_ids.iter().cloned());
        if !self.extensions.is_empty() {
            fields.push(tlv(0xa3, &sequence(&self.extensions)));
        }
        sequence(&[sequence(&fields), algorithm, tlv(0x03, &[0x00, 0xaa, 0xbb])])
    }
}

fn time(text: &str) -> Vec<u8> {
    let tag = if text.len() == 13 { 0x17 } else { 0x18 };
    tlv(tag, text.as_bytes())
}
