//! The key store worker, seen from the engine.
//!
//! Key store calls block (PIN dialogs, slow drivers), so they run on a
//! dedicated thread that owns the [`websign_keystores::KeystoreHub`]. The
//! engine sends [`KeyCommand`]s and receives [`KeyReply`]s as engine events;
//! it never blocks on a key store.

use std::collections::HashMap;
use std::sync::Arc;

use secrecy::SecretString;
use websign_core::{Fingerprint, HashAlgorithm, SignatureAlgorithm};
use websign_keystores::{KeyRef, KeystoreError};
use websign_ui_model::certs::CertCandidate;
use websign_ui_model::possible::PossibleCard;

/// A listing, shared read-only between the engine and the UI.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeySnapshot {
    /// One per de-duplicated certificate, OS path first.
    pub candidates: Vec<CertCandidate>,
    /// The DER of every candidate. `CertCandidate` carries only the parsed
    /// summary, but the caller receives the certificate itself and the host
    /// verifies signatures against it, so the bytes travel with the listing.
    pub certificates: HashMap<Fingerprint, Vec<u8>>,
    /// Plugged-in tokens and cards that brought no certificate and that the
    /// window may suggest (`docs/ux.md` §6).
    pub possible: Vec<PossibleCard>,
    /// `"<source>: <error>"` for sources that failed (diagnostics only).
    pub failures: Vec<String>,
}

/// Engine → key store worker. Commands are served in order.
#[derive(Debug)]
pub enum KeyCommand {
    /// List (from cache unless `refresh`).
    List { refresh: bool },
    /// Sign. `parent_window` owns OS PIN dialogs (the confirmation window's
    /// native handle).
    Sign {
        tag: u64,
        key: KeyRef,
        hash: HashAlgorithm,
        algorithm: SignatureAlgorithm,
        digest: Vec<u8>,
        pin: Option<SecretString>,
        parent_window: Option<isize>,
    },
    /// Issuer chain of a key, for `Certificate.chain`.
    Chain { tag: u64, key: KeyRef },
    /// Device event or "Scan again": drop the cache (sources stay loaded).
    Invalidate,
    /// Connection idle or token removed: forget PINs (`docs/plan.md` D5).
    EndSessions,
}

/// Key store worker → engine.
#[derive(Debug)]
pub enum KeyReply {
    Listed(Arc<KeySnapshot>),
    Signed {
        tag: u64,
        result: Result<Vec<u8>, KeystoreError>,
    },
    Chain {
        tag: u64,
        chain: Vec<Vec<u8>>,
    },
}

/// Sends commands to the worker; replies arrive as
/// [`crate::EngineEvent::Keys`].
pub trait KeyService {
    fn send(&mut self, command: KeyCommand);
}
