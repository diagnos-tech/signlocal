//! Call the WebeSign app from a Rust program.
//!
//! The app is started on demand (`websign connect`) and speaks the same
//! protocol the browser extension uses; this crate hides the framing and the
//! message exchange:
//!
//! ```no_run
//! use websign_client::{Client, SignOptions};
//! use websign_protocol::types::HashName;
//!
//! let mut client = Client::connect()?;
//! let result = client.sign(SignOptions::new(HashName::Sha256), |certificate, algorithm| {
//!     // Build the signed attributes for `certificate` and hash them.
//!     let _ = (certificate, algorithm);
//!     Ok(vec![0u8; 32])
//! })?;
//! # Ok::<(), websign_client::ClientError>(())
//! ```
//!
//! The person confirms every signature in the app's window, which shows this
//! program's name (and its code signer when the OS can verify it).
//! Apache-2.0, like the web SDK.

#![cfg_attr(
    not(test),
    warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

mod client;
mod error;
mod locate;

pub use client::{Client, ConnectOptions, SignOptions};
pub use error::ClientError;
pub use locate::find_executable;
pub use websign_protocol::messages::{SignResult, StatusReply};
pub use websign_protocol::types::{
    Certificate, CertificateFilter, HashName, SignatureAlgorithmName,
};
