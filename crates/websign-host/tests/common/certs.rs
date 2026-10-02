//! Certificates and signatures from websign-core's OpenSSL-made fixtures.
//! Real signatures matter: the engine verifies before it replies.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use websign_core::{CertInfo, Fingerprint, HashAlgorithm, SignatureAlgorithm};
use websign_host::ports::KeySnapshot;
use websign_ui_model::certs::{CertCandidate, KeySource, PinMode};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../websign-core/tests/fixtures")
}

fn records(file: &str) -> Vec<Vec<String>> {
    let path = fixtures().join("vectors").join(file);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    text.lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|l| l.split_whitespace().map(str::to_owned).collect())
        .collect()
}

fn unhex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).expect("hex"))
        .collect()
}

fn hash_word(hash: HashAlgorithm) -> &'static str {
    match hash {
        HashAlgorithm::Sha256 => "sha256",
        HashAlgorithm::Sha384 => "sha384",
        HashAlgorithm::Sha512 => "sha512",
    }
}

/// The digest of the fixture message under `hash`.
pub fn digest(hash: HashAlgorithm) -> Vec<u8> {
    let row = records("digests.txt")
        .into_iter()
        .find(|r| r[0] == hash_word(hash))
        .expect("digest row");
    unhex(&row[1])
}

/// A certificate that has a valid signature over [`digest`] in the fixtures.
#[derive(Clone)]
pub struct Cert {
    /// The certificate file (without `.der`).
    pub name: &'static str,
    /// Which fixture key signs (`p256`, `rsa2048`, …); may differ from `name`.
    pub signer: &'static str,
    pub der: Vec<u8>,
    pub fingerprint: Fingerprint,
    pub info: CertInfo,
    pub algorithms: Vec<SignatureAlgorithm>,
}

impl Cert {
    fn load(name: &'static str, signer: &'static str, algorithms: &[SignatureAlgorithm]) -> Cert {
        let path = fixtures().join("certs").join(format!("{name}.der"));
        let der = fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        Cert {
            name,
            signer,
            fingerprint: Fingerprint::of(&der),
            info: CertInfo::from_der(&der).expect("fixture parses"),
            der,
            algorithms: algorithms.to_vec(),
        }
    }

    /// EC P-256: ECDSA only.
    pub fn p256() -> Cert {
        Cert::load("p256", "p256", &[SignatureAlgorithm::Ecdsa])
    }

    /// RSA 2048: PKCS#1 v1.5 and PSS.
    pub fn rsa() -> Cert {
        Cert::load(
            "rsa2048",
            "rsa2048",
            &[SignatureAlgorithm::RsaPkcs1v15, SignatureAlgorithm::RsaPss],
        )
    }

    /// A second RSA key, so two RSA certificates can be told apart.
    pub fn rsa_b() -> Cert {
        Cert::load(
            "rsa2048b",
            "rsa2048b",
            &[SignatureAlgorithm::RsaPkcs1v15, SignatureAlgorithm::RsaPss],
        )
    }

    /// An ICP-Brasil certificate with a person's name and CPF in it (the
    /// log tests look for them). It shares the P-256 key.
    pub fn person() -> Cert {
        Cert::load("icp-pf-a3", "p256", &[SignatureAlgorithm::Ecdsa])
    }

    pub fn hex(&self) -> String {
        self.fingerprint.to_hex()
    }

    /// The listing entry the key store worker would produce.
    pub fn candidate(&self) -> CertCandidate {
        CertCandidate {
            fingerprint: self.fingerprint,
            info: Ok(self.info.clone()),
            source: KeySource::Windows,
            alternates: Vec::new(),
            device: None,
            pin: PinMode::System,
            algorithms: self.algorithms.clone(),
            hardware: Some(true),
            has_private_key: true,
            removed: false,
        }
    }

    /// A signature over [`digest`] that verifies against this certificate.
    pub fn signature(&self, hash: HashAlgorithm, algorithm: SignatureAlgorithm) -> Vec<u8> {
        let kind = match algorithm {
            SignatureAlgorithm::Ecdsa => "ecdsa",
            SignatureAlgorithm::RsaPkcs1v15 => "pkcs1",
            SignatureAlgorithm::RsaPss => "pss",
        };
        let row = records("signatures.txt")
            .into_iter()
            .find(|r| r[0] == self.signer && r[1] == kind && r[2] == hash_word(hash))
            .unwrap_or_else(|| panic!("no vector for {} {kind} {}", self.signer, hash_word(hash)));
        unhex(&row[3])
    }
}

/// A listing of these certificates, in order.
pub fn snapshot(certs: &[&Cert]) -> Arc<KeySnapshot> {
    Arc::new(KeySnapshot {
        candidates: certs.iter().map(|c| c.candidate()).collect(),
        certificates: certs
            .iter()
            .map(|c| (c.fingerprint, c.der.clone()))
            .collect(),
        ..KeySnapshot::default()
    })
}
