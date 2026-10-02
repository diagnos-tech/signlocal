//! Value types shared by several commands.

use clap::ValueEnum;
use websign_protocol::types::{HashName, SignatureAlgorithmName};

/// `--hash`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum HashArg {
    #[value(name = "SHA-256", alias = "sha256", alias = "sha-256")]
    Sha256,
    #[value(name = "SHA-384", alias = "sha384", alias = "sha-384")]
    Sha384,
    #[value(name = "SHA-512", alias = "sha512", alias = "sha-512")]
    Sha512,
}

impl From<HashArg> for HashName {
    fn from(hash: HashArg) -> HashName {
        match hash {
            HashArg::Sha256 => HashName::Sha256,
            HashArg::Sha384 => HashName::Sha384,
            HashArg::Sha512 => HashName::Sha512,
        }
    }
}

/// `--algorithm`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum AlgorithmArg {
    #[value(name = "ECDSA", alias = "ecdsa")]
    Ecdsa,
    #[value(name = "RSASSA-PKCS1-v1_5", alias = "pkcs1")]
    RsaPkcs1v15,
    #[value(name = "RSASSA-PSS", alias = "pss")]
    RsaPss,
}

impl From<AlgorithmArg> for SignatureAlgorithmName {
    fn from(algorithm: AlgorithmArg) -> SignatureAlgorithmName {
        match algorithm {
            AlgorithmArg::Ecdsa => SignatureAlgorithmName::Ecdsa,
            AlgorithmArg::RsaPkcs1v15 => SignatureAlgorithmName::RsaPkcs1v15,
            AlgorithmArg::RsaPss => SignatureAlgorithmName::RsaPss,
        }
    }
}

/// `--browser`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum BrowserArg {
    All,
    Chrome,
    Chromium,
    Edge,
    Brave,
    Vivaldi,
    Opera,
    Firefox,
}

impl From<BrowserArg> for websign_registration::BrowserChoice {
    fn from(browser: BrowserArg) -> websign_registration::BrowserChoice {
        use websign_registration::BrowserChoice as B;
        match browser {
            BrowserArg::All => B::All,
            BrowserArg::Chrome => B::Chrome,
            BrowserArg::Chromium => B::Chromium,
            BrowserArg::Edge => B::Edge,
            BrowserArg::Brave => B::Brave,
            BrowserArg::Vivaldi => B::Vivaldi,
            BrowserArg::Opera => B::Opera,
            BrowserArg::Firefox => B::Firefox,
        }
    }
}
