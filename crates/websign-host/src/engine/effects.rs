//! Doing what a flow asked for, and what follows when it ends.

use websign_protocol::limits::DIGEST_TIMEOUT;
use websign_protocol::{AppEnvelope, AppMessage, ErrorCode, RequestId, WireError};
use websign_ui_model::confirm::port::RequestKey;

use super::Engine;
use crate::flow::Effect;
use crate::ports::KeyCommand;

impl Engine {
    /// Sends `message` under `id`. A client that is gone is not an error
    /// here: the reader reports the disconnect on its own.
    pub(super) fn send(&mut self, id: &RequestId, message: AppMessage) {
        let envelope = AppEnvelope {
            v: self.reply_version(),
            id: id.clone(),
            message,
        };
        if let Err(error) = self.ports.outbound.send(&envelope) {
            log::debug!("reply not delivered: {error}");
        }
    }

    pub(super) fn send_error(&mut self, id: &RequestId, code: ErrorCode, message: &str) {
        let error = WireError {
            code,
            message: message.to_owned(),
            details: None,
        };
        self.send(id, AppMessage::Error(error));
    }

    /// The negotiated version; before it, the version of the frame being
    /// refused, so even a client whose range does not overlap ours can read
    /// why (`AppOutdated`, `ClientOutdated`).
    fn reply_version(&self) -> u32 {
        self.session.negotiated().unwrap_or(self.refusal_version)
    }

    /// Performs `effects` of request `key` in order, then settles the
    /// request: its digest deadline, and its end when it is done.
    pub(super) fn perform(&mut self, key: RequestKey, effects: Vec<Effect>) {
        for effect in effects {
            match effect {
                Effect::Send(message) => {
                    if let Some(id) = self.requests.get(&key).map(|request| request.id.clone()) {
                        self.send(&id, message);
                    }
                }
                Effect::Ui(command) => self.ports.ui.command(command),
                Effect::Keys(command) => self.send_keys(command),
                Effect::RecordConsent {
                    remember,
                    fingerprint,
                } => self.record_consent(key, remember, &fingerprint),
                Effect::RecordError { code, native } => self.record_error(key, code, native),
            }
        }
        self.settle(key);
    }

    /// Every key command goes through here: signing gets the confirmation
    /// window's handle (the flows never see the window), and listings are
    /// counted so a slow one can be announced.
    pub(super) fn send_keys(&mut self, mut command: KeyCommand) {
        match &mut command {
            KeyCommand::Sign { parent_window, .. } if parent_window.is_none() => {
                *parent_window = self.ports.ui.parent_window();
            }
            KeyCommand::List { .. } => self.listings.started(),
            _ => {}
        }
        self.ports.keys.send(command);
    }

    /// After anything happened to `key`: track the digest deadline; when the
    /// request is done, forget it and let the next one on.
    fn settle(&mut self, key: RequestKey) {
        let now = self.ports.clock.now();
        let Some(request) = self.requests.get_mut(&key) else {
            return;
        };
        request.track_digest(now, DIGEST_TIMEOUT);
        if request.is_done() {
            self.finish(key);
        }
    }

    fn finish(&mut self, key: RequestKey) {
        let Some(request) = self.requests.remove(&key) else {
            return;
        };
        self.session.close(&request.id);
        self.last_activity = self.ports.clock.now();
        log::debug!("{} request finished", request.operation());
        if !request.queued {
            return;
        }
        let promoted = self.queue.remove(key);
        match promoted {
            Some(next) => self.activate(next),
            None => self.refresh_position(),
        }
    }
}
