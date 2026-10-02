//! The person presses Sign.

use secrecy::SecretString;
use websign_core::{Fingerprint, SignatureAlgorithm};
use websign_keystores::KeyRef;
use websign_ui_model::confirm::port::UiCommand;

use super::{SignFlow, SignState};
use crate::flow::Effect;
use crate::ports::KeyCommand;

/// One key store signing operation, kept to interpret its reply.
#[derive(Debug, Clone)]
pub(super) struct Attempt {
    pub fingerprint: Fingerprint,
    pub via: usize,
    pub algorithm: SignatureAlgorithm,
}

impl SignFlow {
    /// Sign was pressed for the certificate whose digest is on screen. Any
    /// other combination is stale input (the selection moved since) and is
    /// ignored.
    pub(super) fn sign(
        &mut self,
        fingerprint: Fingerprint,
        via: usize,
        pin: Option<SecretString>,
        remember: bool,
    ) -> Vec<Effect> {
        let SignState::Ready {
            fingerprint: ready, ..
        } = self.state
        else {
            return Vec::new();
        };
        let Some(candidate) = self
            .listing
            .as_ref()
            .and_then(|listing| listing.usable(&fingerprint))
        else {
            return Vec::new();
        };
        if ready != fingerprint || via > candidate.alternates.len() {
            return Vec::new();
        }
        let (Some(algorithm), Some(digest)) = (self.algorithm_for(candidate), self.digest.clone())
        else {
            return Vec::new();
        };
        let tag = self.next_tag();
        self.attempt = Some(Attempt {
            fingerprint,
            via,
            algorithm,
        });
        self.remember_choice = remember;
        self.state = SignState::Signing { tag, fingerprint };
        vec![
            Effect::Keys(KeyCommand::Sign {
                tag,
                key: KeyRef {
                    fingerprint,
                    path: via,
                },
                hash: self.hash(),
                algorithm,
                digest,
                pin,
                parent_window: None,
            }),
            Effect::Ui(UiCommand::Signing { key: self.key }),
        ]
    }
}
