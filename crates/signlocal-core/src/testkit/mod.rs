//! Hand-assembled DER certificates for unit tests.
//!
//! Real certificates would pin the tests to whatever a CA happened to issue;
//! building the DER by hand lets each test change exactly one field. The
//! signature is a placeholder because nothing here checks certificate
//! signatures.

mod cert;
mod extensions;
mod keys;

pub(crate) use cert::TestCert;
pub(crate) use extensions::*;
pub(crate) use keys::*;

/// One TLV with a definite length (short form, or long form up to 64 KiB).
pub(crate) fn tlv(tag: u8, content: &[u8]) -> Vec<u8> {
    let mut out = vec![tag];
    match content.len() {
        n if n < 0x80 => out.push(n as u8),
        n if n < 0x100 => out.extend([0x81, n as u8]),
        n => out.extend([0x82, (n >> 8) as u8, n as u8]),
    }
    out.extend_from_slice(content);
    out
}

pub(crate) fn sequence(parts: &[Vec<u8>]) -> Vec<u8> {
    tlv(0x30, &parts.concat())
}

/// A whole OBJECT IDENTIFIER TLV from dotted text (arcs up to `u128`).
pub(crate) fn oid(dotted: &str) -> Vec<u8> {
    let arcs: Vec<u128> = dotted.split('.').map(|arc| arc.parse().unwrap()).collect();
    let first = arcs[0] * 40 + arcs[1];
    let mut content = Vec::new();
    for arc in std::iter::once(first).chain(arcs[2..].iter().copied()) {
        let mut groups = vec![(arc & 0x7f) as u8];
        let mut rest = arc >> 7;
        while rest > 0 {
            groups.push((rest & 0x7f) as u8 | 0x80);
            rest >>= 7;
        }
        content.extend(groups.iter().rev());
    }
    tlv(0x06, &content)
}

/// A Name from already-built RDNs.
pub(crate) fn name(rdns: &[Vec<u8>]) -> Vec<u8> {
    sequence(rdns)
}

/// A single-attribute RDN with the given string type tag.
pub(crate) fn rdn(attribute_oid: &str, string_tag: u8, value: &[u8]) -> Vec<u8> {
    tlv(
        0x31,
        &sequence(&[oid(attribute_oid), tlv(string_tag, value)]),
    )
}

/// A BIT STRING of whole bytes.
pub(crate) fn bit_string(bytes: &[u8]) -> Vec<u8> {
    tlv(0x03, &[&[0u8][..], bytes].concat())
}

pub(crate) const UTF8: u8 = 0x0c;
pub(crate) const PRINTABLE: u8 = 0x13;
pub(crate) const TELETEX: u8 = 0x14;
pub(crate) const IA5: u8 = 0x16;
pub(crate) const BMP: u8 = 0x1e;

pub(crate) const CN: &str = "2.5.4.3";
pub(crate) const ORG: &str = "2.5.4.10";
pub(crate) const OU: &str = "2.5.4.11";
pub(crate) const COUNTRY: &str = "2.5.4.6";
