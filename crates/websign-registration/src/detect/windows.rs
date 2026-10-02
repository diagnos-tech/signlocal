//! Windows: the browsers registered for "Default apps"
//! (`Software\Clients\StartMenuInternet`) and the executables registered in
//! `App Paths`, in `HKCU` and `HKLM`.

use std::path::Path;

use super::{BrowserPackaging, InstalledBrowser, ordered};
use crate::browsers::Browser;
use crate::registry::{Hive, Registry};

const START_MENU: &str = r"Software\Clients\StartMenuInternet";
const APP_PATHS: &str = r"Software\Microsoft\Windows\CurrentVersion\App Paths";
const APP_PATH_EXECUTABLES: [&str; 6] = [
    "chrome.exe",
    "msedge.exe",
    "firefox.exe",
    "brave.exe",
    "vivaldi.exe",
    "opera.exe",
];

/// `version_of` reads an executable's version resource.
pub fn detect(
    registry: &dyn Registry,
    version_of: &dyn Fn(&Path) -> Option<String>,
) -> Vec<InstalledBrowser> {
    let mut found = Vec::new();
    for executable in executables(registry) {
        if let Some(browser) = browser_of(&executable) {
            found.push(InstalledBrowser {
                browser,
                version: version_of(Path::new(&executable)),
                packaging: BrowserPackaging::Native,
            });
        }
    }
    ordered(found)
}

/// Executable paths named by the registry, per-user entries first.
fn executables(registry: &dyn Registry) -> Vec<String> {
    let mut commands = Vec::new();
    for hive in [Hive::CurrentUser, Hive::LocalMachine] {
        for client in registry.subkey_names(hive, START_MENU).unwrap_or_default() {
            let subkey = format!(r"{START_MENU}\{client}\shell\open\command");
            commands.extend(registry.get_string(hive, &subkey, "").ok().flatten());
        }
        for name in APP_PATH_EXECUTABLES {
            let subkey = format!(r"{APP_PATHS}\{name}");
            commands.extend(registry.get_string(hive, &subkey, "").ok().flatten());
        }
    }
    commands
        .iter()
        .filter_map(|command| executable_of(command))
        .collect()
}

/// The program of a command line: `"C:\x\chrome.exe" --flag` → `C:\x\chrome.exe`.
fn executable_of(command: &str) -> Option<String> {
    let command = command.trim();
    let path = match command.strip_prefix('"') {
        Some(rest) => rest.split('"').next()?,
        None => {
            let end = command.to_ascii_lowercase().find(".exe")? + ".exe".len();
            &command[..end]
        }
    };
    (!path.is_empty()).then(|| path.to_owned())
}

/// Chromium ships `chrome.exe` too, so its folder tells them apart; Opera
/// registers `launcher.exe` inside its own folder.
fn browser_of(executable: &str) -> Option<Browser> {
    let lower = executable.to_ascii_lowercase().replace('/', "\\");
    let file = lower.rsplit('\\').next()?;
    Some(match file {
        "chrome.exe" if lower.contains("\\chromium\\") => Browser::Chromium,
        "chrome.exe" => Browser::Chrome,
        "msedge.exe" => Browser::Edge,
        "firefox.exe" => Browser::Firefox,
        "brave.exe" => Browser::Brave,
        "vivaldi.exe" => Browser::Vivaldi,
        "opera.exe" => Browser::Opera,
        "launcher.exe" if lower.contains("\\opera") => Browser::Opera,
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
