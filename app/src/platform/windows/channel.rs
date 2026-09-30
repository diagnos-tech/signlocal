//! MSIX or not: a packaged process has a package identity.

use crate::platform::channel::InstallFormat;

// Declared here instead of through the `windows` crate, whose
// `Win32_Storage_Packaging_Appx` feature is not enabled for the app.
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentPackageFullName(length: *mut u32, name: *mut u16) -> i32;
}

const ERROR_INSUFFICIENT_BUFFER: i32 = 122;

pub fn install_format() -> InstallFormat {
    let mut length = 0u32;
    // SAFETY: a size query: no buffer, `length` is a valid out-pointer.
    // Unpackaged processes get `APPMODEL_ERROR_NO_PACKAGE` instead.
    let status = unsafe { GetCurrentPackageFullName(&mut length, std::ptr::null_mut()) };
    if status == ERROR_INSUFFICIENT_BUFFER {
        InstallFormat::Msix
    } else {
        InstallFormat::Archive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runs_are_not_packaged() {
        assert_eq!(install_format(), InstallFormat::Archive);
    }
}
