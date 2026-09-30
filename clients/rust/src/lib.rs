//! Call the WebeSign app from a Rust program.
//!
//! The app is started on demand (`websign connect`) and speaks the same
//! protocol the browser extension uses; this crate hides the framing and the
//! message exchange:
//!
//! ```no_run
//! use websign_client::{Client, ClientError, HashName, SignOptions};
//! use websign_protocol::ErrorCode;
//!
//! # fn signed_attributes_digest(_: &[u8], _: &[Vec<u8>]) -> Vec<u8> { vec![0; 32] }
//! let mut client = Client::connect()?;
//! let signed = client.sign(SignOptions::new(HashName::Sha256), |certificate, _algorithm| {
//!     // The certificate arrives decoded: DER bytes of the leaf and its chain.
//!     let chain: Vec<Vec<u8>> = certificate.chain.iter().map(|c| c.as_bytes().to_vec()).collect();
//!     Ok(signed_attributes_digest(certificate.der.as_bytes(), &chain))
//! });
//! match signed {
//!     Ok(result) => println!("{} signature bytes", result.signature.as_bytes().len()),
//!     Err(error) if error.code() == ErrorCode::UserCancelled => println!("cancelled"),
//!     Err(error) => return Err(error),
//! }
//! # Ok::<(), ClientError>(())
//! ```
//!
//! Errors are typed ([`ClientError`]): [`ClientError::AppMissing`] when the
//! app is not installed, [`ClientError::App`] with the protocol's stable
//! code when the app refuses or the person cancels, [`ClientError::Prepare`]
//! when your closure fails, and [`ClientError::Connection`] when the pipe
//! breaks or the app misbehaves. After a connection error the [`Client`]
//! is unusable; drop it and connect again. Dropping a client ends the app
//! and never leaves a process behind.
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
mod handshake;
mod locate;
mod process;
mod reader;
mod session;
mod sign;
mod timing;

pub use client::{Client, ConnectOptions, SignOptions};
pub use error::ClientError;
pub use locate::find_executable;
pub use websign_protocol::messages::{SignResult, StatusReply};
pub use websign_protocol::types::{
    Certificate, CertificateFilter, HashName, SignatureAlgorithmName,
};

/// Not part of the API: lets this crate's tests shorten the waits for a
/// silent or wedged app. May change or disappear in any release.
#[doc(hidden)]
pub mod __testing {
    use std::time::Duration;

    use crate::{Client, ClientError, ConnectOptions, timing::Timing};

    pub fn connect_with_waits(
        options: ConnectOptions,
        hello_reply: Duration,
        exit_grace: Duration,
    ) -> Result<Client, ClientError> {
        Client::connect_timed(
            options,
            Timing {
                hello_reply,
                exit_grace,
            },
        )
    }
}
