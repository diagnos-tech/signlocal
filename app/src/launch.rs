//! Telling apart the ways this binary is started (see `main.rs`).

use websign_host::BrowserLaunch;

/// How this process was started.
#[derive(Debug)]
pub enum Launch {
    /// A browser, for our extension.
    Browser(BrowserLaunch),
    /// The OS, for a `websign:` URL (the single argument).
    Url(String),
    /// Anything else: parse the command line.
    Command,
}

/// Inspects `argv` (browser shapes first, then a lone `websign:` URL).
pub fn detect() -> Launch {
    if let Some(browser) = websign_host::detect_browser_launch() {
        return Launch::Browser(browser);
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [url] if is_scheme_url(url) => Launch::Url(url.clone()),
        _ => Launch::Command,
    }
}

/// `websign:…`, case-insensitively, as the OS passes it.
fn is_scheme_url(arg: &str) -> bool {
    let scheme = websign_project::URL_SCHEME;
    arg.len() > scheme.len()
        && arg.as_bytes()[scheme.len()] == b':'
        && arg[..scheme.len()].eq_ignore_ascii_case(scheme)
}
