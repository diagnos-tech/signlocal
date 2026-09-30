//! SPEC §6 and §6.5: `KeyUsage`, BasicConstraints, EKU, policies and `can_sign`.

mod can_sign;
mod extensions;

#[path = "../common/mod.rs"]
mod common;

use common::{blank_info, info};
use websign_core::{CertInfo, KeyUsage};

fn only(bit: &str) -> KeyUsage {
    let mut usage = KeyUsage::default();
    match bit {
        "digital_signature" => usage.digital_signature = true,
        "non_repudiation" => usage.non_repudiation = true,
        "key_encipherment" => usage.key_encipherment = true,
        "data_encipherment" => usage.data_encipherment = true,
        "key_agreement" => usage.key_agreement = true,
        "key_cert_sign" => usage.key_cert_sign = true,
        "crl_sign" => usage.crl_sign = true,
        other => panic!("unknown key usage bit {other}"),
    }
    usage
}

fn usage_with(bits: &[&str]) -> KeyUsage {
    let mut usage = KeyUsage::default();
    for bit in bits {
        let one = only(bit);
        usage.digital_signature |= one.digital_signature;
        usage.non_repudiation |= one.non_repudiation;
        usage.key_encipherment |= one.key_encipherment;
        usage.data_encipherment |= one.data_encipherment;
        usage.key_agreement |= one.key_agreement;
        usage.key_cert_sign |= one.key_cert_sign;
        usage.crl_sign |= one.crl_sign;
    }
    usage
}

const ALL_BITS: [&str; 7] = [
    "digital_signature",
    "non_repudiation",
    "key_encipherment",
    "data_encipherment",
    "key_agreement",
    "key_cert_sign",
    "crl_sign",
];
