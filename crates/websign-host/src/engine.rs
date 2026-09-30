//! The deterministic core loop of a host process.
//!
//! Single-threaded: every input is an [`EngineEvent`], every output goes
//! through a port. The runtime feeds events from the reader thread, the UI
//! thread, the key store worker, the device monitor and a timer; tests feed
//! them by hand. `SPEC.md` §8 lists the scenarios it must pass.

use websign_devices::monitor::DeviceEvent;
use websign_protocol::types::AppInfo;
use websign_ui_model::confirm::UiEvent;

use crate::ports::{Clock, ConfirmUi, KeyReply, KeyService, Launcher, Outbound};
use crate::session::Transport;
use crate::store::Stores;

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
}

impl Engine {
    /// An engine waiting for `hello`.
    pub fn new(config: EngineConfig, ports: Ports) -> Engine {
        Engine { config, ports }
    }

    /// Handles one event.
    pub fn handle(&mut self, event: EngineEvent) -> Control {
        let _ = (event, &self.config, &mut self.ports);
        todo!("SPEC.md §8")
    }
}
