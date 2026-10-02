//! `IFileOpenDialog`, modal over the owner window, for one file system
//! file of the requested types.

use std::path::PathBuf;

use windows::Win32::Foundation::{ERROR_CANCELLED, RPC_E_CHANGED_MODE};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoCreateInstance,
    CoInitializeEx, CoTaskMemFree, CoUninitialize,
};
use windows::Win32::UI::Shell::Common::COMDLG_FILTERSPEC;
use windows::Win32::UI::Shell::{
    FOS_FILEMUSTEXIST, FOS_FORCEFILESYSTEM, FileOpenDialog, IFileOpenDialog, SIGDN_FILESYSPATH,
};
use windows::core::{HRESULT, PCWSTR};

use super::wide::{wide, window};
use crate::platform::file_picker::{FileRequest, Picked};

pub fn choose_file(
    request: FileRequest,
    parent: Option<isize>,
    done: impl FnOnce(Picked) + Send + 'static,
) {
    let Some(_com) = Apartment::enter() else {
        done(Picked::Unavailable);
        return;
    };
    let picked = match show(&request, parent) {
        Ok(picked) => picked,
        Err(error) if error.code() == HRESULT::from_win32(ERROR_CANCELLED.0) => Picked::Cancelled,
        Err(error) => {
            log::warn!("file picker: failed (0x{:08X})", error.code().0);
            Picked::Unavailable
        }
    };
    done(picked);
}

fn show(request: &FileRequest, parent: Option<isize>) -> windows::core::Result<Picked> {
    let title = wide(&request.title);
    let name = wide(&request.type_name);
    let patterns = request
        .extensions
        .iter()
        .map(|extension| format!("*.{extension}"))
        .collect::<Vec<_>>()
        .join(";");
    let patterns = wide(patterns);
    let filters = [COMDLG_FILTERSPEC {
        pszName: PCWSTR(name.as_ptr()),
        pszSpec: PCWSTR(patterns.as_ptr()),
    }];
    // SAFETY: COM is initialized on this thread (`Apartment`); the strings
    // and the filter array outlive every call that reads them (the dialog
    // copies them); the returned path is freed once with `CoTaskMemFree`.
    unsafe {
        let dialog: IFileOpenDialog =
            CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
        dialog.SetTitle(PCWSTR(title.as_ptr()))?;
        dialog.SetFileTypes(&filters)?;
        let options = dialog.GetOptions()?;
        dialog.SetOptions(options | FOS_FILEMUSTEXIST | FOS_FORCEFILESYSTEM)?;
        dialog.Show(window(parent))?;
        let item = dialog.GetResult()?;
        let path = item.GetDisplayName(SIGDN_FILESYSPATH)?;
        let chosen = path.to_string();
        CoTaskMemFree(Some(path.0 as *const _));
        Ok(match chosen {
            Ok(chosen) if !chosen.is_empty() => Picked::Chosen(PathBuf::from(chosen)),
            _ => Picked::Cancelled,
        })
    }
}

/// COM on this thread for the dialog's lifetime. The UI thread usually
/// has it already (winit initializes OLE for drag and drop); then this
/// neither initializes nor uninitializes it.
struct Apartment {
    owned: bool,
}

impl Apartment {
    fn enter() -> Option<Apartment> {
        // SAFETY: no pointers; balanced by `drop` when it succeeded.
        let result =
            unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) };
        if result == RPC_E_CHANGED_MODE {
            // Multithreaded COM: the dialog still works there.
            return Some(Apartment { owned: false });
        }
        if result.is_err() {
            log::warn!("file picker: COM unavailable (0x{:08X})", result.0);
            return None;
        }
        Some(Apartment { owned: true })
    }
}

impl Drop for Apartment {
    fn drop(&mut self) {
        if self.owned {
            // SAFETY: pairs the successful `CoInitializeEx` (S_OK or
            // S_FALSE) of this thread.
            unsafe { CoUninitialize() };
        }
    }
}
