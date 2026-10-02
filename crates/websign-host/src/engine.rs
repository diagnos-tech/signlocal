//! The deterministic core loop of a host process.
//!
//! Single-threaded: every input is an [`EngineEvent`], every output goes
//! through a port. The runtime feeds events from the reader thread, the UI
//! thread, the key store worker, the device monitor and a timer; tests feed
//! them by hand. `SPEC.md` §8 lists the scenarios it must pass.

mod effects;
mod frames;
mod lifecycle;
mod listing;
mod persist;
mod present;
mod queueing;
mod request;
mod test_signature;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::time::Instant;

use websign_devices::monitor::DeviceEvent;
use websign_protocol::types::AppInfo;
use websign_ui_model::confirm::UiEvent;
use websign_ui_model::confirm::port::RequestKey;

use crate::ports::{Clock, ConfirmUi, KeyReply, KeyService, Launcher, Outbound};
use crate::queue::RequestQueue;
use crate::session::{Session, Transport};
use crate::store::Stores;
use request::Request;

/// Static facts of this process.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub app: AppInfo,
    pub transport: Transport,
}

/// Every input of the engine.
#[derive(Debug)]
pub enum EngineEvent {
    /// A complete frame from the client.
    Frame(Vec<u8>),
    /// The client closed the stream between frames.
    Closed,
    /// The stream broke or violated the framing (reason for the log).
    Broken(String),
    Ui(UiEvent),
    Keys(KeyReply),
    Device(DeviceEvent),
    /// Periodic tick (deadlines, idle exit); at least every 250 ms while a
    /// request is open.
    Tick,
}

/// What the runtime does after an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Control {
    Continue,
    /// End the process with this exit status once the UI has finished its
    /// closing animation.
    Exit(i32),
}

/// Everything outside the engine.
pub struct Ports {
    pub clock: Box<dyn Clock>,
    pub outbound: Box<dyn Outbound>,
    pub ui: Box<dyn ConfirmUi>,
    pub keys: Box<dyn KeyService>,
    pub launcher: Box<dyn Launcher>,
    pub stores: Box<dyn Stores>,
}

impl std::fmt::Debug for Ports {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Ports { .. }")
    }
}

/// The engine: session, queue and flows over the ports.
#[derive(Debug)]
pub struct Engine {
    config: EngineConfig,
    ports: Ports,
    session: Session,
    queue: RequestQueue,
    /// Open requests, in key order so that anything done to all of them is
    /// deterministic.
    requests: BTreeMap<RequestKey, Request>,
    next_key: u64,
    started: Instant,
    /// The last frame or request end, for the desktop idle exit.
    last_activity: Instant,
    listings: listing::Listings,
    /// Whether the launch arguments were checked (once, at the first frame).
    launch_checked: bool,
    /// The `v` of errors sent before negotiation: the version of the frame
    /// being refused, which its sender can read (protocol `SPEC.md` §5.1).
    refusal_version: u32,
}

impl Engine {
    /// An engine waiting for `hello`.
    pub fn new(config: EngineConfig, ports: Ports) -> Engine {
        let now = ports.clock.now();
        let session = Session::new(config.transport.clone(), config.app.clone());
        Engine {
            config,
            ports,
            session,
            queue: RequestQueue::default(),
            requests: BTreeMap::new(),
            next_key: 1,
            started: now,
            last_activity: now,
            listings: listing::Listings::default(),
            launch_checked: false,
            refusal_version: websign_protocol::PROTOCOL_VERSION,
        }
    }

    /// Handles one event.
    pub fn handle(&mut self, event: EngineEvent) -> Control {
        match event {
            EngineEvent::Frame(frame) => self.on_frame(&frame),
            EngineEvent::Closed => self.disconnect(0),
            EngineEvent::Broken(reason) => {
                log::warn!("connection broken: {reason}");
                self.disconnect(1)
            }
            EngineEvent::Ui(event) => {
                self.on_ui(event);
                Control::Continue
            }
            EngineEvent::Keys(reply) => {
                self.on_keys(reply);
                Control::Continue
            }
            EngineEvent::Device(event) => {
                self.on_device(&event);
                Control::Continue
            }
            EngineEvent::Tick => self.on_tick(),
        }
    }
}
