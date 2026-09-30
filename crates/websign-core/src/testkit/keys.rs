//! `SubjectPublicKeyInfo` builders.

use super::{bit_string, oid, sequence, tlv};

/// An rsaEncryption SPKI with an all-ones modulus of `modulus_bits` bits.
pub(crate) fn spki_rsa(modulus_bits: usize) -> Vec<u8> {
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
    spki_rsa_integers(&content, &[0x01, 0x00, 0x01])
}

/// An rsaEncryption SPKI whose INTEGERs have exactly the given contents.
pub(crate) fn spki_rsa_integers(modulus: &[u8], exponent: &[u8]) -> Vec<u8> {
    let key = sequence(&[tlv(0x02, modulus), tlv(0x02, exponent)]);
    let algorithm = sequence(&[oid("1.2.840.113549.1.1.1"), tlv(0x05, &[])]);
    sequence(&[algorithm, bit_string(&key)])
}

/// An id-ecPublicKey SPKI naming `curve_oid`, with a dummy uncompressed point.
pub(crate) fn spki_ec(curve_oid: &str) -> Vec<u8> {
    spki_ec_point(curve_oid, &[&[0x04u8][..], &[0x11; 64]].concat())
}

/// An id-ecPublicKey SPKI naming `curve_oid` with the given encoded point.
pub(crate) fn spki_ec_point(curve_oid: &str, point: &[u8]) -> Vec<u8> {
    let algorithm = sequence(&[oid("1.2.840.10045.2.1"), oid(curve_oid)]);
    sequence(&[algorithm, bit_string(point)])
}

/// An SPKI for an arbitrary algorithm OID, without parameters.
pub(crate) fn spki_other(algorithm_oid: &str) -> Vec<u8> {
    let algorithm = sequence(&[oid(algorithm_oid)]);
    sequence(&[algorithm, tlv(0x03, &[0x00, 0x01, 0x02, 0x03])])
}
