//! Recognizing that a browser (not a person) started this process.
//!
//! Promoted from the Phase-0 kit (`probe/src/nm/launch.rs`), reviewed; CI
//! exercised it with Chromium on all three systems and through the MSIX alias.
//!
//! Browsers pass arguments the CLI never would, so they are told apart by
//! shape alone and checked before any command-line parsing:
//!
//! * Chrome, Edge, Brave, Opera, Vivaldi: the caller's origin
//!   (`chrome-extension://<id>/`) first; on Windows also
//!   `--parent-window=<decimal HWND>`.
//! * Firefox: the manifest's path, then the extension ID.
//! * Safari: no browser starts a host; the app extension inside the macOS
//!   app starts its bundled copy of this binary with [`SAFARI_FLAG`] and its
//!   own bundle ID (`safari/SPEC.md` §4). Only a binary that sits in an
//!   `.appex` bundle accepts that shape ([`is_appex_executable`]), so it
//!   never turns a command typed at a terminal into host mode.

use std::ffi::OsString;
use std::fmt;
use std::path::Path;

const CHROMIUM_ORIGIN_PREFIX: &str = "chrome-extension://";
const PARENT_WINDOW_FLAG: &str = "--parent-window=";
/// The first argument of a launch by the Safari app extension.
pub const SAFARI_FLAG: &str = "--safari-web-extension";

/// Browser engine family, which decides how the extension was identified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserFamily {
    /// Chrome, Chromium, Edge, Brave, Vivaldi, Opera.
    Chromium,
    Firefox,
    /// Safari, through the app extension's relay.
    Safari,
    /// Started by hand (tests, `websign connect` debugging).
    Manual,
}

impl BrowserFamily {
    /// Stable name used on the wire and in the log.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Chromium => "chromium",
            Self::Firefox => "firefox",
            Self::Safari => "safari",
            Self::Manual => "manual",
        }
    }
}

impl fmt::Display for BrowserFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How a browser started this process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserLaunch {
    /// The calling extension's origin as the browser reported it: the
    /// `chrome-extension://…/` URL, Firefox's extension ID, or the Safari
    /// app extension's bundle ID.
    pub origin: String,
    pub family: BrowserFamily,
    /// The extension's ID alone (no scheme, no slashes).
    pub extension_id: String,
    /// The browser window that should own dialogs this host opens (Windows,
    /// Chromium family). `None` when absent or zero: Chrome reports zero when
    /// the request comes from a service worker, which every MV3 extension is.
    pub parent_window: Option<isize>,
}

impl BrowserLaunch {
    /// The launch record for a host started by hand.
    pub fn manual() -> Self {
        Self {
            origin: "manual".to_owned(),
            family: BrowserFamily::Manual,
            extension_id: "manual".to_owned(),
            parent_window: None,
        }
    }
}

/// Recognizes the arguments browsers pass when they start a native host.
pub fn detect_browser_launch() -> Option<BrowserLaunch> {
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|arg: OsString| arg.to_string_lossy().into_owned())
        .collect();
    parse_launch_at(running_in_appex(), &args)
}

/// The launch of a host that browsers start directly (Chromium, Firefox),
/// over the arguments after `argv[0]`. Never Safari: see [`parse_launch_at`].
pub fn parse_launch(args: &[String]) -> Option<BrowserLaunch> {
    let first = args.first()?;
    if let Some(rest) = first.strip_prefix(CHROMIUM_ORIGIN_PREFIX) {
        return parse_chromium(first, rest, &args[1..]);
    }
    if args.len() >= 2 && looks_like_manifest_path(first) {
        return parse_firefox(&args[1]);
    }
    None
}

/// Pure core of [`detect_browser_launch`]: [`parse_launch`], plus the Safari
/// shape when `in_appex` (this binary is the app extension's copy).
pub fn parse_launch_at(in_appex: bool, args: &[String]) -> Option<BrowserLaunch> {
    match args {
        [flag, bundle_id] if in_appex && flag == SAFARI_FLAG => parse_safari(bundle_id),
        _ => parse_launch(args),
    }
}

/// Whether `exe` is `<name>.appex/Contents/MacOS/<binary>`: the layout
/// `cargo xtask package` gives the Safari app extension's copy of the host.
pub fn is_appex_executable(exe: &Path) -> bool {
    let macos = exe.parent();
    let contents = macos.and_then(Path::parent);
    let bundle = contents.and_then(Path::parent);
    macos
        .and_then(Path::file_name)
        .is_some_and(|name| name == "MacOS")
        && contents
            .and_then(Path::file_name)
            .is_some_and(|name| name == "Contents")
        && bundle
            .and_then(Path::extension)
            .is_some_and(|ext| ext.eq_ignore_ascii_case("appex"))
}

/// Whether this process is the Safari app extension's copy of the host.
/// Only macOS has app extensions; elsewhere the Safari shape is never read.
pub fn running_in_appex() -> bool {
    cfg!(target_os = "macos") && std::env::current_exe().is_ok_and(|exe| is_appex_executable(&exe))
}

/// A bundle ID: reverse-DNS letters, digits, `-` and `.` (Apple's rule).
fn parse_safari(bundle_id: &str) -> Option<BrowserLaunch> {
    let plausible = !bundle_id.is_empty()
        && bundle_id.len() <= 255
        && !bundle_id.starts_with(['-', '.'])
        && bundle_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.');
    plausible.then(|| BrowserLaunch {
        origin: bundle_id.to_owned(),
        family: BrowserFamily::Safari,
        extension_id: bundle_id.to_owned(),
        parent_window: None,
    })
}

fn parse_chromium(origin: &str, after_scheme: &str, extra: &[String]) -> Option<BrowserLaunch> {
    let id = after_scheme.strip_suffix('/').unwrap_or(after_scheme);
    if !is_chromium_extension_id(id) {
        return None;
    }
    let parent_window = extra
        .iter()
        .find_map(|arg| arg.strip_prefix(PARENT_WINDOW_FLAG))
        .and_then(|handle| handle.parse::<i64>().ok())
        .filter(|&handle| handle != 0)
        .and_then(|handle| isize::try_from(handle).ok());
    Some(BrowserLaunch {
        origin: origin.to_owned(),
        family: BrowserFamily::Chromium,
        extension_id: id.to_owned(),
        parent_window,
    })
}

fn parse_firefox(extension_id: &str) -> Option<BrowserLaunch> {
    let plausible = !extension_id.is_empty()
        && extension_id.len() <= 256
        && !extension_id.starts_with('-')
        && !extension_id
            .chars()
            .any(|c| c.is_control() || c.is_whitespace());
    plausible.then(|| BrowserLaunch {
        origin: extension_id.to_owned(),
        family: BrowserFamily::Firefox,
        extension_id: extension_id.to_owned(),
        parent_window: None,
    })
}

pub use websign_project::is_chromium_extension_id;

/// Firefox passes the manifest's full path; the CLI has no `.json` argument.
fn looks_like_manifest_path(arg: &str) -> bool {
    arg.len() > ".json".len()
        && arg[arg.len() - ".json".len()..].eq_ignore_ascii_case(".json")
        && (arg.contains('/') || arg.contains('\\'))
}

#[cfg(test)]
mod tests;
