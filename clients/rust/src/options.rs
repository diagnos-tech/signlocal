//! What the caller asks for: how to start the app, what to sign, and what
//! `prepare` is told. Same meaning as the web SDK's options.

use std::path::PathBuf;

use websign_protocol::types::{Certificate, FingerprintHex, HashName, SignatureAlgorithmName};

/// How to start the app.
#[derive(Debug, Clone, Default)]
pub struct ConnectOptions {
    /// Use this executable instead of [`crate::find_executable`].
    pub executable: Option<PathBuf>,
    /// Sent in `hello` (logs only); defaults to `"websign-client"` and this
    /// crate's version.
    pub client_name: Option<String>,
    pub client_version: Option<String>,
}

/// What to sign: the SDK's `{ hash, algorithm, certificate }`.
///
/// ```
/// use websign_client::{HashName, SignOptions, SignatureAlgorithmName};
///
/// let options = SignOptions::new(HashName::Sha256)
///     .algorithms([SignatureAlgorithmName::Ecdsa, SignatureAlgorithmName::RsaPss]);
/// assert_eq!(options.algorithms.len(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct SignOptions {
    pub hash: HashName,
    /// Acceptable algorithms, preferred first; empty = the app's default
    /// preference (ECDSA for EC keys, PKCS#1 v1.5 for RSA). Duplicates are
    /// dropped when sent. The app's choice must be in this set.
    pub algorithms: Vec<SignatureAlgorithmName>,
    /// Preselect this certificate (from an earlier `certificates()`).
    pub certificate: Option<FingerprintHex>,
}

impl SignOptions {
    /// Any algorithm, no preselection.
    pub fn new(hash: HashName) -> SignOptions {
        SignOptions {
            hash,
            algorithms: Vec::new(),
            certificate: None,
        }
    }

    /// Accepts only `algorithm`.
    pub fn algorithm(self, algorithm: SignatureAlgorithmName) -> SignOptions {
        self.algorithms([algorithm])
    }

    /// Accepts these algorithms, preferred first.
    pub fn algorithms(
        mut self,
        algorithms: impl IntoIterator<Item = SignatureAlgorithmName>,
    ) -> SignOptions {
        self.algorithms = algorithms.into_iter().collect();
        self
    }

    /// Preselects a certificate returned by `certificates()`; only its
    /// fingerprint is sent.
    pub fn certificate(self, certificate: &Certificate) -> SignOptions {
        self.fingerprint(certificate.fingerprint.clone())
    }

    /// Preselects a certificate by its SHA-256 fingerprint.
    pub fn fingerprint(mut self, fingerprint: FingerprintHex) -> SignOptions {
        self.certificate = Some(fingerprint);
        self
    }

    /// The algorithm set without duplicates, in the caller's order; `None`
    /// when empty (the protocol refuses an empty list).
    pub(crate) fn algorithm_set(&self) -> Option<Vec<SignatureAlgorithmName>> {
        unique(&self.algorithms)
    }
}

/// Passed to `prepare` with the certificate: what the signature will be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrepareContext {
    pub hash: HashName,
    /// The algorithm the signature will use (for CMS `signatureAlgorithm`
    /// and the algorithm-protection attribute).
    pub algorithm: SignatureAlgorithmName,
}

/// `algorithms` without duplicates, order kept; `None` when empty.
pub(crate) fn unique(algorithms: &[SignatureAlgorithmName]) -> Option<Vec<SignatureAlgorithmName>> {
    let mut set: Vec<SignatureAlgorithmName> = Vec::with_capacity(algorithms.len());
    for algorithm in algorithms {
        if !set.contains(algorithm) {
            set.push(*algorithm);
        }
    }
    (!set.is_empty()).then_some(set)
}

#[cfg(test)]
mod tests {
    use super::*;
    use SignatureAlgorithmName::{Ecdsa, RsaPss};

    #[test]
    fn the_algorithm_set_drops_duplicates_and_keeps_the_order() {
        let options = SignOptions::new(HashName::Sha256).algorithms([RsaPss, Ecdsa, RsaPss]);
        assert_eq!(options.algorithm_set(), Some(vec![RsaPss, Ecdsa]));
    }

    #[test]
    fn an_empty_set_means_the_default_preference() {
        assert_eq!(SignOptions::new(HashName::Sha256).algorithm_set(), None);
    }

    #[test]
    fn a_single_algorithm_replaces_the_set() {
        let options = SignOptions::new(HashName::Sha256)
            .algorithms([Ecdsa])
            .algorithm(RsaPss);
        assert_eq!(options.algorithms, [RsaPss]);
    }
}
