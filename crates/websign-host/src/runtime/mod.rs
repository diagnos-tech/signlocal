//! Real threads around the engine: the stdio reader, the key store worker,
//! the device monitor and the tick timer. The app supplies the UI port and
//! the launcher, and runs the UI event loop on the main thread.

mod device_label;
mod devices;
mod key_worker;
mod possible;
mod slow_listing;
mod snapshot;
#[cfg(test)]
mod softhsm_tests;
mod stdio;

use std::io::Read;
use std::sync::mpsc::{Sender, channel};
use std::time::Duration;

pub use key_worker::spawn_key_worker;
pub use stdio::{StdoutOutbound, WriterOutbound, spawn_stdin_reader};
use websign_keystores::{KeystoreHub, Options};

use crate::engine::{Control, Engine, EngineConfig, EngineEvent, Ports};
use crate::ports::{
    ConfirmUi, KeyCommand, KeyService, Launcher, Outbound, SLOW_LISTING, SystemClock,
};
use crate::store::{DiskStores, MemoryStores, Stores, data_dir};

/// Channel into the engine, cloned into every producer (the UI included).
pub type EventSender = Sender<EngineEvent>;

/// How often the engine hears the clock while a request may be open.
const TICK: Duration = Duration::from_millis(250);

/// Runs a host process on stdin/stdout until the client disconnects or the
/// idle limit passes; returns the exit status.
///
/// `ui` is built by the app with the returned [`EventSender`] so the window
/// can post decisions; the engine runs on the calling thread, so the app
/// calls this from a worker thread and keeps the main thread for egui.
pub fn serve_stdio(
    config: EngineConfig,
    make_ui: impl FnOnce(EventSender) -> Box<dyn ConfirmUi>,
    launcher: Box<dyn Launcher>,
) -> i32 {
    // Without a data folder the host still signs; it just remembers nothing.
    let stores: Box<dyn Stores> = match data_dir() {
        Some(dir) => Box::new(DiskStores::new(&dir)),
        None => {
            log::warn!("no data folder: running without persistence");
            Box::new(MemoryStores::new())
        }
    };
    serve(
        std::io::stdin(),
        Box::new(StdoutOutbound::new()),
        config,
        make_ui,
        launcher,
        stores,
        Options::default(),
    )
}

/// [`serve_stdio`] over any byte streams and stores; `options` are the base
/// key source options, to which the user's drivers from the settings are
/// added. Tests use it with pipes and a SoftHSM2 token.
pub fn serve(
    input: impl Read + Send + 'static,
    outbound: Box<dyn Outbound>,
    config: EngineConfig,
    make_ui: impl FnOnce(EventSender) -> Box<dyn ConfirmUi>,
    launcher: Box<dyn Launcher>,
    stores: Box<dyn Stores>,
    options: Options,
) -> i32 {
    let parts = Parts {
        input,
        outbound,
        config,
        launcher,
        stores,
    };
    serve_with(parts, make_ui, options, KeystoreHub::new)
}

/// What [`serve`] runs besides the window and the key stores.
pub(crate) struct Parts<R> {
    pub input: R,
    pub outbound: Box<dyn Outbound>,
    pub config: EngineConfig,
    pub launcher: Box<dyn Launcher>,
    pub stores: Box<dyn Stores>,
}

/// [`serve`] with the key store hub built by `make_hub` from the final
/// options, on the key worker's thread (tests put fake sources next to a
/// real module).
pub(crate) fn serve_with(
    parts: Parts<impl Read + Send + 'static>,
    make_ui: impl FnOnce(EventSender) -> Box<dyn ConfirmUi>,
    mut options: Options,
    make_hub: impl FnOnce(Options) -> KeystoreHub + Send + 'static,
) -> i32 {
    let Parts {
        input,
        outbound,
        config,
        launcher,
        mut stores,
    } = parts;
    let (events, inbox) = channel();
    match stores.settings().get() {
        Ok(settings) => options.extra_modules.extend(settings.user_modules),
        Err(error) => log::warn!("settings unavailable: {}", error.kind()),
    }
    let ui = make_ui(events.clone());
    let keys = key_worker::spawn_worker(move || make_hub(options), SLOW_LISTING, events.clone());
    stdio::spawn_reader(input, events.clone());
    let _monitor = devices::start_monitor(events.clone());
    spawn_ticker(events);

    let mut engine = Engine::new(
        config,
        Ports {
            clock: Box::new(SystemClock),
            outbound,
            ui,
            keys: Box::new(ChannelKeys(keys)),
            launcher,
            stores,
        },
    );
    while let Ok(event) = inbox.recv() {
        if let Control::Exit(status) = engine.handle(event) {
            return status;
        }
    }
    0
}

/// The engine's handle on the key store worker.
struct ChannelKeys(Sender<KeyCommand>);

impl KeyService for ChannelKeys {
    fn send(&mut self, command: KeyCommand) {
        if self.0.send(command).is_err() {
            log::error!("the key store worker is gone");
        }
    }
}

/// Posts a `Tick` every [`TICK`] until the engine stops listening.
fn spawn_ticker(events: EventSender) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(TICK);
            if events.send(EngineEvent::Tick).is_err() {
                return;
            }
        }
    });
}
