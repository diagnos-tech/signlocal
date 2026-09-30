//! The OS file picker behind "Add driver…" (`docs/ux.md` §8.4) and, on
//! macOS, "Import .pfx file…" (§8.5). The dialog answers later on Linux
//! (another program, another thread), so its outcome comes back through a
//! channel the window reads each frame.

use std::sync::mpsc::{Receiver, TryRecvError, channel};

use websign_i18n::{Catalog, k};

use super::services::{Os, Owner};
use crate::platform::file_picker::{FileRequest, Picked};

/// What the chosen file is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Purpose {
    Driver,
    Pfx,
}

/// A dialog's outcome, for its purpose.
pub type Outcome = (Purpose, Picked);

/// The dialog the window opened, until it answers.
#[derive(Debug, Default)]
pub struct Picks {
    pending: Option<(Purpose, Receiver<Picked>)>,
}

impl Picks {
    /// Opens the dialog for `purpose` unless one is open; `wake` runs a
    /// frame once it answers.
    pub fn open(
        &mut self,
        os: &mut dyn Os,
        catalog: &Catalog,
        purpose: Purpose,
        owner: Owner,
        wake: impl FnOnce() + Send + 'static,
    ) {
        if self.pending.is_some() {
            return;
        }
        let (answer, answered) = channel();
        self.pending = Some((purpose, answered));
        let done = Box::new(move |picked| {
            // The window may have closed meanwhile; nobody is left to tell.
            let _ = answer.send(picked);
            wake();
        });
        os.choose_file(request(catalog, purpose), owner, done);
    }

    /// A dialog is waiting for the person.
    pub fn is_open(&self) -> bool {
        self.pending.is_some()
    }

    /// The answer that arrived since the last frame, once.
    pub fn answered(&mut self) -> Option<Outcome> {
        let (purpose, pending) = self.pending.as_ref()?;
        let picked = match pending.try_recv() {
            Ok(picked) => picked,
            Err(TryRecvError::Empty) => return None,
            // The dialog went away without answering (it could not start).
            Err(TryRecvError::Disconnected) => Picked::Unavailable,
        };
        let purpose = *purpose;
        self.pending = None;
        Some((purpose, picked))
    }
}

/// The dialog's localized title and filter.
fn request(catalog: &Catalog, purpose: Purpose) -> FileRequest {
    let (title, type_name, extensions) = match purpose {
        Purpose::Driver => (
            k::DEVICES_DRIVER_PICK_TITLE,
            k::DEVICES_DRIVER_FILE_TYPE,
            vec![driver_extension()],
        ),
        Purpose::Pfx => (
            k::CERTS_TAB_IMPORT_PICK_TITLE,
            k::CERTS_TAB_PFX_FILE_TYPE,
            vec!["pfx", "p12"],
        ),
    };
    FileRequest {
        title: catalog.tr(title).to_string(),
        type_name: catalog.tr(type_name).to_string(),
        extensions,
    }
}

/// What a token driver (PKCS#11 module) file ends with on this OS.
fn driver_extension() -> &'static str {
    if cfg!(windows) {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    }
}
