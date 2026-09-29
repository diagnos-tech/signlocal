//! Platform-independent building blocks shared by every key source.
//!
//! Hash and signature algorithm names, signature encodings, certificate
//! summaries, signature verification and de-duplication. Nothing here talks to
//! the operating system, so every rule can be pinned down by unit tests.
//!
//! The behavior contract lives in `SPEC.md` next to this crate.

pub mod algorithm;
pub mod cert;
pub mod dedup;
pub mod ecdsa;
pub mod fingerprint;
pub mod hash;
pub mod pkcs1;
pub mod verify;

pub use algorithm::{SignatureAlgorithm, UnknownAlgorithmError};
pub use cert::{
    CertError, CertInfo, DistinguishedName, IcpBrasil, IcpLevel, KeyUsage, PublicKeyKind, QcType,
    Qualified,
};
pub use dedup::{Deduped, SourceKind, dedup_by_fingerprint};
pub use ecdsa::{Curve, EcdsaEncodingError};
pub use fingerprint::{Fingerprint, ParseFingerprintError};
pub use hash::{DigestLengthError, HashAlgorithm};
pub use verify::{VerifyError, verify};
