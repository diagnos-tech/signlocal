//! Identifying the program that started `websign connect`/`sign`/`choose`.
//!
//! The caller is the **parent process**, found by the OS, never by anything
//! the program sends:
//!
//! * Windows: parent PID (Toolhelp snapshot, checked to have started before
//!   us so a reused PID is not taken for the parent),
//!   `QueryFullProcessImageNameW`, version resource
//!   `FileDescription`/`ProductName`, Authenticode signer via
//!   `WinVerifyTrust` (embedded signature, else a signed catalog, which is
//!   how Windows' own programs are signed: "Microsoft Windows") + the leaf
//!   certificate's subject CN.
//! * macOS: `proc_pidpath` of the parent, the enclosing `.app`'s
//!   `CFBundleName`, `SecCodeCopyGuestWithAttributes` (audit token of the
//!   stdin peer when there is one, else PID) + `SecCodeCheckValidity` for
//!   team ID and identifier; `getppid` re-read afterwards against PID reuse.
//! * Linux: `/proc/<ppid>/exe`; product name from a `.desktop` file whose
//!   `Exec` matches; no signer (packages are verified at install time, not
//!   per executable, so there is nothing to show as "Signed by");
//!   `getppid` re-read afterwards against PID reuse.
//!
//! A parent that is a shell (`bash`, `zsh`, `cmd`, `powershell`) is reported
//! as itself: the person started the command.
//!
//! Nothing here logs the executable path: it usually contains the user name.

use websign_core::present::caller::DesktopCaller;

/// The calling program; `None` when the parent cannot be read (then the
/// request is refused with `Internal`).
pub fn parent_caller() -> Option<DesktopCaller> {
    let caller = super::os::caller::parent_caller();
    match &caller {
        Some(found) => log::info!("caller identified; signed: {}", found.signer.is_some()),
        None => log::warn!("caller: parent process could not be read"),
    }
    caller
}
