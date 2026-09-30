//! The thread that owns every key store.

use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

use websign_devices::hints::DeviceDatabase;
use websign_keystores::{KeyRef, KeystoreHub, Options, SignRequest};

use super::EventSender;
use super::snapshot::snapshot;
use crate::engine::EngineEvent;
use crate::ports::{KeyCommand, KeyReply};

/// Starts the worker: it creates the [`websign_keystores::KeystoreHub`] on
/// its own thread (key stores are not `Send`), serves commands in order, and
/// posts replies as `EngineEvent::Keys`. Listing maps the inventory to
/// `CertCandidate`s (device labels from `devices.json`).
///
/// The thread ends when the engine drops the returned sender.
pub fn spawn_key_worker(options: Options, events: EventSender) -> Sender<KeyCommand> {
    let (commands, inbox) = channel();
    std::thread::spawn(move || serve(options, &inbox, &events));
    commands
}

fn serve(options: Options, inbox: &Receiver<KeyCommand>, events: &EventSender) {
    let mut hub = KeystoreHub::new(options);
    // Device names and driver hints are a nicety: without the database the
    // list still works, with generic device names and no suggestions.
    let hints = DeviceDatabase::embedded()
        .inspect_err(|error| log::warn!("device hints unavailable: {error}"))
        .ok();
    for command in inbox {
        if let Some(reply) = execute(&mut hub, hints.as_ref(), command)
            && events.send(EngineEvent::Keys(reply)).is_err()
        {
            return;
        }
    }
}

/// Runs one command; commands without a reply return `None`.
fn execute(
    hub: &mut KeystoreHub,
    hints: Option<&DeviceDatabase>,
    command: KeyCommand,
) -> Option<KeyReply> {
    match command {
        KeyCommand::List { refresh } => {
            if refresh {
                hub.invalidate();
            }
            Some(KeyReply::Listed(Arc::new(snapshot(hub, hints))))
        }
        KeyCommand::Sign {
            tag,
            key,
            hash,
            algorithm,
            digest,
            pin,
            parent_window,
        } => {
            let request = SignRequest {
                hash,
                algorithm,
                digest: &digest,
                pin: pin.as_ref(),
                parent_window,
            };
            let result = hub.sign(key, &request).map(|signature| {
                log::debug!(
                    "signed with {} in {} ms",
                    signature.api,
                    signature.elapsed.as_millis()
                );
                signature.bytes
            });
            // The PIN was only needed for this call; dropping the command's
            // copy here zeroizes it.
            drop(pin);
            Some(KeyReply::Signed { tag, result })
        }
        KeyCommand::Chain { tag, key } => Some(chain(hub, tag, key)),
        KeyCommand::Invalidate => {
            hub.invalidate();
            None
        }
        KeyCommand::EndSessions => {
            hub.end_sessions();
            None
        }
    }
}

fn chain(hub: &mut KeystoreHub, tag: u64, key: KeyRef) -> KeyReply {
    KeyReply::Chain {
        tag,
        chain: hub.chain(key),
    }
}
