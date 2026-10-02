//! Conversions between core types and the wire types of `websign-protocol`.
//!
//! The protocol crate is Apache-2.0 and cannot depend on this GPL crate, so
//! it has its own small enums; these conversions are the only bridge.

use websign_protocol::types::{
    CertificateProfile, CurveName, EidasProfile, EidasType, HashName, KeyDescription, KeyStorage,
    SignatureAlgorithmName,
};

use crate::algorithm::SignatureAlgorithm;
use crate::cert::{CertInfo, IcpLevel, PublicKeyKind, QcType};
use crate::ecdsa::Curve;
use crate::hash::HashAlgorithm;

/// Core → wire.
pub fn hash_name(hash: HashAlgorithm) -> HashName {
    match hash {
        HashAlgorithm::Sha256 => HashName::Sha256,
        HashAlgorithm::Sha384 => HashName::Sha384,
        HashAlgorithm::Sha512 => HashName::Sha512,
    }
}

/// Wire → core.
pub fn hash_algorithm(hash: HashName) -> HashAlgorithm {
    match hash {
        HashName::Sha256 => HashAlgorithm::Sha256,
        HashName::Sha384 => HashAlgorithm::Sha384,
        HashName::Sha512 => HashAlgorithm::Sha512,
    }
}

/// Core → wire.
pub fn algorithm_name(algorithm: SignatureAlgorithm) -> SignatureAlgorithmName {
    match algorithm {
        SignatureAlgorithm::Ecdsa => SignatureAlgorithmName::Ecdsa,
        SignatureAlgorithm::RsaPkcs1v15 => SignatureAlgorithmName::RsaPkcs1v15,
        SignatureAlgorithm::RsaPss => SignatureAlgorithmName::RsaPss,
    }
}

/// Wire → core.
pub fn signature_algorithm(algorithm: SignatureAlgorithmName) -> SignatureAlgorithm {
    match algorithm {
        SignatureAlgorithmName::Ecdsa => SignatureAlgorithm::Ecdsa,
        SignatureAlgorithmName::RsaPkcs1v15 => SignatureAlgorithm::RsaPkcs1v15,
        SignatureAlgorithmName::RsaPss => SignatureAlgorithm::RsaPss,
    }
}

/// Core → wire.
pub fn curve_name(curve: Curve) -> CurveName {
    match curve {
        Curve::P256 => CurveName::P256,
        Curve::P384 => CurveName::P384,
        Curve::P521 => CurveName::P521,
        Curve::BrainpoolP256r1 => CurveName::BrainpoolP256r1,
        Curve::BrainpoolP384r1 => CurveName::BrainpoolP384r1,
        Curve::BrainpoolP512r1 => CurveName::BrainpoolP512r1,
    }
}

/// The wire key description; `None` for keys the app cannot sign with
/// (such certificates never reach a caller).
pub fn key_description(key: &PublicKeyKind) -> Option<KeyDescription> {
    match key {
        PublicKeyKind::Rsa { bits } => Some(KeyDescription::Rsa { bits: *bits }),
        PublicKeyKind::Ec { curve } => Some(KeyDescription::Ec {
            curve: curve_name(*curve),
        }),
        PublicKeyKind::Unsupported { .. } => None,
    }
}

/// The certificate profile callers use to enforce their own policy
/// (`SPEC.md` §13). `hardware` comes from the key store.
pub fn certificate_profile(info: &CertInfo, hardware: Option<bool>) -> CertificateProfile {
    CertificateProfile {
        icp_brasil: info
            .icp_brasil
            .as_ref()
            .map(|icp| icp.level.map_or_else(unknown_level, level_name)),
        eidas: info.qualified.as_ref().map(|qualified| EidasProfile {
            qualified: qualified.compliance,
            qscd: qualified.sscd,
            types: qualified.types.iter().copied().map(eidas_type).collect(),
        }),
        key_storage: match hardware {
            Some(true) => KeyStorage::Hardware,
            Some(false) => KeyStorage::Software,
            None => KeyStorage::Unknown,
        },
    }
}

fn unknown_level() -> String {
    "ICP-Brasil".to_owned()
}

fn level_name(level: IcpLevel) -> String {
    match level {
        IcpLevel::Other(_) => unknown_level(),
        known => known.to_string(),
    }
}

fn eidas_type(kind: QcType) -> EidasType {
    match kind {
        QcType::ESign => EidasType::Esign,
        QcType::ESeal => EidasType::Eseal,
        QcType::Web => EidasType::Web,
    }
}
