//! Extension builders, each producing one whole `Extension` SEQUENCE.

use super::{IA5, oid, sequence, tlv};

fn extension(extension_oid: &str, critical: bool, value: &[u8]) -> Vec<u8> {
    extension_with_criticality(extension_oid, critical.then_some(0xff), value)
}

/// An extension whose `critical` BOOLEAN, when present, has the given octet
/// (so explicit FALSE and BER-style TRUE can be spelled out).
pub(crate) fn extension_with_criticality(
    extension_oid: &str,
    critical: Option<u8>,
    value: &[u8],
) -> Vec<u8> {
    let mut parts = vec![oid(extension_oid)];
    parts.extend(critical.map(|octet| tlv(0x01, &[octet])));
    parts.push(tlv(0x04, value));
    sequence(&parts)
}

/// A KeyUsage extension with the given bit positions set (0 = digitalSignature).
pub(crate) fn key_usage(bits: &[usize]) -> Vec<u8> {
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

pub(crate) fn basic_constraints(ca: bool) -> Vec<u8> {
    let body = if ca { tlv(0x01, &[0xff]) } else { Vec::new() };
    extension("2.5.29.19", true, &tlv(0x30, &body))
}

pub(crate) fn extended_key_usage(oids: &[&str]) -> Vec<u8> {
    let list: Vec<Vec<u8>> = oids.iter().map(|o| oid(o)).collect();
    extension("2.5.29.37", false, &sequence(&list))
}

/// CertificatePolicies with one PolicyInformation per OID, the first carrying
/// a CPS qualifier so qualifier skipping is exercised.
pub(crate) fn policies(oids: &[&str]) -> Vec<u8> {
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
pub(crate) fn san_other_names(entries: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let names: Vec<Vec<u8>> = entries
        .iter()
        .map(|(type_id, value)| tlv(0xa0, &[oid(type_id), tlv(0xa0, value)].concat()))
        .collect();
    extension("2.5.29.17", false, &sequence(&names))
}

/// qcStatements from (statement OID, optional info TLV).
pub(crate) fn qc_statements(statements: &[(&str, Option<Vec<u8>>)]) -> Vec<u8> {
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
pub(crate) fn raw_extension(extension_oid: &str, value: &[u8]) -> Vec<u8> {
    extension(extension_oid, false, value)
}
