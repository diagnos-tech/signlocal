//! Fakes of every port, so a whole host process runs in a test in
//! milliseconds: a manual clock, recording outbound/UI/key/launcher ports and
//! in-memory stores. Handles are cheap clones sharing state with the port
//! given to the engine, so a test keeps its own handle to look and to
//! advance time.

mod clock;
mod recorders;

#[cfg(test)]
pub(crate) mod fixture;

pub use clock::ManualClock;
pub use recorders::{FakeKeys, FakeLauncher, FakeOutbound, FakeUi};

use crate::engine::Ports;
use crate::store::MemoryStores;

/// Test-side handles of the fakes inside a [`Ports`].
#[derive(Debug, Clone, Default)]
pub struct Handles {
    pub clock: ManualClock,
    pub outbound: FakeOutbound,
    pub ui: FakeUi,
    pub keys: FakeKeys,
    pub launcher: FakeLauncher,
}

/// Ports made of fakes, and the handles to drive and inspect them. The
/// stores are in memory and private to the engine.
pub fn fake_ports() -> (Ports, Handles) {
    let handles = Handles::default();
    let ports = Ports {
        clock: Box::new(handles.clock.clone()),
        outbound: Box::new(handles.outbound.clone()),
        ui: Box::new(handles.ui.clone()),
        keys: Box::new(handles.keys.clone()),
        launcher: Box::new(handles.launcher.clone()),
        stores: Box::new(MemoryStores::new()),
    };
    (ports, handles)
}
