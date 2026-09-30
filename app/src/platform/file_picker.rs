//! The OS "open file" dialog (`docs/ux.md` §8.4 "Add driver…", §8.5 the
//! `.pfx` import on macOS): `IFileOpenDialog` on Windows, `NSOpenPanel` on
//! macOS, the desktop's own dialog on Linux (`zenity` or `kdialog`).
//!
//! The person picks the file; the app never browses the disk itself, so a
//! driver path is always one the person chose in a dialog they know.

use std::path::PathBuf;

/// What to ask for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRequest {
    /// The dialog's title, localized.
    pub title: String,
    /// The name of the accepted file type, localized ("Token driver").
    pub type_name: String,
    /// Accepted extensions without the dot (`["dll"]`, `["so"]`). On Linux
    /// `so` also accepts versioned names (`libx.so.1`).
    pub extensions: Vec<&'static str>,
}

/// How the dialog ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Picked {
    Chosen(PathBuf),
    Cancelled,
    /// This computer has no dialog the app can show (a Linux desktop
    /// without `zenity` or `kdialog`); the caller offers a path field.
    Unavailable,
}

/// Shows the dialog and calls `done` with the outcome.
///
/// Windows and macOS show it modally over `parent` (`HWND` on Windows;
/// unused elsewhere) and call `done` before returning: call this on the UI
/// thread, as AppKit requires. On Linux the dialog is another program, so
/// it runs on a worker thread and `done` is called from there once it
/// closes; the window keeps drawing meanwhile.
pub fn choose_file(
    request: FileRequest,
    parent: Option<isize>,
    done: impl FnOnce(Picked) + Send + 'static,
) {
    super::os::file_picker::choose_file(request, parent, done);
}
