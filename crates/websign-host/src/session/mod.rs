//! One connection's protocol state: the `hello` handshake, the negotiated
//! version, the per-transport rules, and which request ids are open.

mod transport;
mod validate;

use std::collections::HashMap;

use websign_protocol::types::{AppInfo, BrowserInfo};
use websign_protocol::{ClientEnvelope, RequestId, WireError};
use websign_ui_model::confirm::port::RequestKey;

pub use transport::Transport;
pub use validate::validate;

use crate::caller::Caller;

/// A frame accepted by the session.
#[derive(Debug)]
pub struct Accepted {
    pub envelope: ClientEnvelope,
    /// Who asks, for requests that need it (`status`, `choose`,
    /// `sign.begin`); `None` for `hello` and continuations.
    pub caller: Option<Caller>,
}

/// A frame refused by the session.
#[derive(Debug)]
pub struct Rejection {
    /// The id to address the error reply to, when the frame had one.
    pub id: Option<RequestId>,
    pub error: WireError,
    /// The stream is no longer trustworthy (no `hello`, bad framing): reply
    /// if possible, then end the connection.
    pub close: bool,
}

/// Protocol state of one connection.
#[derive(Debug)]
pub struct Session {
    transport: Transport,
    app: AppInfo,
    negotiated: Option<u32>,
    browser: Option<BrowserInfo>,
    open: HashMap<RequestId, RequestKey>,
}

impl Session {
    /// A session before `hello`.
    pub fn new(transport: Transport, app: AppInfo) -> Session {
        Session {
            transport,
            app,
            negotiated: None,
            browser: None,
            open: HashMap::new(),
        }
    }

    /// The transport this session runs on.
    pub fn transport(&self) -> &Transport {
        &self.transport
    }

    /// The version agreed in `hello`.
    pub fn negotiated(&self) -> Option<u32> {
        self.negotiated
    }

    /// Parses and validates one frame (`SPEC.md` §2): strict parsing,
    /// `hello` first and only once, transport rules for `web`/`browser`,
    /// open-request bookkeeping for continuations, the per-connection
    /// in-flight limit.
    pub fn accept(&mut self, frame: &[u8]) -> Result<Accepted, Box<Rejection>> {
        let _ = (frame, &self.app, &self.browser);
        todo!("SPEC.md §2")
    }

    /// Records that request `id` is open under `key`.
    pub fn open(&mut self, id: RequestId, key: RequestKey) {
        self.open.insert(id, key);
    }

    /// Forgets request `id` once its final reply is sent.
    pub fn close(&mut self, id: &RequestId) -> Option<RequestKey> {
        self.open.remove(id)
    }

    /// The request an id refers to, for continuations.
    pub fn lookup(&self, id: &RequestId) -> Option<RequestKey> {
        self.open.get(id).copied()
    }
}
