//! The version resource of a Windows executable.

use std::ffi::c_void;
use std::path::Path;

use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VS_FIXEDFILEINFO, VerQueryValueW,
};
use windows::core::{HSTRING, w};

/// `major.minor.build.patch` from `VS_FIXEDFILEINFO`, or `None` when the file
/// has no version resource.
pub fn read(path: &Path) -> Option<String> {
    let name = HSTRING::from(path.as_os_str());
    // SAFETY: `name` is a live NUL-terminated wide string; no handle is asked for.
    let size = unsafe { GetFileVersionInfoSizeW(&name, None) };
    if size == 0 {
        return None;
    }
    let mut block = vec![0u8; size as usize];
    // SAFETY: `block` holds exactly `size` bytes, the length passed.
    unsafe { GetFileVersionInfoW(&name, None, size, block.as_mut_ptr().cast()) }.ok()?;
    let mut info: *mut c_void = std::ptr::null_mut();
    let mut len = 0u32;
    // SAFETY: `block` is the initialized version block; `info` and `len` are
    // live out-pointers. The returned pointer points into `block`.
    let found = unsafe { VerQueryValueW(block.as_ptr().cast(), w!("\\"), &mut info, &mut len) };
    if !found.as_bool() || info.is_null() || (len as usize) < size_of::<VS_FIXEDFILEINFO>() {
        return None;
    }
    // SAFETY: `info` points to at least one `VS_FIXEDFILEINFO` inside
    // `block`, which is still alive; `read_unaligned` makes no alignment
    // assumption.
    let fixed = unsafe { info.cast::<VS_FIXEDFILEINFO>().read_unaligned() };
    let (ms, ls) = (fixed.dwFileVersionMS, fixed.dwFileVersionLS);
    Some(format!(
        "{}.{}.{}.{}",
        ms >> 16,
        ms & 0xffff,
        ls >> 16,
        ls & 0xffff
    ))
}
