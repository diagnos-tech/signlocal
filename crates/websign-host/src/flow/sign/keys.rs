//! The key store answers.

use websign_core::PublicKeyKind;
use websign_keystores::KeystoreError;
use websign_protocol::messages::SignResult;
use websign_protocol::types::{Base64Bytes, Certificate};
use websign_protocol::{AppMessage, ErrorCode};
use websign_ui_model::certs::{CertCandidate, KeySource, PinMode};
use websign_ui_model::confirm::port::{Failure, Finish, UiCommand};

use super::signing::Attempt;
use super::{SignFlow, SignState};
use crate::flow::Effect;
use crate::flow::certificate::wire_certificate;
use crate::ports::KeyReply;

impl SignFlow {
    /// The key store answered. The chain a release waits for sends its
    /// `sign.need_digest`.
    pub fn on_keys(&mut self, reply: &KeyReply) -> Vec<Effect> {
        match reply {
            KeyReply::Chain { tag, chain } => {
                if let Some(fingerprint) = self.chain_lookups.remove(tag) {
                    self.chains.insert(fingerprint, chain.clone());
                }
                match self.state {
                    SignState::Releasing {
                        tag: waiting,
                        fingerprint,
                    } if waiting == *tag => self.need_digest(fingerprint),
                    _ => Vec::new(),
                }
            }
            KeyReply::Signed { tag, result } => match self.state {
                SignState::Signing { tag: current, .. } if current == *tag => {
                    self.signed(result.as_deref())
                }
                _ => Vec::new(),
            },
            KeyReply::Listed(_) | KeyReply::SlowListing { .. } => Vec::new(),
        }
    }

    fn signed(&mut self, result: Result<&[u8], &KeystoreError>) -> Vec<Effect> {
        let Some(attempt) = self.attempt.clone() else {
            return Vec::new();
        };
        match result {
            Ok(signature) => self.finish_signed(&attempt, signature),
            Err(error) => self.signing_failed(&attempt, error),
        }
    }

    /// Verifies the signature against the certificate, then sends it
    /// (`SPEC.md` §4.1): a driver that returns garbage must not reach a
    /// signature file.
    fn finish_signed(&mut self, attempt: &Attempt, signature: &[u8]) -> Vec<Effect> {
        let certificate = match self.verified_certificate(attempt, signature) {
            Ok(certificate) => certificate,
            Err(failure) => return self.back_to_ready(failure),
        };
        self.state = SignState::Done;
        vec![
            Effect::RecordConsent {
                remember: self.remember_choice,
                fingerprint: attempt.fingerprint.to_hex(),
            },
            Effect::Send(AppMessage::SignResult(SignResult {
                certificate,
                hash: self.request.hash,
                algorithm: websign_core::present::wire::algorithm_name(attempt.algorithm),
                signature: Base64Bytes::new(signature.to_vec()),
            })),
            Effect::Ui(UiCommand::Finished {
                key: self.key,
                finish: Finish::Signed,
            }),
        ]
    }

    /// The certificate to send with `signature`, or the failure to show when
    /// the signature does not verify against it.
    fn verified_certificate(
        &self,
        attempt: &Attempt,
        signature: &[u8],
    ) -> Result<Certificate, (Failure, Option<String>)> {
        let listing = self.listing.as_ref();
        let candidate = listing.and_then(|listing| listing.candidate(&attempt.fingerprint));
        let fail = || unverified(candidate_driver(candidate, attempt.via), candidate, attempt);
        let (Some(listing), Some(candidate)) = (listing, candidate) else {
            return Err(fail());
        };
        let der = listing.snapshot().certificates.get(&attempt.fingerprint);
        let verified = match (der, self.digest.as_deref(), candidate.info.as_ref()) {
            (Some(der), Some(digest), Ok(info)) => match info.key {
                PublicKeyKind::Ec { curve } if !curve.has_verifier() => {
                    log::debug!("signature not verified: no verifier for this curve");
                    true
                }
                _ => websign_core::verify(der, self.hash(), attempt.algorithm, digest, signature)
                    .is_ok(),
            },
            _ => false,
        };
        if !verified {
            return Err(fail());
        }
        let chain = self
            .chains
            .get(&attempt.fingerprint)
            .map_or(&[][..], Vec::as_slice);
        wire_certificate(listing.snapshot(), candidate, chain).ok_or_else(fail)
    }

    /// Back to `Ready` with a failure for the window and a line in the
    /// recent errors.
    fn back_to_ready(&mut self, failure: (Failure, Option<String>)) -> Vec<Effect> {
        let SignState::Signing { fingerprint, .. } = self.state else {
            return Vec::new();
        };
        let seq = self.seq;
        self.state = SignState::Ready { seq, fingerprint };
        let (failure, native) = failure;
        vec![
            Effect::Ui(UiCommand::Failed {
                key: self.key,
                failure,
            }),
            Effect::RecordError {
                code: ErrorCode::DriverFailure,
                native,
            },
        ]
    }
}

fn unverified(
    driver: String,
    candidate: Option<&CertCandidate>,
    attempt: &Attempt,
) -> (Failure, Option<String>) {
    let native = "signature did not verify".to_owned();
    let failure = Failure::DriverFailure {
        driver,
        native: native.clone(),
        alternate: attempt.via == 0 && candidate.is_some_and(|c| !c.alternates.is_empty()),
    };
    (failure, Some(native))
}

/// The source and PIN mode of path `via` of `candidate` (0 = primary,
/// n = `alternates[n - 1]`).
pub(super) fn path_of(candidate: &CertCandidate, via: usize) -> Option<(&KeySource, PinMode)> {
    match via.checked_sub(1) {
        None => Some((&candidate.source, candidate.pin)),
        Some(index) => candidate
            .alternates
            .get(index)
            .map(|path| (&path.source, path.pin)),
    }
}

/// Key store or driver name of path `via`, for "{driver} didn't respond".
pub(super) fn candidate_driver(candidate: Option<&CertCandidate>, via: usize) -> String {
    match candidate
        .and_then(|c| path_of(c, via))
        .map(|(source, _)| source)
    {
        Some(KeySource::Windows) => "Windows".to_owned(),
        Some(KeySource::MacosKeychain) => "macOS Keychain".to_owned(),
        Some(KeySource::MacosToken) => "macOS smart card".to_owned(),
        Some(KeySource::Driver { path }) => std::path::Path::new(path)
            .file_name()
            .map_or_else(|| path.clone(), |name| name.to_string_lossy().into_owned()),
        None => String::new(),
    }
}
