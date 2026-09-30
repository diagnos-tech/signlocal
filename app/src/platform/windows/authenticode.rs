//! The Authenticode signer of an executable: its embedded signature, else
//! its entry in a signed catalog (Windows' own programs), verified by
//! `WinVerifyTrust` ([`super::trust`]).
//!
//! The file is opened once and both checks read that handle, shared for
//! reading only when Windows allows it, so the file cannot be rewritten
//! between the checks.
//!
//! Residual risk (documented, not solved): the path comes from
//! `QueryFullProcessImageNameW`, the name the image was started from. A
//! program that renames its own running file and puts a signed one under
//! the old name would be shown with that signer. Doing so already requires
//! running code as the person, which `docs/architecture/security.md` puts
//! out of scope; the check still stops the common case of an unsigned
//! program claiming a trusted name.

use std::fs::File;
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::AsRawHandle;
use std::path::Path;

use windows::Win32::Foundation::HANDLE;
use windows::Win32::Security::WinTrust::WINTRUST_FILE_INFO;
use windows::Win32::Storage::FileSystem::FILE_SHARE_READ;
use windows::core::PCWSTR;

use super::catalog;
use super::trust::{Subject, verified_signer};
use super::wide::wide;

/// The verified signer's subject CN; `None` when unsigned or not trusted.
pub fn signer(executable: &Path) -> Option<String> {
    let file = open(executable)?;
    embedded(executable, &file).or_else(|| catalog::signer(executable, &file))
}

/// Opened for reading, refusing writers while open when possible (another
/// process may hold the file in a way that forbids that; then shared).
fn open(executable: &Path) -> Option<File> {
    File::options()
        .read(true)
        .share_mode(FILE_SHARE_READ.0)
        .open(executable)
        .or_else(|_| File::open(executable))
        .ok()
}

fn embedded(executable: &Path, file: &File) -> Option<String> {
    let path = wide(executable);
    let mut info = WINTRUST_FILE_INFO {
        cbStruct: size_of::<WINTRUST_FILE_INFO>() as u32,
        pcwszFilePath: PCWSTR(path.as_ptr()),
        hFile: HANDLE(file.as_raw_handle()),
        ..Default::default()
    };
    verified_signer(Subject::File(&mut info))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_file_has_no_signer() {
        assert_eq!(signer(Path::new("C:\\no\\such\\file.exe")), None);
    }
}
