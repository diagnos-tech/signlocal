//! UTF-16 strings for Win32 calls.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use windows::Win32::Foundation::HWND;

/// `text` as a NUL-terminated UTF-16 buffer; keep it alive while the
/// pointer is in use.
pub fn wide(text: impl AsRef<OsStr>) -> Vec<u16> {
    text.as_ref().encode_wide().chain([0]).collect()
}

/// The text before the first NUL (or all of it), lossily decoded.
pub fn from_wide(units: &[u16]) -> String {
    let end = units
        .iter()
        .position(|&unit| unit == 0)
        .unwrap_or(units.len());
    String::from_utf16_lossy(&units[..end])
}

/// The owner window for a dialog: `None` for an absent or zero handle.
pub fn window(handle: Option<isize>) -> Option<HWND> {
    handle
        .filter(|&handle| handle != 0)
        .map(|handle| HWND(handle as *mut core::ffi::c_void))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_utf16() {
        let buffer = wide("C:\\Programas\\São Paulo.exe");
        assert_eq!(buffer.last(), Some(&0));
        assert_eq!(from_wide(&buffer), "C:\\Programas\\São Paulo.exe");
        assert_eq!(from_wide(&[0x41, 0, 0x42]), "A");
        assert!(window(Some(0)).is_none());
    }
}
