//! Frames from the client: accepted by the session, then dispatched by type.

use websign_project::{FIREFOX_ID, chromium_extension_ids};
use websign_protocol::messages::{Choose, Hello, HelloReply, OpenDiagnostics, SignBegin, Status};
use websign_protocol::messages::{Done, StatusReply};
use websign_protocol::{
    AppMessage, ClientEnvelope, ClientMessage, ErrorCode, RequestId, parse_client_message,
    refusal_version,
};

use super::request::Flow;
use super::{Control, Engine};
use crate::caller::Caller;
use crate::flow::choose::ChooseFlow;
use crate::flow::sign::SignFlow;
use crate::launch::{BrowserFamily, BrowserLaunch};
use crate::ports::unix_seconds;
use crate::session::{Accepted, Rejection, Transport};
use crate::store::ConnectionRecord;

impl Engine {
    pub(super) fn on_frame(&mut self, frame: &[u8]) -> Control {
        self.last_activity = self.ports.clock.now();
        if self.session.negotiated().is_none() {
            self.refusal_version = refusal_version(frame);
        }
        if !self.launch_checked {
            self.launch_checked = true;
            if let Transport::NativeMessaging { launch } = &self.config.transport
                && !launch_is_ours(launch)
            {
                return self.refuse_launch(frame);
            }
        }
        match self.session.accept(frame) {
            Ok(Accepted { envelope, caller }) => {
                self.dispatch(envelope, caller);
                Control::Continue
            }
            Err(rejection) => self.reject(*rejection),
        }
    }

    /// Defense in depth: the browser already checked the extension against
    /// the manifest, so a launch by anyone else is refused on its first
    /// frame.
    fn refuse_launch(&mut self, frame: &[u8]) -> Control {
        log::warn!("refusing a launch by an unknown extension");
        let id = match parse_client_message(frame, None) {
            Ok(envelope) => Some(envelope.id),
            Err(error) => error.id,
        };
        if let Some(id) = id {
            self.send_error(
                &id,
                ErrorCode::InvalidRequest,
                "this extension is not allowed",
            );
        }
        Control::Exit(1)
    }

    fn reject(&mut self, rejection: Rejection) -> Control {
        log::debug!("frame refused: {}", rejection.error.code.as_str());
        if let Some(id) = &rejection.id {
            self.send(id, AppMessage::Error(rejection.error));
        }
        if rejection.close {
            Control::Exit(1)
        } else {
            Control::Continue
        }
    }

    fn dispatch(&mut self, envelope: ClientEnvelope, caller: Option<Caller>) {
        let ClientEnvelope { id, message, .. } = envelope;
        match (message, caller) {
            (ClientMessage::Hello(hello), _) => self.on_hello(&id, &hello),
            (ClientMessage::Status(status), Some(caller)) => self.on_status(&id, &status, &caller),
            (ClientMessage::Choose(choose), Some(caller)) => self.on_choose(id, choose, caller),
            (ClientMessage::SignBegin(begin), Some(caller)) => {
                self.on_sign_begin(id, begin, caller)
            }
            (ClientMessage::OpenDiagnostics(open), _) => self.on_open_diagnostics(&id, &open),
            (ClientMessage::SignDigest(digest), _) => self.on_digest(&id, digest),
            (ClientMessage::Cancel(_), _) => self.on_cancel(&id),
            // The session guarantees a caller for these; without one there is
            // nothing safe to do.
            (_, None) => self.send_error(&id, ErrorCode::Internal, "the caller is unknown"),
        }
    }

    fn on_hello(&mut self, id: &RequestId, hello: &Hello) {
        let Some(protocol) = self.session.negotiated() else {
            return;
        };
        if let (Some(browser), Transport::NativeMessaging { .. }) =
            (&hello.browser, &self.config.transport)
        {
            let record = ConnectionRecord {
                browser: browser.name,
                browser_version: browser.version.clone(),
                extension_version: hello.client.version.clone(),
                last_seen: unix_seconds(self.ports.clock.wall()),
            };
            if let Err(error) = self.ports.stores.connections().record(record) {
                super::persist::log_store("connection not saved", &error);
            }
        }
        let reply = HelloReply {
            app: self.config.app.clone(),
            protocol,
        };
        self.send(id, AppMessage::Hello(reply));
    }

    fn on_status(&mut self, id: &RequestId, _status: &Status, caller: &Caller) {
        let remembered = self.consent_of(&caller.consent_key()).is_some();
        let reply = StatusReply {
            app: self.config.app.clone(),
            remembered,
        };
        self.send(id, AppMessage::Status(reply));
    }

    fn on_choose(&mut self, id: RequestId, request: Choose, caller: Caller) {
        let remembered = self
            .consent_of(&caller.consent_key())
            .map(|record| record.certificates)
            .unwrap_or_default();
        let key = self.new_key();
        let flow = ChooseFlow::new(key, request, remembered);
        let windowless = flow.answers_without_window();
        let flow = Flow::Choose(Box::new(flow));
        if windowless {
            self.start_windowless(key, id, caller, flow);
        } else {
            self.enqueue(key, id, caller, flow);
        }
    }

    fn on_sign_begin(&mut self, id: RequestId, request: SignBegin, caller: Caller) {
        let remembered = self.consent_of(&caller.consent_key()).is_some();
        let key = self.new_key();
        let flow = SignFlow::new(key, request, remembered);
        self.enqueue(key, id, caller, Flow::Sign(Box::new(flow)));
    }

    fn on_open_diagnostics(&mut self, id: &RequestId, open: &OpenDiagnostics) {
        match self.ports.launcher.open_diagnostics(open.tab) {
            Ok(()) => self.send(id, AppMessage::Done(Done {})),
            Err(_) => {
                log::warn!("could not start the diagnostics window");
                self.send_error(
                    id,
                    ErrorCode::Internal,
                    "the diagnostics window could not be started",
                );
            }
        }
    }

    fn on_digest(&mut self, id: &RequestId, digest: websign_protocol::messages::SignDigest) {
        let Some(key) = self.session.lookup(id) else {
            return;
        };
        let Some(request) = self.requests.get_mut(&key) else {
            return;
        };
        let effects = match &mut request.flow {
            Flow::Sign(flow) => flow.on_digest(digest),
            // A digest only answers `sign.need_digest`; a `choose` never asked.
            Flow::Choose(_) => request.end(ErrorCode::InvalidRequest),
        };
        self.perform(key, effects);
    }

    fn on_cancel(&mut self, id: &RequestId) {
        let Some(key) = self.session.lookup(id) else {
            return;
        };
        let effects = self
            .requests
            .get_mut(&key)
            .map(|request| request.end(ErrorCode::Aborted))
            .unwrap_or_default();
        self.perform(key, effects);
    }
}

/// Whether `launch` names an extension of this product.
fn launch_is_ours(launch: &BrowserLaunch) -> bool {
    match launch.family {
        BrowserFamily::Chromium => chromium_extension_ids().contains(&launch.extension_id.as_str()),
        BrowserFamily::Firefox => launch.extension_id == FIREFOX_ID,
        BrowserFamily::Manual => true,
    }
}
