//! The parent process on macOS.
//!
//! **Which process.** The PID from `getppid`. A PID alone could name another
//! program if the parent exited and the number was reused while we looked;
//! but an exited parent changes `getppid` for good (we are reparented to
//! launchd), so reading it again after every check proves the PID named our
//! parent throughout.
//!
//! **Which code.** The code signature is checked on the running process
//! (`code_signature`), by audit token when stdin is a Unix socket whose peer
//! is the parent (libuv-based callers such as Node and Electron use socket
//! pairs), else by PID. The token also pins the executable image, so a
//! parent that `exec`s another program during the check is not taken for
//! the one it replaced. Residual risk with the PID (plain pipes, which is
//! what Rust, Python and Go callers use): a parent that `exec`s a different
//! program right after the check. That is the parent choosing to become
//! another program on its own pipes, not a third party spoofing it, and it
//! requires running code as the person (out of scope in
//! `docs/architecture/security.md`).

use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;

use websign_core::present::caller::DesktopCaller;

use super::code_signature::{self, Guest};
use super::{bundle, peer};

pub fn parent_caller() -> Option<DesktopCaller> {
    let parent = std::os::unix::process::parent_id();
    // 1 (launchd): the program that started us is gone and we were
    // reparented; reporting launchd would name the wrong program.
    if parent <= 1 {
        return None;
    }
    let pid = i32::try_from(parent).ok()?;
    let guest = peer::stdin_audit_token()
        .filter(|token| token.pid() == pid)
        .map_or(Guest::Pid(pid), Guest::AuditToken);
    let executable = executable_of(pid)?;
    let caller = DesktopCaller {
        product_name: bundle::product_name(&executable),
        signer: code_signature::signer(&guest, &executable),
        executable,
    };
    (std::os::unix::process::parent_id() == parent).then_some(caller)
}

fn executable_of(pid: i32) -> Option<PathBuf> {
    let mut buffer = vec![0u8; libc::PROC_PIDPATHINFO_MAXSIZE as usize];
    // SAFETY: `buffer` has the size passed; the kernel writes at most that
    // and returns the length written.
    let len = unsafe { libc::proc_pidpath(pid, buffer.as_mut_ptr().cast(), buffer.len() as u32) };
    let len = usize::try_from(len).ok().filter(|&len| len > 0)?;
    buffer.truncate(len);
    Some(PathBuf::from(OsString::from_vec(buffer)))
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
