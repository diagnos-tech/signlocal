//! Certificate viewer and URLs on Linux; there is no OS `.pfx` import.

use std::path::Path;
use std::time::Duration;

use super::command::launch;
use super::private_file;

/// GCR's viewer (GNOME's certificate viewer; `gcr-viewer-gtk4` in GCR 4).
const VIEWERS: [&str; 2] = ["gcr-viewer", "gcr-viewer-gtk4"];

/// `xdg-open` returns once the handler started, before it read the file;
/// the file stays this long, then goes.
const HANDLER_GRACE: Duration = Duration::from_secs(120);

pub fn view_certificate(der: &[u8], parent: Option<isize>) -> bool {
    let _ = parent;
    let Some(file) = private_file::write("certificate", "crt", der) else {
        return false;
    };
    let Some(arg) = file.to_str().map(str::to_owned) else {
        remove(&file);
        return false;
    };
    let opened = VIEWERS.iter().any(|viewer| {
        let file = file.clone();
        // The viewer holds the window until closed: remove the file then.
        launch(viewer, &[&arg], move || remove(&file))
    }) || {
        let file = file.clone();
        launch("xdg-open", &[&arg], move || {
            std::thread::sleep(HANDLER_GRACE);
            remove(&file);
        })
    };
    if !opened {
        remove(&file);
        log::info!("certificate viewer: no gcr-viewer or xdg-open");
    }
    opened
}

/// Linux has no user certificate store every program reads, hence no
/// import (see [`crate::platform::system_ui::import_pfx`]).
pub fn import_pfx(file: Option<&Path>, parent: Option<isize>) -> bool {
    let _ = (file, parent);
    false
}

pub fn open_url(url: &str) -> bool {
    launch("xdg-open", &[url], || {})
}

fn remove(file: &Path) {
    let _ = std::fs::remove_file(file);
}
