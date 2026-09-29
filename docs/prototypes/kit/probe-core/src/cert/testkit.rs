//! Hand-assembled DER certificates for unit tests.
//!
//! Real certificates would pin the tests to whatever a CA happened to issue;
//! building the DER by hand lets each test change exactly one field. The
//! signature is a placeholder because nothing here checks certificate
//! signatures.

use der::asn1::ObjectIdentifier;

/// One TLV with a definite length (short form, or long form up to 64 KiB).
pub(super) fn tlv(tag: u8, content: &[u8]) -> Vec<u8> {
    let mut out = vec![tag];
    match content.len() {
        n if n < 0x80 => out.push(n as u8),
        n if n < 0x100 => out.extend([0x81, n as u8]),
        n => out.extend([0x82, (n >> 8) as u8, n as u8]),
    }
    out.extend_from_slice(content);
    out
}

pub(super) fn sequence(parts: &[Vec<u8>]) -> Vec<u8> {
    tlv(0x30, &parts.concat())
}

/// A whole OBJECT IDENTIFIER TLV from dotted text.
pub(super) fn oid(dotted: &str) -> Vec<u8> {
    tlv(0x06, ObjectIdentifier::new_unwrap(dotted).as_bytes())
}

/// A Name from already-built RDNs.
pub(super) fn name(rdns: &[Vec<u8>]) -> Vec<u8> {
    sequence(rdns)
}

/// A single-attribute RDN with the given string type tag.
pub(super) fn rdn(attribute_oid: &str, string_tag: u8, value: &[u8]) -> Vec<u8> {
    tlv(
        0x31,
        &sequence(&[oid(attribute_oid), tlv(string_tag, value)]),
    )
}

pub(super) const UTF8: u8 = 0x0c;
pub(super) const PRINTABLE: u8 = 0x13;
pub(super) const TELETEX: u8 = 0x14;
pub(super) const IA5: u8 = 0x16;
pub(super) const BMP: u8 = 0x1e;

pub(super) const CN: &str = "2.5.4.3";
pub(super) const ORG: &str = "2.5.4.10";
pub(super) const OU: &str = "2.5.4.11";
pub(super) const COUNTRY: &str = "2.5.4.6";

fn extension(extension_oid: &str, critical: bool, value: &[u8]) -> Vec<u8> {
    let mut parts = vec![oid(extension_oid)];
    if critical {
        parts.push(tlv(0x01, &[0xff]));
    }
    parts.push(tlv(0x04, value));
    sequence(&parts)
}

/// A KeyUsage extension with the given bit positions set (0 = digitalSignature).
pub(super) fn key_usage(bits: &[usize]) -> Vec<u8> {
    let mut bytes = [0u8; 2];
    for &bit in bits {
        bytes[bit / 8] |= 0x80 >> (bit % 8);
    }
    let used = bits.iter().max().map_or(1, |max| max + 1);
    let byte_len = used.div_ceil(8);
    let unused = (byte_len * 8 - used) as u8;
    let mut content = vec![unused];
    content.extend_from_slice(&bytes[..byte_len]);
    extension("2.5.29.15", true, &tlv(0x03, &content))
}

pub(super) fn basic_constraints(ca: bool) -> Vec<u8> {
    let body = if ca { tlv(0x01, &[0xff]) } else { Vec::new() };
    extension("2.5.29.19", true, &tlv(0x30, &body))
}

pub(super) fn extended_key_usage(oids: &[&str]) -> Vec<u8> {
    let list: Vec<Vec<u8>> = oids.iter().map(|o| oid(o)).collect();
    extension("2.5.29.37", false, &sequence(&list))
}

/// CertificatePolicies with one PolicyInformation per OID, the first carrying
/// a CPS qualifier so qualifier skipping is exercised.
pub(super) fn policies(oids: &[&str]) -> Vec<u8> {
    let list: Vec<Vec<u8>> = oids
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let mut parts = vec![oid(p)];
            if i == 0 {
                let cps = sequence(&[
                    oid("1.3.6.1.5.5.7.2.1"),
                    tlv(IA5, b"http://example.test/dpc"),
                ]);
                parts.push(sequence(&[cps]));
            }
            sequence(&parts)
        })
        .collect();
    extension("2.5.29.32", false, &sequence(&list))
}

/// SubjectAltName made of the given `otherName` entries: (type-id, value TLV).
pub(super) fn san_other_names(entries: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let names: Vec<Vec<u8>> = entries
        .iter()
        .map(|(type_id, value)| tlv(0xa0, &[oid(type_id), tlv(0xa0, value)].concat()))
        .collect();
    extension("2.5.29.17", false, &sequence(&names))
}

