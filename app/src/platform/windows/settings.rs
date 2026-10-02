//! Reduce motion and dark mode on Windows.

use std::ffi::c_void;

use windows::Win32::UI::WindowsAndMessaging::{
    SPI_GETCLIENTAREAANIMATION, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW,
};
use windows::core::{BOOL, w};

// Declared here instead of through the `windows` crate, whose
// `Win32_System_Registry` feature is not enabled for the app.
#[link(name = "advapi32")]
unsafe extern "system" {
    fn RegGetValueW(
        key: isize,
        sub_key: *const u16,
        value: *const u16,
        flags: u32,
        kind: *mut u32,
        data: *mut c_void,
        size: *mut u32,
    ) -> u32;
}

/// `HKEY_CURRENT_USER`: `(HKEY)(ULONG_PTR)(LONG)0x80000001`, sign-extended.
const HKEY_CURRENT_USER: isize = 0x8000_0001_u32 as i32 as isize;
const RRF_RT_REG_DWORD: u32 = 0x10;

/// Settings › Accessibility › Visual effects › "Animation effects".
pub fn reduce_motion() -> bool {
    let mut enabled = BOOL(1);
    // SAFETY: this action writes one `BOOL` into the pointer given.
    let read = unsafe {
        SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            Some((&raw mut enabled).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    };
    read.is_ok() && !enabled.as_bool()
}

/// Settings › Personalization › Colors › "Choose your app mode".
pub fn dark_mode() -> Option<bool> {
    let mut value = 0u32;
    let mut size = size_of::<u32>() as u32;
    // SAFETY: the strings are NUL-terminated literals; `RRF_RT_REG_DWORD`
    // makes Windows write exactly one `u32` into `value` (of `size` bytes).
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize").as_ptr(),
            w!("AppsUseLightTheme").as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            (&raw mut value).cast(),
            &mut size,
        )
    };
    (status == 0).then_some(value == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queries_answer_without_failing() {
        let _ = reduce_motion();
        let _ = dark_mode();
    }
}
