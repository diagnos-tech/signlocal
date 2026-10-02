//! `FakeApp`: what the integrator configures and connects to.

use std::io;
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;

use websign_protocol::types::{Certificate, SignatureAlgorithmName};
use websign_protocol::{ClientMessage, ErrorCode};

use super::sample::sample_certificate;
use super::serve::serve;
use crate::error::{ClientError, Reason};
use crate::session::Session;
use crate::timing::Timing;
use crate::{Client, ConnectOptions};

/// Where in a request the fake fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Stage {
    /// At the certificate window, before `prepare` runs.
    Choose,
    /// After `prepare` returned, like a wrong PIN or a removed token.
    Confirm,
}

#[derive(Debug, Clone)]
pub(super) struct Failure {
    pub code: ErrorCode,
    pub message: String,
    pub stage: Stage,
}

#[derive(Debug, Clone)]
pub(super) struct Config {
    pub certificates: Vec<Certificate>,
    pub remembered: bool,
    pub failure: Option<Failure>,
    pub signature: Option<Vec<u8>>,
}

impl Config {
    /// The certificates whose key can produce one of `wanted` (any when `None`).
    pub fn usable<'a>(
        &'a self,
        wanted: Option<&'a [SignatureAlgorithmName]>,
    ) -> impl Iterator<Item = &'a Certificate> {
        self.certificates
            .iter()
            .filter(move |c| wanted.is_none_or(|set| c.algorithms.iter().any(|a| set.contains(a))))
    }
}

/// A stand-in for the WebeSign app that lives inside your test process.
///
/// [`FakeApp::connect`] returns a real [`Client`] wired to it over in-process
/// pipes, speaking the real protocol: your code runs unchanged. It answers
/// `hello`, `status`, `choose`, `sign` and `diagnostics.open`, enforces the
/// app's digest-length rule, and answers `sign` with the digest itself as the
/// signature, so it never verifies: do not use it to test signature checks.
///
/// ```
/// use websign_client::testing::FakeApp;
/// use websign_client::{HashName, SignOptions};
///
/// let app = FakeApp::new();
/// let mut client = app.connect()?;
/// let signed = client.sign(SignOptions::new(HashName::Sha256), |_certificate, _context| {
///     Ok(vec![0; 32]) // your document's digest
/// })?;
/// assert_eq!(signed.certificate.display_name, "Test Holder");
/// # Ok::<(), websign_client::ClientError>(())
/// ```
#[derive(Debug, Clone)]
pub struct FakeApp {
    config: Arc<Config>,
    requests: Arc<Mutex<Vec<ClientMessage>>>,
}

impl Default for FakeApp {
    fn default() -> FakeApp {
        FakeApp::new()
    }
}

impl FakeApp {
    /// One ECDSA certificate, every signature succeeds.
    pub fn new() -> FakeApp {
        FakeApp::builder().build()
    }

    /// Configures the certificates and failures of the fake.
    pub fn builder() -> FakeAppBuilder {
        FakeAppBuilder {
            certificates: Vec::new(),
            config: Config {
                certificates: Vec::new(),
                remembered: false,
                failure: None,
                signature: None,
            },
        }
    }

    /// Starts a new in-process connection (each call is a fresh "app").
    pub fn connect(&self) -> Result<Client, ClientError> {
        self.connect_with(ConnectOptions::default())
    }

    /// Like [`FakeApp::connect`]; only `client_name` and `client_version` of
    /// `options` matter, since there is no executable to start.
    pub fn connect_with(&self, options: ConnectOptions) -> Result<Client, ClientError> {
        let (app_input, client_output) = pipe()?;
        let (client_input, app_output) = pipe()?;
        let (config, requests) = (Arc::clone(&self.config), Arc::clone(&self.requests));
        // Detached: it ends when the client drops its end of the pipe.
        thread::Builder::new()
            .name("websign-fake-app".into())
            .spawn(move || serve(config, requests, app_input, app_output))
            .map_err(|error| {
                ClientError::Connection(Reason::caused_by("cannot start the fake app", error))
            })?;
        let session = Session::over_pipes(client_input, client_output, Timing::DEFAULT)?;
        Client::connect_over(session, options)
    }

    /// Every request the fake heard, across connections, in order
    /// (`Hello`, `SignBegin`, `SignDigest`, …).
    pub fn requests(&self) -> Vec<ClientMessage> {
        self.requests
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

fn pipe() -> Result<(io::PipeReader, io::PipeWriter), ClientError> {
    io::pipe().map_err(|error| {
        ClientError::Connection(Reason::caused_by("cannot create the fake's pipes", error))
    })
}

/// Builder of a [`FakeApp`].
///
/// ```
/// use websign_client::testing::FakeApp;
/// use websign_client::{ErrorCode, HashName, SignOptions};
///
/// let app = FakeApp::builder().fail_at_choose(ErrorCode::UserCancelled).build();
/// let mut client = app.connect()?;
/// let error = client
///     .sign(SignOptions::new(HashName::Sha256), |_, _| Ok(vec![0; 32]))
///     .unwrap_err();
/// assert_eq!(error.code(), ErrorCode::UserCancelled);
/// # Ok::<(), websign_client::ClientError>(())
/// ```
#[derive(Debug, Clone)]
pub struct FakeAppBuilder {
    certificates: Vec<Certificate>,
    config: Config,
}

impl FakeAppBuilder {
    /// Adds a certificate the person "has". With none added, the app offers
    /// [`sample_certificate`].
    pub fn certificate(mut self, certificate: Certificate) -> FakeAppBuilder {
        self.certificates.push(certificate);
        self
    }

    /// What `status` reports as `remembered`; default `false`.
    pub fn remembered(mut self, remembered: bool) -> FakeAppBuilder {
        self.config.remembered = remembered;
        self
    }

    /// The bytes returned as `signature`; default the digest itself.
    pub fn signature(mut self, bytes: Vec<u8>) -> FakeAppBuilder {
        self.config.signature = Some(bytes);
        self
    }

    /// Fails `choose` and `sign.begin` with `code`, like a person who
    /// cancels; `prepare` never runs.
    pub fn fail_at_choose(self, code: ErrorCode) -> FakeAppBuilder {
        self.failing(code, Stage::Choose)
    }

    /// Fails a signature with `code` after `prepare` returned, like a
    /// blocked PIN or a removed token.
    pub fn fail_at_confirm(self, code: ErrorCode) -> FakeAppBuilder {
        self.failing(code, Stage::Confirm)
    }

    fn failing(mut self, code: ErrorCode, stage: Stage) -> FakeAppBuilder {
        self.config.failure = Some(Failure {
            code,
            message: format!("the fake app was told to fail with {code:?}"),
            stage,
        });
        self
    }

    /// The configured fake.
    pub fn build(mut self) -> FakeApp {
        if self.certificates.is_empty() {
            self.certificates.push(sample_certificate());
        }
        self.config.certificates = self.certificates;
        FakeApp {
            config: Arc::new(self.config),
            requests: Arc::default(),
        }
    }
}
