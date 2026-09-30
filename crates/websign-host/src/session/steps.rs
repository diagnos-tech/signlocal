//! The individual checks of [`Session::accept`], one function per rule of
//! `SPEC.md` §2.

use websign_core::present::origin::OriginError;
use websign_protocol::limits::MAX_IN_FLIGHT_PER_CONNECTION;
use websign_protocol::messages::Hello;
use websign_protocol::types::WebContext;
use websign_protocol::{
    ClientEnvelope, ClientMessage, ErrorCode, ParseError, RequestId, WireError, negotiate,
};

use super::{Accepted, Rejection, Session, Transport, validate};
use crate::caller::Caller;

pub(super) fn rejection(
    id: Option<RequestId>,
    code: ErrorCode,
    message: impl Into<String>,
    close: bool,
) -> Box<Rejection> {
    Box::new(Rejection {
        id,
        error: WireError {
            code,
            message: message.into(),
            details: None,
        },
        close,
    })
}

impl Session {
    /// A frame that failed strict parsing. Without an id the reply cannot be
    /// addressed, and before `hello` nothing about the peer is trusted.
    pub(super) fn parse_failure(&self, error: ParseError) -> Box<Rejection> {
        let close = error.id.is_none() || self.negotiated.is_none();
        rejection(error.id, error.code, error.message, close)
    }

    pub(super) fn accept_hello(
        &mut self,
        envelope: ClientEnvelope,
        hello: &Hello,
    ) -> Result<Accepted, Box<Rejection>> {
        if self.negotiated.is_some() {
            return Err(rejection(
                Some(envelope.id),
                ErrorCode::InvalidRequest,
                "hello was already sent on this connection",
                false,
            ));
        }
        if let Err(error) = validate(&self.transport, &envelope.message) {
            return Err(rejection(
                Some(envelope.id),
                error.code,
                error.message,
                true,
            ));
        }
        match negotiate(self.app.protocols, hello.protocols) {
            Ok(version) => {
                self.negotiated = Some(version);
                self.browser = hello.browser.clone();
                Ok(Accepted {
                    envelope,
                    caller: None,
                })
            }
            Err(code) => Err(rejection(
                Some(envelope.id),
                code,
                negotiation_message(code),
                true,
            )),
        }
    }

    pub(super) fn accept_continuation(
        &self,
        envelope: ClientEnvelope,
    ) -> Result<Accepted, Box<Rejection>> {
        let is_open = self.lookup(&envelope.id).is_some();
        let is_cancel = matches!(envelope.message, ClientMessage::Cancel(_));
        // A cancel can cross the request's final reply on the wire, so an
        // unknown id is not an error; a digest for nothing is.
        if !is_open && !is_cancel {
            return Err(rejection(
                Some(envelope.id),
                ErrorCode::InvalidRequest,
                "no open request has this id",
                false,
            ));
        }
        Ok(Accepted {
            envelope,
            caller: None,
        })
    }

    pub(super) fn accept_request(
        &self,
        envelope: ClientEnvelope,
    ) -> Result<Accepted, Box<Rejection>> {
        let id = envelope.id.clone();
        if self.open.contains_key(&id) {
            return Err(rejection(
                Some(id),
                ErrorCode::InvalidRequest,
                "duplicate id",
                false,
            ));
        }
        if self.open.len() >= MAX_IN_FLIGHT_PER_CONNECTION {
            return Err(rejection(
                Some(id),
                ErrorCode::Busy,
                "too many open requests on this connection",
                false,
            ));
        }
        if let Err(error) = validate(&self.transport, &envelope.message) {
            return Err(rejection(Some(id), error.code, error.message, false));
        }
        let caller = match &envelope.message {
            ClientMessage::Status(m) => Some(self.caller_of(m.web.as_ref())),
            ClientMessage::Choose(m) => Some(self.caller_of(m.web.as_ref())),
            ClientMessage::SignBegin(m) => Some(self.caller_of(m.web.as_ref())),
            _ => None,
        };
        match caller.transpose() {
            Ok(caller) => Ok(Accepted { envelope, caller }),
            Err(error) => Err(rejection(Some(id), error.code, error.message, false)),
        }
    }

    /// The caller as the transport establishes it; `web` is only trusted
    /// over native messaging, where the extension read it from the browser.
    fn caller_of(&self, web: Option<&WebContext>) -> Result<Caller, WireError> {
        let invalid = |message: &str| WireError {
            code: ErrorCode::InvalidRequest,
            message: message.to_owned(),
            details: None,
        };
        match (&self.transport, web, &self.browser) {
            (Transport::Desktop { caller }, _, _) => Ok(Caller::Desktop(caller.clone())),
            (Transport::NativeMessaging { .. }, Some(web), Some(browser)) => {
                Caller::web(web, browser).map_err(|error| match error {
                    OriginError::Insecure => WireError {
                        code: ErrorCode::InsecureOrigin,
                        message: "the origin is not a secure context".to_owned(),
                        details: None,
                    },
                    OriginError::Malformed => invalid("web.origin is not a valid origin"),
                })
            }
            (Transport::NativeMessaging { .. }, _, _) => {
                Err(invalid("web is required over native messaging"))
            }
        }
    }
}

fn negotiation_message(code: ErrorCode) -> &'static str {
    match code {
        ErrorCode::ClientOutdated => "the client speaks only older protocol versions",
        ErrorCode::AppOutdated => "the client requires a newer protocol version than this app",
        _ => "hello.protocols is not a valid range",
    }
}
