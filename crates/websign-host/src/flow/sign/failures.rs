//! A signature the key store could not make: what the window shows and
//! where the flow goes next (`SPEC.md` §4 table).

use websign_keystores::KeystoreError;
use websign_protocol::ErrorCode;
use websign_ui_model::certs::{CertCandidate, PinMode};
use websign_ui_model::confirm::port::{Failure, UiCommand};

use super::keys::{candidate_driver, path_of};
use super::signing::Attempt;
use super::{SignFlow, SignState};
use crate::flow::Effect;

impl SignFlow {
    /// Maps a key store error to what the window shows and where the flow
    /// goes next (`SPEC.md` §4 table).
    pub(super) fn signing_failed(
        &mut self,
        attempt: &Attempt,
        error: &KeystoreError,
    ) -> Vec<Effect> {
        let candidate = self
            .listing
            .as_ref()
            .and_then(|listing| listing.candidate(&attempt.fingerprint))
            .cloned();
        let key = self.key;
        let failed = |failure| Effect::Ui(UiCommand::Failed { key, failure });
        let (state, effects) = match error {
            KeystoreError::WrongPin => (
                Next::Ready,
                vec![failed(wrong_pin(candidate.as_ref(), attempt.via))],
            ),
            KeystoreError::PinLocked => (
                Next::Selecting,
                vec![
                    failed(Failure::PinLocked {
                        tool: None,
                        issuer: issuer_of(candidate.as_ref()),
                    }),
                    record(ErrorCode::PinLocked, None),
                ],
            ),
            KeystoreError::Cancelled => (Next::Ready, Vec::new()),
            KeystoreError::TokenRemoved => (Next::Selecting, vec![failed(Failure::TokenRemoved)]),
            KeystoreError::NotFound => (
                Next::Selecting,
                vec![failed(Failure::CertificateUnavailable)],
            ),
            KeystoreError::Unsupported(_) => (
                Next::Selecting,
                vec![failed(Failure::UnsupportedAlgorithm {
                    algorithm: websign_core::present::wire::algorithm_name(attempt.algorithm),
                })],
            ),
            KeystoreError::Native { .. } | KeystoreError::PinRequired | KeystoreError::Other(_) => {
                let native = native_status(error);
                (
                    Next::Ready,
                    vec![
                        failed(Failure::DriverFailure {
                            driver: candidate_driver(candidate.as_ref(), attempt.via),
                            native: native.clone(),
                            alternate: attempt.via == 0
                                && candidate.is_some_and(|c| !c.alternates.is_empty()),
                        }),
                        record(ErrorCode::DriverFailure, Some(native)),
                    ],
                )
            }
        };
        self.state = match state {
            Next::Ready => SignState::Ready {
                seq: self.seq,
                fingerprint: attempt.fingerprint,
            },
            Next::Selecting => SignState::Selecting { selected: None },
        };
        effects
    }
}

/// Where the flow goes after a failed signing.
enum Next {
    /// Same certificate and digest: the person can try again.
    Ready,
    /// The certificate cannot be used now: choose another.
    Selecting,
}

fn record(code: ErrorCode, native: Option<String>) -> Effect {
    Effect::RecordError { code, native }
}

/// The PIN flags of the path that was tried, as of the listing: the worker
/// reports only "wrong PIN", so the counters are the last ones read.
fn wrong_pin(candidate: Option<&CertCandidate>, via: usize) -> Failure {
    let pin = candidate.and_then(|c| path_of(c, via)).map(|(_, pin)| pin);
    let (count_low, final_try) = match pin {
        Some(PinMode::App {
            count_low,
            final_try,
            ..
        }) => (count_low, final_try),
        _ => (false, false),
    };
    Failure::PinIncorrect {
        count_low,
        final_try,
    }
}

fn issuer_of(candidate: Option<&CertCandidate>) -> String {
    candidate
        .and_then(|c| c.info.as_ref().ok())
        .and_then(|info| {
            info.issuer
                .common_name
                .clone()
                .or(info.issuer.organization.clone())
        })
        .unwrap_or_default()
}

/// The native status for "Technical details": codes and symbolic names only.
fn native_status(error: &KeystoreError) -> String {
    match error {
        KeystoreError::Native { message, code, .. } => format!("{message} ({code:#010x})"),
        KeystoreError::PinRequired => "a PIN is required".to_owned(),
        _ => "unspecified key store error".to_owned(),
    }
}
