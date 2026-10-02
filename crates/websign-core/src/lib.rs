//! Platform-independent building blocks shared by every key source.
//!
//! Hash and signature algorithm names, signature encodings, certificate
//! summaries, signature verification and de-duplication (promoted from the
//! Phase-0 kit's `probe-core`, reviewed with 391 tests), plus the text rules
//! of [`present`]. Nothing here talks to the operating system, so every rule
//! can be pinned down by unit tests.
//!
//! The behavior contract lives in `SPEC.md` next to this crate.

#![cfg_attr(
    not(test),
    warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod algorithm;
pub mod cert;
pub mod dedup;
pub mod ecdsa;
pub mod fingerprint;
pub mod hash;
pub mod pkcs1;
pub mod present;
pub mod verify;

mod hex;
#[cfg(test)]
mod testkit;

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
/// The verification code lives in the Apache-2.0 protocol crate, because the
/// SDK and client libraries must compute it identically.
pub use websign_protocol::code::{VerificationCode, verification_code};
