//! `RtlGetVersion`: the real version numbers, whatever the manifest says.

use windows::Wdk::System::SystemServices::RtlGetVersion;
use windows::Win32::System::SystemInformation::OSVERSIONINFOW;

/// `(major, minor, build)`; `None` when the call fails.
pub fn numbers() -> Option<(u32, u32, u32)> {
    let mut info = OSVERSIONINFOW {
        dwOSVersionInfoSize: size_of::<OSVERSIONINFOW>() as u32,
        ..Default::default()
    };
    // SAFETY: `info` is a valid, writable `OSVERSIONINFOW` whose size field
    // is set, as the call requires.
    let status = unsafe { RtlGetVersion(&mut info) };
    status
        .is_ok()
        .then_some((info.dwMajorVersion, info.dwMinorVersion, info.dwBuildNumber))
}