/// qcStatements from (statement OID, optional info TLV).
pub(super) fn qc_statements(statements: &[(&str, Option<Vec<u8>>)]) -> Vec<u8> {
    let list: Vec<Vec<u8>> = statements
        .iter()
        .map(|(id, info)| {
            let mut parts = vec![oid(id)];
            parts.extend(info.clone());
            sequence(&parts)
        })
        .collect();
    extension("1.3.6.1.5.5.7.1.3", false, &sequence(&list))
}

/// An extension of any type, for unknown-extension and malformed cases.
pub(super) fn raw_extension(extension_oid: &str, value: &[u8]) -> Vec<u8> {
    extension(extension_oid, false, value)
}

pub(super) fn spki_rsa(modulus_bits: usize) -> Vec<u8> {
    let bytes = modulus_bits.div_ceil(8);
    let top_bits = modulus_bits - (bytes - 1) * 8;
    let mut modulus = vec![0xff; bytes];
    modulus[0] = 0xffu8 >> (8 - top_bits);
    let mut content = Vec::new();
    // A leading zero keeps the INTEGER positive when the top bit is set.
    if modulus[0] & 0x80 != 0 {
        content.push(0);
    }
    content.extend(modulus);
    let key = sequence(&[tlv(0x02, &content), tlv(0x02, &[0x01, 0x00, 0x01])]);
    let algorithm = sequence(&[oid("1.2.840.113549.1.1.1"), tlv(0x05, &[])]);
    sequence(&[algorithm, tlv(0x03, &[&[0u8][..], &key].concat())])
}

/// An id-ecPublicKey SPKI naming `curve_oid`, with a dummy uncompressed point.
pub(super) fn spki_ec(curve_oid: &str) -> Vec<u8> {
    let algorithm = sequence(&[oid("1.2.840.10045.2.1"), oid(curve_oid)]);
    let point = [&[0x04u8][..], &[0x11; 64]].concat();
    sequence(&[algorithm, tlv(0x03, &[&[0u8][..], &point].concat())])
}

/// An SPKI for an arbitrary algorithm OID, without parameters.
pub(super) fn spki_other(algorithm_oid: &str) -> Vec<u8> {
    let algorithm = sequence(&[oid(algorithm_oid)]);
    sequence(&[algorithm, tlv(0x03, &[0x00, 0x01, 0x02, 0x03])])
}

/// Builder for a v3 certificate; every field has a sensible default.
#[derive(Debug, Clone)]
pub(super) struct TestCert {
    serial: Vec<u8>,
    issuer: Vec<u8>,
    subject: Vec<u8>,
    not_before: String,
    not_after: String,
    spki: Vec<u8>,
    extensions: Vec<Vec<u8>>,
}

impl TestCert {
    pub(super) fn new() -> Self {
        Self {
            serial: vec![0x01],
            issuer: name(&[rdn(CN, UTF8, b"Test CA")]),
            subject: name(&[rdn(CN, UTF8, b"Test Subject")]),
            not_before: "240101000000Z".into(),
            not_after: "250101000000Z".into(),
            spki: spki_rsa(2048),
            extensions: Vec::new(),
        }
    }

    /// Serial number as INTEGER content octets (include the sign byte yourself).
    pub(super) fn serial(mut self, content: &[u8]) -> Self {
        self.serial = content.to_vec();
        self
    }

    pub(super) fn subject(mut self, rdns: &[Vec<u8>]) -> Self {
        self.subject = name(rdns);
        self
    }

    pub(super) fn issuer(mut self, rdns: &[Vec<u8>]) -> Self {
        self.issuer = name(rdns);
        self
    }

    /// `UTCTime` (`YYMMDDHHMMSSZ`) or `GeneralizedTime` (`YYYYMMDDHHMMSSZ`) text.
    pub(super) fn validity(mut self, not_before: &str, not_after: &str) -> Self {
        self.not_before = not_before.into();
        self.not_after = not_after.into();
        self
    }

    pub(super) fn spki(mut self, spki: Vec<u8>) -> Self {
        self.spki = spki;
        self
    }

    pub(super) fn extension(mut self, extension: Vec<u8>) -> Self {
        self.extensions.push(extension);
        self
    }

    pub(super) fn build(&self) -> Vec<u8> {
        let algorithm = sequence(&[oid("1.2.840.113549.1.1.11"), tlv(0x05, &[])]);
        let mut fields = vec![
            tlv(0xa0, &tlv(0x02, &[0x02])),
            tlv(0x02, &self.serial),
            algorithm.clone(),
            self.issuer.clone(),
            sequence(&[time(&self.not_before), time(&self.not_after)]),
            self.subject.clone(),
            self.spki.clone(),
        ];
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
