//! The parent process on Linux: `/proc/<ppid>/exe` and a matching `.desktop`
//! entry for its name.

use std::path::PathBuf;

use websign_core::present::caller::DesktopCaller;

use super::desktop_entry;

pub fn parent_caller() -> Option<DesktopCaller> {
    let parent = std::os::unix::process::parent_id();
    // 0 or 1 (init): the program that started us is gone and we were
    // reparented; reporting init would name the wrong program.
    if parent <= 1 {
        return None;
    }
    let executable = executable_of(parent)?;
    let caller = DesktopCaller {
        product_name: desktop_entry::product_name(&executable),
        executable,
        signer: None,
    };
    // A parent that exited while we looked (its PID possibly reused by
    // another program) changes our parent PID for good: reading it again
    // proves the PID named our parent throughout.
    (std::os::unix::process::parent_id() == parent).then_some(caller)
}

/// The real path of the program `pid` runs, as the kernel recorded it at
/// `execve`: symlinks resolved, and not something the program can set
/// (unlike `argv[0]` or `/proc/<pid>/comm`). A binary replaced since keeps
/// the kernel's ` (deleted)` suffix, which is shown as is.
fn executable_of(pid: u32) -> Option<PathBuf> {
    std::fs::read_link(format!("/proc/{pid}/exe")).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_test_runner_has_a_readable_parent() {
        let caller = parent_caller().expect("the test process has a live parent");
        assert!(caller.executable.is_absolute());
        assert_eq!(caller.signer, None);
    }

    #[test]
    fn our_own_executable_is_the_current_exe() {
        let own = executable_of(std::process::id()).expect("own /proc entry");
        assert_eq!(Some(own), std::env::current_exe().ok());
    }
}
