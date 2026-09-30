//! What the window asks of the rest of the app: the OS (links, certificate
//! viewer, file picker, `.pfx` import), browser registration, and a scan of
//! this computer. Behind traits so the tests drive the window with fakes and
//! nothing real is opened, registered or scanned.

use std::path::Path;
use std::sync::mpsc::{Receiver, TryRecvError, channel};

use websign_registration::{Action, BrowserChoice, Request, Scope};

use super::facts::{Facts, ScanInput, collect};
use crate::platform::file_picker::{self, FileRequest, Picked};

/// The window's native handle, owner of the OS dialogs (`HWND` on
/// Windows; `None` in tests and where dialogs need no owner).
pub type Owner = Option<isize>;

/// OS actions.
pub trait Os {
    /// Opens an https link in the default browser.
    fn open_url(&mut self, url: &str);
    /// Rewrites the host registration of every browser (`[Repair]`);
    /// `true` when nothing failed.
    fn repair_registration(&mut self) -> bool;
    /// The OS certificate viewer.
    fn view_certificate(&mut self, der: &[u8], owner: Owner);
    /// The OS `.pfx` import of `file`, or of the file its own dialog asks
    /// for; `true` when it ran.
    fn import_pfx(&mut self, file: Option<&Path>, owner: Owner) -> bool;
    /// The OS "open file" dialog; `done` gets the outcome, possibly from
    /// another thread ([`file_picker::choose_file`]).
    fn choose_file(
        &mut self,
        request: FileRequest,
        owner: Owner,
        done: Box<dyn FnOnce(Picked) + Send>,
    );
}

/// A scan of this computer, run somewhere else than the UI thread.
pub trait Scanner {
    /// Starts a scan unless one is running; `wake` is called when it ends.
    fn start(&mut self, input: ScanInput, wake: Box<dyn FnOnce() + Send>);
    /// The facts of the scan that just finished, once.
    fn finished(&mut self) -> Option<Facts>;
    fn busy(&self) -> bool;
}

/// The real OS.
#[derive(Debug, Default)]
pub struct RealOs;

impl Os for RealOs {
    fn open_url(&mut self, url: &str) {
        if !crate::platform::system_ui::open_url(url) {
            log::warn!("diagnostics: the link could not be opened");
        }
    }

    fn repair_registration(&mut self) -> bool {
        let request = Request {
            action: Action::Install,
            scope: Scope::User,
            browsers: vec![BrowserChoice::All],
            user_data_dir: None,
            extension_ids: Vec::new(),
            manifest_dir: None,
            host: None,
            dry_run: false,
        };
        match websign_registration::run(&request) {
            Ok(report) => !report.failed(),
            Err(error) => {
                log::warn!("diagnostics: repair could not run: {error}");
                false
            }
        }
    }

    fn view_certificate(&mut self, der: &[u8], owner: Owner) {
        crate::platform::system_ui::view_certificate(der, owner);
    }

    fn import_pfx(&mut self, file: Option<&Path>, owner: Owner) -> bool {
        crate::platform::system_ui::import_pfx(file, owner)
    }

    fn choose_file(
        &mut self,
        request: FileRequest,
        owner: Owner,
        done: Box<dyn FnOnce(Picked) + Send>,
    ) {
        file_picker::choose_file(request, owner, done);
    }
}

/// Scans on a worker thread, one at a time.
#[derive(Debug, Default)]
pub struct ThreadScanner {
    running: Option<Receiver<Facts>>,
    done: Option<Facts>,
}

impl Scanner for ThreadScanner {
    fn start(&mut self, input: ScanInput, wake: Box<dyn FnOnce() + Send>) {
        if self.running.is_some() {
            return;
        }
        let (send, receive) = channel();
        let spawned = std::thread::Builder::new()
            .name("diagnostics-scan".to_owned())
            .spawn(move || {
                let _ = send.send(collect(&input));
                wake();
            });
        match spawned {
            Ok(_) => self.running = Some(receive),
            Err(error) => log::warn!("diagnostics: the scan could not start: {error}"),
        }
    }

    fn finished(&mut self) -> Option<Facts> {
        let receive = self.running.as_ref()?;
        match receive.try_recv() {
            Ok(facts) => {
                self.running = None;
                self.done = Some(facts);
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                log::warn!("diagnostics: the scan ended without a result");
                self.running = None;
            }
        }
        self.done.take()
    }

    fn busy(&self) -> bool {
        self.running.is_some()
    }
}
