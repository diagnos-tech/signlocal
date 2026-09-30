//! The thread that owns every key store.

use std::sync::mpsc::Sender;

use websign_keystores::Options;

use super::EventSender;
use crate::ports::KeyCommand;

/// Starts the worker: it creates the [`websign_keystores::KeystoreHub`] on
/// its own thread (key stores are not `Send`), serves commands in order, and
/// posts replies as `EngineEvent::Keys`. Listing maps the inventory to
/// `CertCandidate`s (device labels from `devices.json`).
pub fn spawn_key_worker(options: Options, events: EventSender) -> Sender<KeyCommand> {
    let _ = (options, events);
    todo!("SPEC.md §9")
}
