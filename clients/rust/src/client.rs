//! One connection to the app.

use std::time::Instant;

use websign_protocol::messages::{
    Cancel, Choose, ClientMessage, DiagnosticsTab, Done, OpenDiagnostics, Status, StatusReply,
};
use websign_protocol::types::{Certificate, CertificateFilter, SignatureAlgorithmName};
use websign_protocol::{AppMessage, ClientEnvelope, ErrorCode, RequestId};

use crate::error::ClientError;
use crate::handshake;
use crate::options::{ConnectOptions, unique};
use crate::session::{Expect, Session, Wait};
use crate::timing::Timing;

/// A running `websign connect` child and its negotiated protocol.
///
/// Calls block until the app answers; the person's decision time is limited
/// by the app itself (300 s), so no call waits forever on a healthy app.
/// Dropping the client ends the session and reaps the child.
///
/// ```no_run
/// use websign_client::Client;
///
/// let mut client = Client::connect()?;
/// println!("WebeSign {}", client.status()?.app.version);
/// # Ok::<(), websign_client::ClientError>(())
/// ```
#[derive(Debug)]
pub struct Client {
    pub(crate) session: Session,
    pub(crate) protocol: u32,
    next_id: u64,
}

impl Client {
    /// Starts the app with default options and says `hello`.
    ///
    /// Fails with [`ClientError::AppMissing`] when the app is not installed
    /// or exits at once, and with `ClientOutdated` / `AppOutdated`
    /// ([`ClientError::App`]) when no protocol version is common.
    ///
    /// ```no_run
    /// use websign_client::{Client, ClientError, ErrorCode};
    ///
    /// match Client::connect() {
    ///     Ok(_client) => println!("ready"),
    ///     Err(error) if error.code() == ErrorCode::AppMissing => eprintln!("{}", error.hint()),
    ///     Err(error) => return Err(error),
    /// }
    /// # Ok::<(), ClientError>(())
    /// ```
    pub fn connect() -> Result<Client, ClientError> {
        Client::connect_with(ConnectOptions::default())
    }

    /// Starts the app and says `hello`, with `options`.
    ///
    /// ```no_run
    /// use websign_client::{Client, ConnectOptions};
    ///
    /// let options = ConnectOptions::new().client_name("my-invoicing-app");
    /// let client = Client::connect_with(options)?;
    /// # Ok::<(), websign_client::ClientError>(())
    /// ```
    pub fn connect_with(options: ConnectOptions) -> Result<Client, ClientError> {
        Client::connect_timed(options, Timing::DEFAULT)
    }

    pub(crate) fn connect_timed(
        options: ConnectOptions,
        timing: Timing,
    ) -> Result<Client, ClientError> {
        let (session, protocol) = handshake::connect(options, timing)?;
        Ok(Client::from_parts(session, protocol))
    }

    /// Says `hello` over a session started elsewhere (the in-process fake).
    #[cfg(feature = "testing")]
    pub(crate) fn connect_over(
        session: Session,
        options: ConnectOptions,
    ) -> Result<Client, ClientError> {
        let (session, protocol) = handshake::say_hello(session, options, Timing::DEFAULT)?;
        Ok(Client::from_parts(session, protocol))
    }

    fn from_parts(session: Session, protocol: u32) -> Client {
        Client {
            session,
            protocol,
            next_id: 0,
        }
    }

    /// Asks the app who it is and whether this program is remembered. Never
    /// opens a window; `remembered` says whether
    /// [`certificates`](Client::certificates) will not open one either.
    ///
    /// ```no_run
    /// # let mut client = websign_client::Client::connect()?;
    /// let status = client.status()?;
    /// println!("WebeSign {} (remembered: {})", status.app.version, status.remembered);
    /// # Ok::<(), websign_client::ClientError>(())
    /// ```
    pub fn status(&mut self) -> Result<StatusReply, ClientError> {
        let id = self.send_request(ClientMessage::Status(Status::default()))?;
        match self.next_message(&id)? {
            AppMessage::Status(reply) => Ok(reply),
            other => Err(self.unexpected(&other)),
        }
    }

    /// `choose`: the certificate the person picks (or the ones this program
    /// already used, when remembered). `algorithms` narrows the choice to
    /// keys that can produce one of them; empty = any.
    ///
    /// Fails with `NoCertificates` when the person chose none, and with
    /// `UserCancelled` when they closed the window.
    ///
    /// ```no_run
    /// use websign_client::{HashName, SignOptions, SignatureAlgorithmName};
    ///
    /// # let mut client = websign_client::Client::connect()?;
    /// let chosen = client.certificates(&[SignatureAlgorithmName::Ecdsa])?;
    /// // Later signatures skip the choice:
    /// let options = SignOptions::new(HashName::Sha256).certificate(&chosen[0]);
    /// # let _ = options;
    /// # Ok::<(), websign_client::ClientError>(())
    /// ```
    pub fn certificates(
        &mut self,
        algorithms: &[SignatureAlgorithmName],
    ) -> Result<Vec<Certificate>, ClientError> {
        let filter = unique(algorithms).map(|algorithms| CertificateFilter {
            algorithms: Some(algorithms),
        });
        let id = self.send_request(ClientMessage::Choose(Choose { web: None, filter }))?;
        match self.next_message(&id)? {
            AppMessage::ChooseResult(result) if result.certificates.is_empty() => {
                Err(ClientError::App {
                    code: ErrorCode::NoCertificates,
                    message: "no certificate was chosen".into(),
                })
            }
            AppMessage::ChooseResult(result) => Ok(result.certificates),
            other => Err(self.unexpected(&other)),
        }
    }

    /// Opens the app's diagnostics window (in its own process), for example
    /// to help a person whose token is not listed.
    ///
    /// ```no_run
    /// use websign_client::DiagnosticsTab;
    ///
    /// # let mut client = websign_client::Client::connect()?;
    /// client.open_diagnostics(Some(DiagnosticsTab::Devices))?;
    /// # Ok::<(), websign_client::ClientError>(())
    /// ```
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
            .map_err(|error| self.session.poison_by("invalid request id", error))?;
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
