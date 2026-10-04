//! Small localized pieces of the window: who asks, the window title, names
//! of browsers, hashes and algorithms, and the spoken code.

use websign_core::present::origin::FormattedOrigin;
use websign_i18n::{Catalog, k};
use websign_protocol::types::{BrowserName, HashName, SignatureAlgorithmName};
use websign_ui_model::confirm::ConfirmView;
use websign_ui_model::confirm::port::{CallerView, Mode};
use websign_ui_model::confirm::view::PinSystem;

/// The `{site}` of every message: the host with its port
/// (`app.diagnos.health`), or the program's name.
pub fn site(caller: &CallerView) -> String {
    match caller {
        CallerView::Web { origin, .. } => host(origin).to_owned(),
        CallerView::Desktop { label } => label.name.clone(),
    }
}

/// The host of `origin` with its port, without the scheme.
pub fn host(origin: &FormattedOrigin) -> &str {
    host_of(&origin.canonical)
}

/// `https://app.example.com:8443` → `app.example.com:8443`.
fn host_of(canonical: &str) -> &str {
    canonical
        .split_once("://")
        .map_or(canonical, |(_, host)| host)
}

/// Longest host the OS title shows before cutting subdomains.
const TITLE_HOST: usize = 40;

/// `{site}` for the OS title. Taskbars and window switchers cut long titles
/// at the end, where the registrable domain is, so a long host is cut here
/// from the left instead (`docs/ux.md` §4.3): never the registrable domain
/// or the port. A program's name is already clean and short
/// (`present::caller`).
fn title_site(caller: &CallerView) -> String {
    match caller {
        CallerView::Web { origin, .. } => {
            let port = origin.port.map_or(0, |port| port.to_string().len() + 1);
            cut_left(host(origin), origin.registrable.chars().count() + port)
        }
        CallerView::Desktop { label } => label.name.clone(),
    }
}

/// `host` in at most [`TITLE_HOST`] characters, keeping its last `keep`.
fn cut_left(host: &str, keep: usize) -> String {
    let len = host.chars().count();
    let tail = (TITLE_HOST - 1).max(keep);
    if len <= TITLE_HOST || tail >= len {
        return host.to_owned();
    }
    let kept: String = host.chars().skip(len - tail).collect();
    format!("…{kept}")
}

/// The OS window title: "Sign for {site} — SignLocal", "(1 of 3)" when a
/// queue waits.
pub fn window_title(tr: &Catalog, view: &ConfirmView) -> String {
    let key = match view.mode {
        Mode::Sign { .. } => k::CONFIRM_WINDOW_TITLE,
        Mode::Choose => k::CONFIRM_WINDOW_TITLE_SELECT,
    };
    let title = tr
        .tr(key)
        .arg("site", title_site(&view.header.caller))
        .to_string();
    match view.header.queue {
        Some((current, total)) => tr
            .tr(k::CONFIRM_WINDOW_TITLE_QUEUE)
            .arg("title", title)
            .arg("current", current)
            .arg("total", total)
            .to_string(),
        None => title,
    }
}

/// The short browser name of the eyebrow ("via Chrome"); `None` for a
/// browser without a name we know.
pub fn browser(name: BrowserName) -> Option<&'static str> {
    Some(match name {
        BrowserName::Chrome => "Chrome",
        BrowserName::Chromium => "Chromium",
        BrowserName::Edge => "Edge",
        BrowserName::Brave => "Brave",
        BrowserName::Opera => "Opera",
        BrowserName::Vivaldi => "Vivaldi",
        BrowserName::Firefox => "Firefox",
        BrowserName::Safari => "Safari",
        BrowserName::Other => return None,
    })
}

/// The system whose dialog asks for a key's PIN: "{os} will ask for your
/// PIN in its own window".
pub fn pin_system(tr: &Catalog, system: PinSystem) -> String {
    let key = match system {
        PinSystem::Windows => k::OS_WINDOWS,
        PinSystem::Macos => k::OS_MACOS,
    };
    tr.tr(key).to_string()
}

/// The name of this OS ("Download for {os}").
pub fn this_os(tr: &Catalog) -> String {
    let key = if cfg!(windows) {
        k::OS_WINDOWS
    } else if cfg!(target_os = "macos") {
        k::OS_MACOS
    } else {
        k::OS_LINUX
    };
    tr.tr(key).to_string()
}

pub fn hash(name: HashName) -> &'static str {
    match name {
        HashName::Sha256 => "SHA-256",
        HashName::Sha384 => "SHA-384",
        HashName::Sha512 => "SHA-512",
    }
}

pub fn algorithm(name: SignatureAlgorithmName) -> &'static str {
    match name {
        SignatureAlgorithmName::Ecdsa => "ECDSA",
        SignatureAlgorithmName::RsaPkcs1v15 => "RSASSA-PKCS1-v1_5",
        SignatureAlgorithmName::RsaPss => "RSASSA-PSS",
    }
}

/// "7F3A 9C21" → "7 F 3 A, 9 C 2 1": screen readers would read "7F3A" as a
/// word (`docs/ux.md` §4.4).
pub fn spelled(code: &str) -> String {
    code.split_whitespace()
        .map(|group| {
            let chars: Vec<String> = group.chars().map(String::from).collect();
            chars.join(" ")
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hosts_and_spelled_codes() {
        assert_eq!(host_of("https://app.diagnos.health"), "app.diagnos.health");
        assert_eq!(host_of("http://localhost:5173"), "localhost:5173");
        assert_eq!(spelled("7F3A 9C21"), "7 F 3 A, 9 C 2 1");
    }

    #[test]
    fn long_title_hosts_lose_subdomains_from_the_left() {
        assert_eq!(cut_left("app.diagnos.health", 14), "app.diagnos.health");
        let long = "diagnos.health.login.secure.account.verify.cadastro-medico.com";
        let cut = cut_left(long, "cadastro-medico.com".len());
        assert_eq!(cut.chars().count(), TITLE_HOST);
        assert!(cut.starts_with('…') && cut.ends_with(".verify.cadastro-medico.com"));
        let domain = "a-registrable-domain-longer-than-the-limit-itself.com";
        assert_eq!(
            cut_left(&format!("x.{domain}"), domain.len()),
            format!("…{domain}")
        );
    }
}
