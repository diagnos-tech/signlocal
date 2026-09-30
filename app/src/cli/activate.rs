//! `websign:activate` from the website's `/activate` page: register with
//! every browser (stores run no install scripts), then open diagnostics with
//! "Getting started". Any other `websign:` URL only opens diagnostics.
//!
//! A web page can open this URL without asking, so the worst it may cause
//! is what the person could do from the app menu: registration repair and a
//! window. The URL carries nothing else.

use std::process::ExitCode;

use websign_host::ports::Launcher;

use crate::host_process::launcher::ProcessLauncher;
use crate::launch::UrlAction;

/// Handles the URL action the OS passed.
pub fn run(action: UrlAction) -> ExitCode {
    log::info!("started for a websign: URL ({action:?})");
    match action {
        UrlAction::Activate => super::install::activate_silently(),
        UrlAction::Diagnostics => super::install::repair_silently(),
    }
    // Diagnostics shows "Getting started" while anything is left to do.
    super::diagnostics::open(None)
}

/// A `websign:` URL delivered to this running process as an event (macOS,
/// [`crate::platform::url_events`]). Runs on the main thread inside the
/// event loop, so the work goes to a thread: the same registration as
/// [`run`], then a diagnostics window unless this process already shows
/// one (Launch Services also brings that one to the front).
pub fn on_url_event(url: &str) {
    let Some(action) = crate::launch::parse_url(url) else {
        log::warn!("ignored a URL event of another scheme");
        return;
    };
    log::info!("received a websign: URL event ({action:?})");
    let open_window = !super::diagnostics::is_open();
    let worker = std::thread::Builder::new()
        .name("url-event".to_owned())
        .spawn(move || {
            if action == UrlAction::Activate {
                super::install::activate_silently();
            }
            if open_window && let Err(error) = ProcessLauncher.open_diagnostics(None) {
                log::warn!("url event: diagnostics did not start: {error}");
            }
        });
    if worker.is_err() {
        log::warn!("url event: no thread to handle it");
    }
}
