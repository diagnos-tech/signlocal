//! One real certificate and a real signature over a fixed digest, so the
//! engine's self-check (`SPEC.md` §4.1) runs against genuine cryptography.
//!
//! P-256, self-signed, valid until 2126, key usage digitalSignature and
//! nonRepudiation. The DER signature is over [`DIGEST`] taken as the
//! already-hashed input (`openssl pkeyutl -sign`); the tests convert it to
//! the raw `r || s` form key stores return.

use std::collections::HashMap;

use websign_core::{CertInfo, Curve, Fingerprint, SignatureAlgorithm, ecdsa};
use websign_ui_model::certs::{CertCandidate, KeySource, PinMode};

use crate::ports::KeySnapshot;

/// The digest that was signed (SHA-256 length).
pub(crate) const DIGEST: [u8; 32] = [0xAA; 32];

const CERTIFICATE_HEX: &str = concat!(
    "308201de30820184a00302010202144c7db7e8bed07f49d4494fb96569fdb03bb7fa66300a06082a8648ce3d",
    "040302303d310b300906035504061302425231123010060355040a0c09486f73742054657374311a30180603",
    "5504030c114d61726961205465737420486f6c6465723020170d3236303933303033333031325a180f323132",
    "36303930363033333031325a303d310b300906035504061302425231123010060355040a0c09486f73742054",
    "657374311a301806035504030c114d61726961205465737420486f6c6465723059301306072a8648ce3d0201",
    "06082a8648ce3d03010703420004392b6dc7ca09765a8c5394e0590ebf89134ac93f129fda9cfd8d381d90c5",
    "63214fa56734c88544c73174e116481510a95da7e299be25e722261b8288071b17c7a360305e301d0603551d",
    "0e041604141ebdb40a97808a0884a8d61a31242655205ed8b9301f0603551d230418301680141ebdb40a9780",
    "8a0884a8d61a31242655205ed8b9300c0603551d130101ff04023000300e0603551d0f0101ff0404030206c0",
    "300a06082a8648ce3d04030203480030450220178b52398bd13787bae7e896e4abdefc98d3bbb31259aa073a",
    "16d0f995b93658022100f69b82f067e6258f586a7785e2e5739f22a69e84c71dafd2061f92ba379bef50",
);

const SIGNATURE_DER_HEX: &str = concat!(
    "3044022068577f69d24e2beb4fe8efea7a45009cc802c4564bc31ca3a81740257c9ac64802203ec9ed698f16",
    "edec7d13df8a4f7724f3c7351486c45d90cea5892fe456fe000b",
);

fn unhex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&text[at..at + 2], 16).unwrap())
        .collect()
}

pub(crate) fn der() -> Vec<u8> {
    unhex(CERTIFICATE_HEX)
}

pub(crate) fn fingerprint() -> Fingerprint {
    Fingerprint::of(&der())
}

/// The signature over [`DIGEST`] as a key store returns it.
pub(crate) fn signature() -> Vec<u8> {
    ecdsa::der_to_raw(&unhex(SIGNATURE_DER_HEX), Curve::P256).unwrap()
}

/// The certificate as a listing candidate, on a PKCS#11 token.
pub(crate) fn candidate() -> CertCandidate {
    let der = der();
    CertCandidate {
        fingerprint: Fingerprint::of(&der),
        info: CertInfo::from_der(&der),
        source: KeySource::Driver {
            path: "libtest.so".to_owned(),
        },
        alternates: Vec::new(),
        device: None,
        pin: PinMode::App {
            length: None,
            count_low: false,
            final_try: false,
            locked: false,
        },
        algorithms: vec![SignatureAlgorithm::Ecdsa],
        hardware: Some(true),
        has_private_key: true,
        removed: false,
    }
}

/// A listing holding just that certificate.
pub(crate) fn snapshot() -> KeySnapshot {
    KeySnapshot {
        candidates: vec![candidate()],
        certificates: HashMap::from([(fingerprint(), der())]),
        ..KeySnapshot::default()
    }
}
