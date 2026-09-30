//! Identifying the program that started `websign connect`/`sign`/`choose`.
//!
//! The caller is the **parent process**, found by the OS, never by anything
//! the program sends:
//!
//! * Windows: parent PID (Toolhelp snapshot), `QueryFullProcessImageNameW`,
//!   version resource `FileDescription`/`ProductName`, Authenticode signer via
//!   `WinVerifyTrust` + the leaf certificate's subject CN.
//! * macOS: `proc_pidpath` of the parent, the enclosing `.app`'s
//!   `CFBundleName`, `SecCodeCopyGuestWithAttributes` + `SecCodeCheckValidity`
//!   for team ID and identifier.
//! * Linux: `/proc/<ppid>/exe`; product name from a `.desktop` file whose
//!   `Exec` matches; no signer.
//!
//! A parent that is a shell (`bash`, `zsh`, `cmd`, `powershell`) is reported
//! as itself: the person started the command.

use websign_core::present::caller::DesktopCaller;

/// The calling program; `None` when the parent cannot be read (then the
/// request is refused with `Internal`).
pub fn parent_caller() -> Option<DesktopCaller> {
    todo!("desktop-api.md §Caller identity")
}
