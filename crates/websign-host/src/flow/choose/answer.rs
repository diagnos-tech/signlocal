//! Building the `choose.result`: which certificates, then their chains.

use std::collections::HashMap;

use websign_core::Fingerprint;
use websign_keystores::KeyRef;
use websign_protocol::AppMessage;
use websign_protocol::messages::ChooseResult;
use websign_protocol::types::Certificate;
use websign_ui_model::certs::ListContext;
use websign_ui_model::confirm::port::{Finish, UiCommand};

use super::{ChooseFlow, ChooseState};
use crate::flow::Effect;
use crate::flow::certificate::{wire_certificate, wire_chain};
use crate::flow::listing::Listing;
use crate::ports::{KeyCommand, KeyReply, KeySnapshot};

/// A reply waiting for the issuer chains of its certificates.
#[derive(Debug)]
pub(super) struct Answer {
    certificates: Vec<Certificate>,
    /// Chain lookups in flight: tag → index in `certificates`.
    pending: HashMap<u64, usize>,
    /// The person's "Remember" when they chose in the window; `None` for a
    /// remembered caller answered without it.
    remember: Option<bool>,
}

impl ChooseFlow {
    /// A listing arrived. `context` holds the list rules' inputs now.
    ///
    /// A windowless request answers with the remembered certificates that
    /// are still present. When none is, the caller is treated as new: the
    /// request returns to `Queued` without remembered certificates and the
    /// engine queues it for the window.
    pub fn on_listed(&mut self, snapshot: &KeySnapshot, context: ListContext) -> Vec<Effect> {
        let waiting = matches!(self.state, ChooseState::Queued | ChooseState::Done);
        if waiting || self.answer.is_some() {
            return Vec::new();
        }
        let listing = Listing::build(snapshot, context);
        if self.windowed {
            let command = listing.command(self.key);
            self.listing = Some(listing);
            self.state = ChooseState::Choosing;
            return vec![Effect::Ui(command)];
        }
        let present: Vec<Fingerprint> = self
            .remembered
            .iter()
            .filter_map(|hex| hex.parse::<Fingerprint>().ok())
            .filter(|fingerprint| listing.usable(fingerprint).is_some())
            .collect();
        self.listing = Some(listing);
        if present.is_empty() {
            self.remembered.clear();
            self.state = ChooseState::Queued;
            self.deadline = None;
            return Vec::new();
        }
        self.start_answer(&present, None)
    }

    /// The key store answered a chain lookup: fill it in, and reply once
    /// the last one is in.
    pub fn on_keys(&mut self, reply: &KeyReply) -> Vec<Effect> {
        let KeyReply::Chain { tag, chain } = reply else {
            return Vec::new();
        };
        let Some(answer) = self.answer.as_mut() else {
            return Vec::new();
        };
        let Some(index) = answer.pending.remove(tag) else {
            return Vec::new();
        };
        if let Some(certificate) = answer.certificates.get_mut(index) {
            certificate.chain = wire_chain(chain);
        }
        if answer.pending.is_empty() {
            self.reply()
        } else {
            Vec::new()
        }
    }

    /// The person chose a certificate the window offered.
    pub(super) fn chosen(&mut self, fingerprint: Fingerprint, remember: bool) -> Vec<Effect> {
        if self.state != ChooseState::Choosing || self.answer.is_some() {
            return Vec::new();
        }
        self.start_answer(&[fingerprint], Some(remember))
    }

    /// Asks for the chain of each certificate in `chosen` that is usable;
    /// nothing happens when none is (an unlisted or disabled choice).
    fn start_answer(&mut self, chosen: &[Fingerprint], remember: Option<bool>) -> Vec<Effect> {
        let Some(listing) = &self.listing else {
            return Vec::new();
        };
        let found: Vec<(Fingerprint, Certificate)> = chosen
            .iter()
            .filter_map(|fingerprint| listing.usable(fingerprint))
            .filter_map(|candidate| {
                let certificate = wire_certificate(listing.snapshot(), candidate, &[])?;
                Some((candidate.fingerprint, certificate))
            })
            .collect();
        if found.is_empty() {
            return Vec::new();
        }
        let (fingerprints, certificates): (Vec<_>, Vec<_>) = found.into_iter().unzip();
        let mut pending = HashMap::new();
        let mut effects = Vec::new();
        for (index, fingerprint) in fingerprints.into_iter().enumerate() {
            let tag = self.next_tag();
            pending.insert(tag, index);
            let key = KeyRef {
                fingerprint,
                path: 0,
            };
            effects.push(Effect::Keys(KeyCommand::Chain { tag, key }));
        }
        self.answer = Some(Answer {
            certificates,
            pending,
            remember,
        });
        effects
    }

    /// Sends the completed answer and ends the request.
    fn reply(&mut self) -> Vec<Effect> {
        let Some(answer) = self.answer.take() else {
            return Vec::new();
        };
        self.state = ChooseState::Done;
        let mut effects = Vec::new();
        if let (Some(remember), Some(first)) = (answer.remember, answer.certificates.first()) {
            effects.push(Effect::RecordConsent {
                remember,
                fingerprint: first.fingerprint.as_str().to_owned(),
            });
        }
        let windowed = answer.remember.is_some();
        effects.push(Effect::Send(AppMessage::ChooseResult(ChooseResult {
            certificates: answer.certificates,
        })));
        if windowed {
            effects.push(Effect::Ui(UiCommand::Finished {
                key: self.key,
                finish: Finish::Chosen,
            }));
        }
        effects
    }
}
