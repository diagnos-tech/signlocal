//! The thread that owns every key store.

use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use websign_devices::hints::DeviceDatabase;
use websign_keystores::{KeyRef, KeystoreHub, Options, SignRequest};

use super::EventSender;
use super::slow_listing::{SlowWatch, reading_device};
use super::snapshot::snapshot;
use crate::engine::EngineEvent;
use crate::ports::{KeyCommand, KeyReply, SLOW_LISTING};

/// Starts the worker: it creates the [`websign_keystores::KeystoreHub`] on
/// its own thread (key stores are not `Send`), serves commands in order, and
/// posts replies as `EngineEvent::Keys`. Listing maps the inventory to
/// `CertCandidate`s (device labels from `devices.json`), and a listing that
/// runs past [`SLOW_LISTING`] is announced with `KeyReply::SlowListing`.
///
/// The thread ends when the engine drops the returned sender.
pub fn spawn_key_worker(options: Options, events: EventSender) -> Sender<KeyCommand> {
    spawn_worker(move || KeystoreHub::new(options), SLOW_LISTING, events)
}

/// [`spawn_key_worker`] over the hub `make_hub` builds on the worker thread
/// (tests give it fake or chosen sources) and announcing slow listings after
/// `slow_after`.
pub(crate) fn spawn_worker(
    make_hub: impl FnOnce() -> KeystoreHub + Send + 'static,
    slow_after: Duration,
    events: EventSender,
) -> Sender<KeyCommand> {
    let (commands, inbox) = channel();
    std::thread::spawn(move || {
        let worker = Worker {
            hub: make_hub(),
            // Device names and driver hints are a nicety: without the
            // database the list still works, with generic device names and
            // no suggestions.
            hints: DeviceDatabase::embedded()
                .inspect_err(|error| log::warn!("device hints unavailable: {error}"))
                .ok(),
            slow_after,
            events,
        };
        worker.serve(&inbox);
    });
    commands
}

/// The worker thread's state.
struct Worker {
    hub: KeystoreHub,
    hints: Option<DeviceDatabase>,
    slow_after: Duration,
    events: EventSender,
}

impl Worker {
    fn serve(mut self, inbox: &Receiver<KeyCommand>) {
        for command in inbox {
            if let Some(reply) = self.execute(command)
                && self.events.send(EngineEvent::Keys(reply)).is_err()
            {
                return;
            }
        }
    }

    /// Lists, announcing a listing that takes long with the device it is
    /// probably reading.
    fn list(&mut self, refresh: bool) -> KeyReply {
        if refresh {
            self.hub.invalidate();
        }
        let scan = websign_devices::Snapshot::scan();
        let device = self
            .hints
            .as_ref()
            .and_then(|database| reading_device(&scan, database));
        let watch = SlowWatch::start(self.slow_after, device, self.events.clone());
        let snapshot = snapshot(&mut self.hub, self.hints.as_ref(), &scan);
        drop(watch);
        KeyReply::Listed(Arc::new(snapshot))
    }

    /// Runs one command; commands without a reply return `None`.
    fn execute(&mut self, command: KeyCommand) -> Option<KeyReply> {
        let hub = &mut self.hub;
        match command {
            KeyCommand::List { refresh } => Some(self.list(refresh)),
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
}

fn chain(hub: &mut KeystoreHub, tag: u64, key: KeyRef) -> KeyReply {
    KeyReply::Chain {
        tag,
        chain: hub.chain(key),
    }
}
