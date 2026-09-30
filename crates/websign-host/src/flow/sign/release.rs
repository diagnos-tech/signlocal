//! Releasing a certificate to the caller with `sign.need_digest`.

use websign_core::Fingerprint;
use websign_core::present::wire::{algorithm_name, hash_algorithm};
use websign_keystores::KeyRef;
use websign_protocol::messages::NeedDigest;
use websign_protocol::{AppMessage, ErrorCode};
use websign_ui_model::confirm::port::UiCommand;

use super::{SignFlow, SignState};
use crate::flow::Effect;
use crate::flow::certificate::wire_certificate;
use crate::flow::errors::error_reply;
use crate::ports::KeyCommand;
use websign_ui_model::confirm::port::Failure;

impl SignFlow {
    /// Asks the caller for the digest to sign with `fingerprint`: the
    /// certificate leaves the app here, so this only runs for a remembered
    /// caller or after "Continue" (D11).
    pub(super) fn release(&mut self, fingerprint: Fingerprint) -> Vec<Effect> {
        let Some(listing) = &self.listing else {
            return Vec::new();
        };
        let Some(candidate) = listing.usable(&fingerprint) else {
            return Vec::new();
        };
        let chain = self.chains.get(&fingerprint).cloned().unwrap_or_default();
        let algorithm = self.algorithm_for(candidate);
        let certificate = wire_certificate(listing.snapshot(), candidate, &chain);
        let (Some(algorithm), Some(certificate)) = (algorithm, certificate) else {
            return self.fail_internal("the selected certificate cannot be described");
        };
        self.seq += 1;
        self.state = SignState::AwaitingDigest {
            seq: self.seq,
            fingerprint,
        };
        let mut effects = vec![
            Effect::Send(AppMessage::NeedDigest(NeedDigest {
                seq: self.seq,
                certificate,
                hash: self.request.hash,
                algorithm: algorithm_name(algorithm),
            })),
            Effect::Ui(UiCommand::DigestPending {
                key: self.key,
                fingerprint,
            }),
        ];
        if !self.chains.contains_key(&fingerprint) {
            let tag = self.next_tag();
            self.chain_lookups.insert(tag, fingerprint);
            effects.push(Effect::Keys(KeyCommand::Chain {
                tag,
                key: KeyRef {
                    fingerprint,
                    path: 0,
                },
            }));
        }
        effects
    }

    /// Ends the request with `InvalidRequest`, telling the window it was the
    /// site's fault (a wrong digest length or sequence).
    pub(super) fn fail_request(&mut self, detail: &str) -> Vec<Effect> {
        self.state = SignState::Done;
        vec![
            error_reply(ErrorCode::InvalidRequest),
            Effect::Ui(UiCommand::Failed {
                key: self.key,
                failure: Failure::Internal {
                    detail: detail.to_owned(),
                },
            }),
        ]
    }

    /// Ends the request with `Internal`: a bug on our side.
    fn fail_internal(&mut self, detail: &str) -> Vec<Effect> {
        self.state = SignState::Done;
        vec![
            error_reply(ErrorCode::Internal),
            Effect::Ui(UiCommand::Failed {
                key: self.key,
                failure: Failure::Internal {
                    detail: detail.to_owned(),
                },
            }),
            Effect::RecordError {
                code: ErrorCode::Internal,
                native: None,
            },
        ]
    }

    /// The hash of the request as the key store names it.
    pub(super) fn hash(&self) -> websign_core::HashAlgorithm {
        hash_algorithm(self.request.hash)
    }
}
