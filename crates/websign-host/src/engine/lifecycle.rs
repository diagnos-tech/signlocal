//! Everything that is not a frame: the window, the key store, devices,
//! deadlines and the end of the connection.

use websign_devices::monitor::DeviceEvent;
use websign_protocol::ErrorCode;
use websign_protocol::limits::{DESKTOP_IDLE_EXIT, HELLO_TIMEOUT};
use websign_ui_model::confirm::UiEvent;
use websign_ui_model::confirm::port::RequestKey;

use super::{Control, Engine};
use crate::flow::{Effect, tag_owner};
use crate::ports::{KeyCommand, KeyReply};
use crate::queue::RequestQueue;
use crate::session::Transport;

impl Engine {
    /// A decision of the person, or a request of the window.
    pub(super) fn on_ui(&mut self, event: UiEvent) {
        match event {
            UiEvent::OpenDiagnostics { tab } => {
                if self.ports.launcher.open_diagnostics(tab).is_err() {
                    log::warn!("could not start the diagnostics window");
                }
            }
            UiEvent::Rescan { key } if self.queue.active() == Some(key) => {
                self.ports.keys.send(KeyCommand::Invalidate);
                self.ports.keys.send(KeyCommand::List { refresh: true });
            }
            UiEvent::Rescan { .. } => {}
            event => {
                let key = ui_key(&event);
                // Only the request on screen can be acted on; anything else
                // is a stale event from a window that has moved on.
                let Some(key) = key.filter(|key| self.queue.active() == Some(*key)) else {
                    return;
                };
                let effects = self
                    .requests
                    .get_mut(&key)
                    .map(|request| request.on_ui(event))
                    .unwrap_or_default();
                self.perform(key, effects);
            }
        }
    }

    /// The key store answered: a listing goes to every request waiting for
    /// one, an operation's result to the request that started it.
    pub(super) fn on_keys(&mut self, reply: KeyReply) {
        match reply {
            KeyReply::Listed(snapshot) => {
                if !snapshot.failures.is_empty() {
                    log::debug!("{} key source(s) failed to list", snapshot.failures.len());
                }
                let keys: Vec<RequestKey> = self.requests.keys().copied().collect();
                for key in keys {
                    let Some(context) = self.list_context_for(key) else {
                        continue;
                    };
                    let Some(request) = self.requests.get_mut(&key) else {
                        continue;
                    };
                    let effects = request.on_listed(&snapshot, context);
                    let needs_window = request.needs_window();
                    self.perform(key, effects);
                    if needs_window {
                        self.queue_for_window(key);
                    }
                }
            }
            reply @ (KeyReply::Signed { tag, .. } | KeyReply::Chain { tag, .. }) => {
                let key = tag_owner(tag);
                let effects = self
                    .requests
                    .get_mut(&key)
                    .map(|request| request.on_keys(&reply))
                    .unwrap_or_default();
                self.perform(key, effects);
            }
        }
    }

    /// A token or reader changed while someone may be looking at the list:
    /// drop the cached listing and read again. Sessions of a token that left
    /// are forgotten (`docs/plan.md` D5).
    pub(super) fn on_device(&mut self, event: &DeviceEvent) {
        if self.queue.active().is_none() {
            return;
        }
        if matches!(
            event,
            DeviceEvent::CardRemoved { .. } | DeviceEvent::ReaderRemoved { .. }
        ) {
            self.ports.keys.send(KeyCommand::EndSessions);
        }
        self.ports.keys.send(KeyCommand::Invalidate);
        self.ports.keys.send(KeyCommand::List { refresh: true });
    }

    /// Deadlines, the `hello` limit and the idle exit of desktop clients.
    pub(super) fn on_tick(&mut self) -> Control {
        let now = self.ports.clock.now();
        let idle_for = |since| now.saturating_duration_since(since);
        if self.session.negotiated().is_none() && idle_for(self.started) >= HELLO_TIMEOUT {
            log::info!("no hello arrived in time");
            return Control::Exit(0);
        }
        let expired: Vec<RequestKey> = self
            .requests
            .iter()
            .filter(|(_, request)| {
                let decided = request.deadline().is_some_and(|deadline| now >= deadline);
                let digest = request.digest_wait.is_some_and(|(_, at)| now >= at);
                decided || digest
            })
            .map(|(key, _)| *key)
            .collect();
        for key in expired {
            let effects = self
                .requests
                .get_mut(&key)
                .map(|request| request.end(ErrorCode::Timeout))
                .unwrap_or_default();
            self.perform(key, effects);
        }
        let desktop = matches!(self.config.transport, Transport::Desktop { .. });
        if desktop && self.requests.is_empty() && idle_for(self.last_activity) >= DESKTOP_IDLE_EXIT
        {
            log::info!("idle for too long");
            return Control::Exit(0);
        }
        Control::Continue
    }

    /// The client is gone: every open request ends without a reply, and the
    /// window tells the person the site cancelled.
    pub(super) fn disconnect(&mut self, status: i32) -> Control {
        // Taken out first: ending the active request must not promote the
        // next one onto a window that is about to say "cancelled".
        let requests = std::mem::take(&mut self.requests);
        self.queue = RequestQueue::default();
        for (_, mut request) in requests {
            for effect in request.disconnected() {
                if let Effect::Ui(command) = effect {
                    self.ports.ui.command(command);
                }
            }
        }
        Control::Exit(status)
    }
}

/// The request a window event is about.
fn ui_key(event: &UiEvent) -> Option<RequestKey> {
    match event {
        UiEvent::Selected { key, .. }
        | UiEvent::Continue { key, .. }
        | UiEvent::Sign { key, .. }
        | UiEvent::Choose { key, .. }
        | UiEvent::Cancel { key, .. }
        | UiEvent::Rescan { key } => Some(*key),
        UiEvent::OpenDiagnostics { .. } => None,
    }
}
