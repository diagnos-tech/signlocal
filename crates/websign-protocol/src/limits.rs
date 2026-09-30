//! Every size and time limit of the protocol, in one place.
//!
//! Limits protect the app from a hostile or buggy peer (memory, log flooding,
//! a request nobody answers) and keep replies inside what browsers accept.
//! `docs/architecture/protocol.md` §Limits explains each value.

use std::time::Duration;

/// Longest request id, in bytes.
pub const MAX_REQUEST_ID_LEN: usize = 64;

/// Longest origin string accepted in a [`crate::types::WebContext`].
pub const MAX_ORIGIN_LEN: usize = 512;

/// Longest free-text field a client may send (client name, version strings).
pub const MAX_SHORT_TEXT_LEN: usize = 64;

/// Largest frame the app reads. Every request is tiny; the cap bounds the
/// allocation a peer can cause.
pub const MAX_INCOMING_FRAME: usize = crate::framing::MAX_INCOMING;

/// Largest frame the app writes: Chrome and Firefox drop hosts that exceed it.
pub const MAX_OUTGOING_FRAME: usize = crate::framing::MAX_OUTGOING;

/// Requests waiting behind the one on screen, per app process. One more is
/// refused with `Busy`.
pub const MAX_QUEUED_REQUESTS: usize = 10;

/// Requests of any kind in flight on one connection.
pub const MAX_IN_FLIGHT_PER_CONNECTION: usize = 16;

/// Time a client has, after starting the app, to send `hello`.
pub const HELLO_TIMEOUT: Duration = Duration::from_secs(5);

/// Time the caller has to answer `sign.need_digest` with `sign.digest`.
pub const DIGEST_TIMEOUT: Duration = Duration::from_secs(60);

/// Time a person has to decide in the confirmation window.
pub const DECISION_TIMEOUT: Duration = Duration::from_secs(300);

/// A `websign connect` session with no request for this long ends, so a
/// forgotten child process does not linger. Browsers close idle ports
/// themselves ([`EXTENSION_IDLE_CLOSE`]).
pub const DESKTOP_IDLE_EXIT: Duration = Duration::from_secs(300);

/// How long the extension keeps an idle native messaging port open, so the
/// next request skips the process start and the PKCS#11 driver load.
pub const EXTENSION_IDLE_CLOSE: Duration = Duration::from_secs(60);

/// How long the extension waits for the app's `hello` reply before it reports
/// the app as not responding.
pub const APP_RESPONSE_TIMEOUT: Duration = Duration::from_secs(3);

/// Certificates in one chain, leaf excluded.
pub const MAX_CHAIN_LEN: usize = 8;
