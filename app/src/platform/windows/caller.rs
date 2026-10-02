//! The parent process on Windows.
//!
//! Windows keeps the parent's PID after the parent exits, and PIDs are
//! reused, so the process found under that PID must have started before we
//! did; otherwise it is an unrelated program that got the number.

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::path::PathBuf;

use websign_core::present::caller::{CodeSigner, DesktopCaller};
use windows::Win32::Foundation::{FILETIME, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetProcessTimes, OpenProcess, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::core::PWSTR;

use super::{authenticode, version_info};

/// Long paths (`\\?\`) can reach 32 767 UTF-16 units.
const MAX_PATH_UNITS: usize = 32_768;

pub fn parent_caller() -> Option<DesktopCaller> {
    let parent = open_parent()?;
    let executable = image_path(&parent)?;
    Some(DesktopCaller {
        product_name: version_info::product_name(&executable),
        signer: authenticode::signer(&executable)
            .map(|subject| CodeSigner::Authenticode { subject }),
        executable,
    })
}

fn open_parent() -> Option<OwnedHandle> {
    let pid = parent_pid()?;
    // SAFETY: plain call; on success we own the returned handle.
    let raw = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
    // SAFETY: `raw` is a valid process handle nobody else closes.
    let parent = unsafe { OwnedHandle::from_raw_handle(raw.0) };
    // SAFETY: the pseudo-handle of the current process needs no closing.
    let own = unsafe { GetCurrentProcess() };
    (started_at(handle(&parent))? <= started_at(own)?).then_some(parent)
}

fn parent_pid() -> Option<u32> {
    let own = std::process::id();
    // SAFETY: plain call; on success we own the snapshot handle.
    let raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }.ok()?;
    // SAFETY: `raw` is a valid snapshot handle nobody else closes.
    let snapshot = unsafe { OwnedHandle::from_raw_handle(raw.0) };
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    // SAFETY: `entry` is writable and its `dwSize` is set, as required.
    unsafe { Process32FirstW(handle(&snapshot), &mut entry) }.ok()?;
    loop {
        if entry.th32ProcessID == own {
            return Some(entry.th32ParentProcessID);
        }
        // SAFETY: as above; an error ends the walk.
        unsafe { Process32NextW(handle(&snapshot), &mut entry) }.ok()?;
    }
}

fn started_at(process: HANDLE) -> Option<u64> {
    let (mut created, mut exited, mut kernel, mut user) = Default::default();
    // SAFETY: `process` is open with query rights; the four out-pointers
    // are valid `FILETIME`s.
    unsafe { GetProcessTimes(process, &mut created, &mut exited, &mut kernel, &mut user) }.ok()?;
    Some(ticks(created))
}

fn ticks(time: FILETIME) -> u64 {
    (u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime)
}

fn image_path(process: &OwnedHandle) -> Option<PathBuf> {
    let mut buffer = vec![0u16; MAX_PATH_UNITS];
    let mut len = buffer.len() as u32;
    // SAFETY: `buffer` holds `len` units; Windows writes at most that many
    // and stores the length written (without the NUL) in `len`.
    unsafe {
        QueryFullProcessImageNameW(
            handle(process),
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut len,
        )
    }
    .ok()?;
    buffer.truncate(len as usize);
    // Not `from_wide`: a path may hold unpaired surrogates, kept as is.
    Some(PathBuf::from(OsString::from_wide(&buffer)))
}

fn handle(owned: &OwnedHandle) -> HANDLE {
    HANDLE(owned.as_raw_handle())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_test_runner_has_a_readable_parent() {
        let caller = parent_caller().expect("cargo is our parent");
        assert!(caller.executable.is_absolute());
    }
}
