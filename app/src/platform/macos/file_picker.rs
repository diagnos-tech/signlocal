//! `NSOpenPanel`, modal, for one file of the requested types.

use std::path::PathBuf;

use core_foundation::array::CFArray;
use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};

use super::objc::{AutoreleasePool, Id, class, on_main_thread, send0, send1};
use crate::platform::file_picker::{FileRequest, Picked};

/// `NSModalResponseOK`.
const MODAL_OK: isize = 1;

pub fn choose_file(
    request: FileRequest,
    _parent: Option<isize>,
    done: impl FnOnce(Picked) + Send + 'static,
) {
    done(open_panel(&request));
}

/// Runs the panel; needs the main thread (AppKit).
pub fn open_panel(request: &FileRequest) -> Picked {
    if !on_main_thread() {
        log::warn!("file picker: AppKit needs the main thread");
        return Picked::Unavailable;
    }
    let Some(panel_class) = class(c"NSOpenPanel") else {
        return Picked::Unavailable;
    };
    let types: Vec<CFString> = request
        .extensions
        .iter()
        .map(|extension| CFString::new(extension))
        .collect();
    let types = CFArray::from_CFTypes(&types);
    let message = CFString::new(&request.title);
    let _pool = AutoreleasePool::new();
    // SAFETY: on the main thread; each method exists with the signature
    // used; `setAllowedFileTypes:` takes an `NSArray` of `NSString`
    // (toll-free bridged from `CFArray` of `CFString`) and `setMessage:` an
    // `NSString` (bridged from `CFString`), both retained by the panel;
    // the returned `NSString` path is bridged back under the get rule.
    unsafe {
        let panel: Id = send0(panel_class, c"openPanel");
        if panel.is_null() {
            return Picked::Unavailable;
        }
        send1::<Id, ()>(panel, c"setMessage:", message.as_concrete_TypeRef() as Id);
        send1::<Id, ()>(
            panel,
            c"setAllowedFileTypes:",
            types.as_concrete_TypeRef() as Id,
        );
        if send0::<isize>(panel, c"runModal") != MODAL_OK {
            return Picked::Cancelled;
        }
        let url: Id = send0(panel, c"URL");
        if url.is_null() {
            return Picked::Cancelled;
        }
        let path: Id = send0(url, c"path");
        if path.is_null() {
            return Picked::Cancelled;
        }
        Picked::Chosen(PathBuf::from(
            CFString::wrap_under_get_rule(path as CFStringRef).to_string(),
        ))
    }
}
