//! A host process: the engine on a worker thread, the UI on the main thread.
//!
//! The main thread must own the UI event loop (macOS requires it), and winit
//! cannot create a second event loop in one process, so the event loop starts
//! lazily with the first window and lives until the process exits; the
//! window is hidden between requests. A connection that never needs a
//! window (`status`, a remembered `choose`) never starts one.
//!
//! Threads (`docs/architecture/overview.md` §Threads): main (UI), engine
//! (this module spawns it), and the stdin reader, key store worker, device
//! monitor and ticker that `websign_host::runtime::serve` spawns. The
//! process ends when the engine does: the client closed stdin, the desktop
//! idle limit passed, or the first frame was refused. Nothing outlives it:
//! the window closes when the engine ends, and [`exit_guard`] ends the
//! process if it has not shortly after.

mod exit_guard;
#[cfg(feature = "e2e")]
mod headless;
pub mod launcher;
mod ui_thread;

use std::io::Read;
use std::process::ExitCode;
use std::sync::mpsc::channel;

use websign_core::present::caller::DesktopCaller;
use websign_host::BrowserLaunch;
use websign_host::engine::EngineConfig;
use websign_host::ports::{ConfirmUi, Outbound};
use websign_host::runtime::{EventSender, StdoutOutbound, serve as serve_engine};
use websign_host::session::Transport;
use websign_host::store::{DiskStores, MemoryStores, Stores, data_dir};
use websign_keystores::Options;
use websign_protocol::ProtocolRange;
use websign_protocol::types::{AppInfo, OsName};
use websign_ui_model::confirm::UiCommand;

/// Serves a browser's native messaging port.
pub fn serve_browser(launch: BrowserLaunch) -> ExitCode {
    log::info!("started by {} for native messaging", launch.family);
    let status = serve(
        Transport::NativeMessaging { launch },
        std::io::stdin(),
        Box::new(StdoutOutbound::new()),
    );
    exit_code(status)
}

/// Serves `websign connect` for the program `caller`.
pub fn serve_desktop(caller: DesktopCaller) -> ExitCode {
    log::info!("started by a program for websign connect");
    let status = serve(
        Transport::Desktop { caller },
        std::io::stdin(),
        Box::new(StdoutOutbound::new()),
    );
    exit_code(status)
}

/// Runs one connection over `input`/`outbound` until the engine exits, with
/// the window on this (the main) thread; returns the engine's status.
pub fn serve(
    transport: Transport,
    input: impl Read + Send + 'static,
    outbound: Box<dyn Outbound + Send>,
) -> i32 {
    let config = EngineConfig {
        app: app_info(),
        transport,
    };
    let (commands, inbox) = channel::<UiCommand>();
    let (handoff, events) = channel::<EventSender>();
    let (ui_returned, exit_guard) = exit_guard::pair();
    let engine = std::thread::Builder::new()
        .name("engine".to_owned())
        .spawn(move || {
            let status = serve_engine(
                input,
                outbound as Box<dyn Outbound>,
                config,
                move |sender| {
                    let _ = handoff.send(sender.clone());
                    confirm_ui(commands, sender)
                },
                Box::new(launcher::ProcessLauncher),
                stores(),
                Options::default(),
            );
            exit_guard.wait_for_ui(status);
            status
        });
    let Ok(engine) = engine else {
        log::error!("could not start the engine thread");
        return 1;
    };
    // Returns once the engine dropped its end of the command channel.
    ui_thread::run(inbox, events);
    drop(ui_returned);
    engine.join().unwrap_or_else(|_| {
        log::error!("the engine thread panicked");
        1
    })
}

/// The window's port: the real window, or the scripted one of e2e builds.
fn confirm_ui(
    commands: std::sync::mpsc::Sender<UiCommand>,
    events: EventSender,
) -> Box<dyn ConfirmUi> {
    #[cfg(feature = "e2e")]
    if let Some(scripted) = headless::from_env(events.clone()) {
        return Box::new(scripted);
    }
    let _ = events;
    let window: Box<dyn ConfirmUi> = Box::new(crate::ui::bridge::WindowBridge::new(commands));
    #[cfg(feature = "e2e")]
    let window = crate::e2e::watch(window);
    window
}

/// Disk stores when the OS has a data folder; the host still signs without
/// one, it just remembers nothing.
fn stores() -> Box<dyn Stores> {
    match data_dir() {
        Some(dir) => Box::new(DiskStores::new(&dir)),
        None => {
            log::warn!("no data folder: running without persistence");
            Box::new(MemoryStores::new())
        }
    }
}

/// What this build says about itself in `hello`, `status` and `version`.
pub fn app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_owned(),
        protocols: ProtocolRange::CURRENT,
        os: if cfg!(windows) {
            OsName::Windows
        } else if cfg!(target_os = "macos") {
            OsName::Macos
        } else {
            OsName::Linux
        },
        arch: std::env::consts::ARCH.to_owned(),
        channel: crate::platform::channel::current(),
    }
}

/// The engine's status as a process exit code (anything odd is a failure).
pub fn exit_code(status: i32) -> ExitCode {
    ExitCode::from(u8::try_from(status).unwrap_or(1))
}
