//! One connection to the app.

use std::path::PathBuf;
use std::process::Child;

use websign_protocol::messages::{DiagnosticsTab, SignResult, StatusReply};
use websign_protocol::types::{
    Certificate, CertificateFilter, FingerprintHex, HashName, SignatureAlgorithmName,
};

use crate::error::ClientError;

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
#[derive(Debug)]
pub struct Client {
    child: Child,
    protocol: u32,
    next_id: u64,
}

impl Client {
    /// Starts the app with default options and says `hello`.
    pub fn connect() -> Result<Client, ClientError> {
        Client::connect_with(ConnectOptions::default())
    }

    /// Starts the app and says `hello`.
    pub fn connect_with(options: ConnectOptions) -> Result<Client, ClientError> {
        let _ = options;
        todo!("SPEC.md §2")
    }

    /// `status`.
    pub fn status(&mut self) -> Result<StatusReply, ClientError> {
        let _ = (&mut self.child, self.protocol, self.next_id);
        todo!("SPEC.md §2")
    }

    /// `choose`: the certificate the person picks (or the ones this program
    /// already used, when remembered).
    pub fn certificates(
        &mut self,
        filter: Option<CertificateFilter>,
    ) -> Result<Vec<Certificate>, ClientError> {
        let _ = filter;
        todo!("SPEC.md §2")
    }

    /// `sign.begin`, answering each `sign.need_digest` with `prepare`.
    /// `prepare` may run more than once (the person switched certificate);
    /// only the last digest is signed.
    pub fn sign(
        &mut self,
        options: SignOptions,
        prepare: impl FnMut(&Certificate, SignatureAlgorithmName) -> Result<Vec<u8>, String>,
    ) -> Result<SignResult, ClientError> {
        let _ = (options, prepare);
        todo!("SPEC.md §2")
    }

    /// `diagnostics.open`.
    pub fn open_diagnostics(&mut self, tab: Option<DiagnosticsTab>) -> Result<(), ClientError> {
        let _ = tab;
        todo!("SPEC.md §2")
    }
}

/// Closing stdin ends the app's session; the child is then reaped.
impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.try_wait();
    }
}
