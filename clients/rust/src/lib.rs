//! Call the WebeSign app from a Rust program.
//!
//! The person picks a certificate in the app's window, your closure supplies
//! the digest for it, and the app signs it with the key (PIN included). The
//! app is started on demand (`websign connect`) and speaks the same protocol
//! as the browser extension; this crate hides the framing and the message
//! exchange. The flow and the options are the web SDK's and the Node
//! client's (`@websign/desktop`).
//!
//! # Quickstart
//!
//! ```no_run
//! use websign_client::{Client, ErrorCode, HashName, SignOptions};
//!
//! # fn my_digest(_certificate_der: &[u8]) -> Vec<u8> { vec![0; 32] }
//! let mut client = Client::connect()?; // ClientError::AppMissing if not installed
//! let options = SignOptions::new(HashName::Sha256);
//! match client.sign(options, |certificate, _context| {
//!     // Runs once the person picked a certificate (again if they switch):
//!     // return the SHA-256 of what they will sign, 32 bytes.
//!     Ok(my_digest(certificate.der.as_bytes()))
//! }) {
//!     Ok(signed) => println!("{} signature bytes", signed.signature.as_bytes().len()),
//!     Err(error) if error.code() == ErrorCode::UserCancelled => println!("cancelled"),
//!     Err(error) => return Err(error),
//! }
//! # Ok::<(), websign_client::ClientError>(())
//! ```
//!
//! For CMS/PAdES, build the signed attributes from `certificate.der` and
//! `context.algorithm` inside the closure and return their digest: that is
//! why the digest comes from a closure and not a parameter.
//!
//! # Errors
//!
//! Every call returns a [`ClientError`]. Branch on [`ClientError::code`]
//! (the protocol's stable [`ErrorCode`]), show [`ClientError::hint`] to the
//! developer, and walk [`std::error::Error::source`] for the underlying
//! failure. After a [`ClientError::Connection`] the [`Client`] is unusable:
//! drop it and connect again. Dropping a client ends the app and never leaves
//! a process behind.
//!
//! # Testing without the app
//!
//! Enable the `testing` feature in your `dev-dependencies` and connect to
//! `testing::FakeApp` instead of the real app:
//!
//! ```toml
//! [dev-dependencies]
//! websign-client = { version = "0.1", features = ["testing"] }
//! ```
//!
//! ```
//! # #[cfg(feature = "testing")] {
//! use websign_client::testing::FakeApp;
//! use websign_client::{HashName, SignOptions};
//!
//! let mut client = FakeApp::new().connect()?;
//! let signed = client.sign(SignOptions::new(HashName::Sha256), |_, _| Ok(vec![0; 32]))?;
//! assert_eq!(signed.signature.as_bytes().len(), 32);
//! # }
//! # Ok::<(), websign_client::ClientError>(())
//! ```
//!
//! # Good to know
//!
//! - The person confirms every signature in the app's window, which shows
//!   this program's name (and its code signer when the OS can verify it).
//! - Blocking API over `std::process`; [`Client`] is `Send`.
//! - `WEBSIGN_EXECUTABLE` or [`ConnectOptions::executable`] pins the binary;
//!   otherwise [`find_executable`] searches `PATH` and the per-OS install
//!   locations.
//!
//! Apache-2.0, like the web SDK.

#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

mod client;
mod error;
mod handshake;
mod hint;
mod locate;
mod options;
mod process;
mod reader;
mod session;
mod sign;
#[cfg(feature = "testing")]
pub mod testing;
mod timing;

pub use client::Client;
pub use error::ClientError;
pub use locate::find_executable;
pub use options::{ConnectOptions, PrepareContext, SignOptions};
pub use websign_protocol::ErrorCode;
pub use websign_protocol::messages::{DiagnosticsTab, SignResult, StatusReply};
pub use websign_protocol::types::{
    AppInfo, Base64Bytes, Certificate, CertificateProfile, Channel, CurveName, EidasProfile,
    EidasType, FingerprintHex, HashName, KeyDescription, KeyStorage, OsName,
    SignatureAlgorithmName,
};

/// Compiles the README's quickstart as a doctest, so it cannot rot.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeQuickstart;

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
