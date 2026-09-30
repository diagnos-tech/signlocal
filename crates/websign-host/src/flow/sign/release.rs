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
    /// Releases `fingerprint` to the caller: the certificate leaves the app
    /// here, so this only runs for a certificate in the caller's consent or
    /// after "Continue" (D11).
    ///
    /// `sign.need_digest` carries the issuer chain, which a first release
    /// has not read yet: the flow asks for it and waits in `Releasing`
    /// until it arrives or [`super::CHAIN_WAIT`] passes
    /// ([`SignFlow::chain_wait_over`]), so a site's `prepare` sees the
    /// chain it needs to build the signature.
    pub(super) fn release(&mut self, fingerprint: Fingerprint) -> Vec<Effect> {
        if self.chains.contains_key(&fingerprint) {
            return self.need_digest(fingerprint);
        }
        let Some(listing) = &self.listing else {
            return Vec::new();
        };
        if listing.usable(&fingerprint).is_none() {
            return Vec::new();
        }
        let tag = self.next_tag();
        self.chain_lookups.insert(tag, fingerprint);
        self.state = SignState::Releasing { tag, fingerprint };
        vec![
            Effect::Ui(UiCommand::DigestPending {
                key: self.key,
                fingerprint,
            }),
            Effect::Keys(KeyCommand::Chain {
                tag,
                key: KeyRef {
                    fingerprint,
                    path: 0,
                },
            }),
        ]
    }

    /// Sends `sign.need_digest` for `fingerprint` with the chain known so
    /// far (none when its lookup is still running).
    pub(super) fn need_digest(&mut self, fingerprint: Fingerprint) -> Vec<Effect> {
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
        let announced = matches!(self.state, SignState::Releasing { .. });
        self.seq += 1;
        self.state = SignState::AwaitingDigest {
            seq: self.seq,
            fingerprint,
        };
        let mut effects = vec![Effect::Send(AppMessage::NeedDigest(NeedDigest {
            seq: self.seq,
            certificate,
            hash: self.request.hash,
            algorithm: algorithm_name(algorithm),
        }))];
        if !announced {
            effects.push(Effect::Ui(UiCommand::DigestPending {
                key: self.key,
                fingerprint,
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
