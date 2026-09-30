//! Telling apart the ways this binary is started (see `main.rs`).
//!
//! Browsers and the OS pass arguments no person would type, so they are
//! recognized by shape before the command line is parsed: a browser's
//! origin or manifest path must never be read as a (mistyped) command.
//!
//! Host mode is never entered by accident: a browser-shaped argument list
//! counts only when it names one of our extensions (`project.toml`, the same
//! IDs the manifests' `allowed_origins`/`allowed_extensions` list). Anything
//! else goes to the command-line parser, which answers with a usage error
//! (exit 2) instead of serving a stranger on stdio.

#[cfg(test)]
mod tests;
mod url;

use websign_host::{BrowserFamily, BrowserLaunch};

pub use url::{UrlAction, parse as parse_url};

/// How this process was started.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Launch {
    /// A browser, for our extension (native messaging on stdio).
    Browser(BrowserLaunch),
    /// The OS, for a `websign:` URL (the single argument).
    Url(UrlAction),
    /// `websign connect`: a program speaking the protocol on stdio.
    Connect,
    /// No arguments (Start menu, Launchpad, app menu): diagnostics.
    Gui,
    /// Anything else: parse the command line.
    Command,
}

/// Classifies this process's own arguments.
pub fn detect() -> Launch {
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned())
        .filter(|arg| !is_launch_services_serial(arg))
        .collect();
    classify(&args)
}

/// Pure core of [`detect`], over the arguments after `argv[0]`.
pub fn classify(args: &[String]) -> Launch {
    if let Some(browser) = websign_host::parse_launch(args).filter(is_ours) {
        return Launch::Browser(browser);
    }
    match args {
        [] => Launch::Gui,
        [only] if only == "connect" => Launch::Connect,
        [only] => url::parse(only).map_or(Launch::Command, Launch::Url),
        _ => Launch::Command,
    }
}

/// Whether `launch` names one of this product's extensions. The engine
/// checks again at `hello`; refusing here keeps a foreign caller from even
/// starting the host threads.
fn is_ours(launch: &BrowserLaunch) -> bool {
    match launch.family {
        BrowserFamily::Chromium => {
            websign_project::chromium_extension_ids().contains(&launch.extension_id.as_str())
        }
        BrowserFamily::Firefox => launch.extension_id == websign_project::FIREFOX_ID,
        BrowserFamily::Manual => false,
    }
}

/// `-psn_0_12345`: the process serial number older macOS versions add when
/// Launch Services starts an app (Finder, Launchpad, a `websign:` URL). It is
/// not an argument the person gave, so it must not turn a plain start into a
/// usage error.
fn is_launch_services_serial(arg: &str) -> bool {
    cfg!(target_os = "macos") && arg.starts_with("-psn_")
}
