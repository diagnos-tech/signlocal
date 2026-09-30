//! One connection to the app.

use std::path::PathBuf;
use std::time::Instant;

use websign_protocol::messages::{
    Cancel, Choose, ClientMessage, DiagnosticsTab, Done, OpenDiagnostics, Status, StatusReply,
};
use websign_protocol::types::{
    Certificate, CertificateFilter, FingerprintHex, HashName, SignatureAlgorithmName,
};
use websign_protocol::{AppMessage, ClientEnvelope, RequestId};

use crate::error::ClientError;
use crate::handshake;
use crate::session::{Expect, Session, Wait};
use crate::timing::Timing;

/// How to start the app.
#[derive(Debug, Clone, Default)]
pub struct ConnectOptions {
    /// Use this executable instead of [`crate::find_executable`].
    pub executable: Option<PathBuf>,
    /// Sent in `hello` (logs only); defaults to `"websign-client"` and this
    /// crate's version.
    pub client_name: Option<String>,
    pub client_version: Option<String>,
}

/// What to sign.
#[derive(Debug, Clone)]
pub struct SignOptions {
    pub hash: HashName,
    /// Acceptable algorithms, preferred first; empty = any.
    pub algorithms: Vec<SignatureAlgorithmName>,
    /// Preselect this certificate.
    pub certificate: Option<FingerprintHex>,
}

impl SignOptions {
    /// Any algorithm, no preselection.
    pub fn new(hash: HashName) -> SignOptions {
        SignOptions {
            hash,
            algorithms: Vec::new(),
            certificate: None,
        }
    }
}

/// A running `websign connect` child and its negotiated protocol.
///
/// Calls block until the app answers; the person's decision time is limited
/// by the app itself (300 s), so no call waits forever on a healthy app.
/// Dropping the client ends the session and reaps the child.
#[derive(Debug)]
pub struct Client {
    pub(crate) session: Session,
    pub(crate) protocol: u32,
    next_id: u64,
}

impl Client {
    /// Starts the app with default options and says `hello`.
    pub fn connect() -> Result<Client, ClientError> {
        Client::connect_with(ConnectOptions::default())
    }

    /// Starts the app and says `hello`.
    pub fn connect_with(options: ConnectOptions) -> Result<Client, ClientError> {
        Client::connect_timed(options, Timing::DEFAULT)
    }

    pub(crate) fn connect_timed(
        options: ConnectOptions,
        timing: Timing,
    ) -> Result<Client, ClientError> {
        let (session, protocol) = handshake::connect(options, timing)?;
        Ok(Client {
            session,
            protocol,
            next_id: 0,
        })
    }

    /// `status`.
    pub fn status(&mut self) -> Result<StatusReply, ClientError> {
        let id = self.send_request(ClientMessage::Status(Status::default()))?;
        match self.next_message(&id)? {
            AppMessage::Status(reply) => Ok(reply),
            other => Err(self.unexpected(&other)),
        }
    }

    /// `choose`: the certificate the person picks (or the ones this program
    /// already used, when remembered).
    pub fn certificates(
        &mut self,
        filter: Option<CertificateFilter>,
    ) -> Result<Vec<Certificate>, ClientError> {
        let id = self.send_request(ClientMessage::Choose(Choose { web: None, filter }))?;
        match self.next_message(&id)? {
            AppMessage::ChooseResult(result) => Ok(result.certificates),
            other => Err(self.unexpected(&other)),
        }
    }

    /// `diagnostics.open`.
    pub fn open_diagnostics(&mut self, tab: Option<DiagnosticsTab>) -> Result<(), ClientError> {
        let id = self.send_request(ClientMessage::OpenDiagnostics(OpenDiagnostics { tab }))?;
        match self.next_message(&id)? {
            AppMessage::Done(Done {}) => Ok(()),
            other => Err(self.unexpected(&other)),
        }
    }

    /// Sends a request under a fresh id. Ids are `n<counter>`: unique for the
    /// life of the connection, well inside the protocol's id alphabet.
    pub(crate) fn send_request(
        &mut self,
        message: ClientMessage,
    ) -> Result<RequestId, ClientError> {
        self.next_id += 1;
        let id = RequestId::new(format!("n{}", self.next_id))
            .map_err(|error| self.session.poison(error.to_string()))?;
        self.send(&id, message)?;
        Ok(id)
    }

    /// Sends a continuation (`sign.digest`, `cancel`) of an open request.
    pub(crate) fn send(
        &mut self,
        id: &RequestId,
        message: ClientMessage,
    ) -> Result<(), ClientError> {
        let envelope = ClientEnvelope {
            v: self.protocol,
            id: id.clone(),
            message,
        };
        self.session.send(&envelope)
    }

    /// The next message of request `id`. An `error` message becomes
    /// [`ClientError::App`]; a message for another id is a protocol
    /// violation, since only one request is ever open.
    pub(crate) fn next_message(&mut self, id: &RequestId) -> Result<AppMessage, ClientError> {
        match self
            .session
            .receive(Expect::Negotiated(self.protocol), None)?
        {
            Wait::Message(envelope) if envelope.id != *id => Err(self
                .session
                .poison("the app answered a request that is not open")),
            Wait::Message(envelope) => match envelope.message {
                AppMessage::Error(error) => Err(ClientError::App {
                    code: error.code,
                    message: error.message,
                }),
                message => Ok(message),
            },
            Wait::TimedOut | Wait::Exited => Err(self.session.poison("the app exited")),
        }
    }

    /// Withdraws request `id` and returns `reason` once the app has ended it.
    ///
    /// The wait for the app's final answer is bounded as a whole: if it does
    /// not come in time, the connection is abandoned rather than reused in an
    /// unknown state.
    pub(crate) fn cancel(&mut self, id: &RequestId, reason: ClientError) -> ClientError {
        if self.send(id, ClientMessage::Cancel(Cancel {})).is_err() {
            return reason;
        }
        let deadline = Instant::now() + self.session.timing().exit_grace;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            match self
                .session
                .receive(Expect::Negotiated(self.protocol), Some(left))
            {
                Ok(Wait::Message(envelope)) if envelope.id == *id => {
                    if envelope.message.is_final() {
                        return reason;
                    }
                }
                Ok(Wait::Message(_)) | Ok(Wait::TimedOut) => {
                    self.session
                        .poison("the app did not end the cancelled request");
                    return reason;
                }
                Ok(Wait::Exited) | Err(_) => return reason,
            }
        }
    }

    pub(crate) fn unexpected(&mut self, message: &AppMessage) -> ClientError {
        let kind = match message {
            AppMessage::Hello(_) => "hello",
            AppMessage::Status(_) => "status",
            AppMessage::ChooseResult(_) => "choose.result",
            AppMessage::NeedDigest(_) => "sign.need_digest",
            AppMessage::SignResult(_) => "sign.result",
            AppMessage::Done(_) => "done",
            AppMessage::Error(_) => "error",
        };
        self.session
            .poison(format!("the app sent an unexpected {kind} message"))
    }
}
