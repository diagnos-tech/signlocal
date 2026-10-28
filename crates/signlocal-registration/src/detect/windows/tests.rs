use super::*;
use crate::registry::MemoryRegistry;

fn start_menu(registry: &MemoryRegistry, hive: Hive, client: &str, command: &str) {
    registry.seed(
        hive,
        &format!(r"{START_MENU}\{client}\shell\open\command"),
        "",
        command,
    );
}

fn version_of(path: &Path) -> Option<String> {
    path.to_string_lossy()
        .contains("Chrome")
        .then(|| "130.0.1.2".to_owned())
}

#[test]
fn start_menu_clients_and_app_paths_are_combined_without_duplicates() {
    let registry = MemoryRegistry::new();
    let chrome = r"C:\Program Files\Google\Chrome\Application\chrome.exe";
    start_menu(
        &registry,
        Hive::LocalMachine,
        "Google Chrome",
        &format!("\"{chrome}\""),
    );
    start_menu(
        &registry,
        Hive::LocalMachine,
        "FIREFOX.EXE",
        r#""C:\Program Files\Mozilla Firefox\firefox.exe" -osint"#,
    );
    start_menu(
        &registry,
        Hive::CurrentUser,
        "Chromium.ABC",
        r"C:\Users\u\AppData\Local\Chromium\Application\chrome.exe",
    );
    registry.seed(
        Hive::LocalMachine,
        &format!(r"{APP_PATHS}\chrome.exe"),
        "",
        chrome,
    );
    registry.seed(
        Hive::CurrentUser,
        &format!(r"{APP_PATHS}\opera.exe"),
        "",
        r"C:\Users\u\AppData\Local\Programs\Opera\opera.exe",
    );

    let found = detect(&registry, &version_of);
    let browsers: Vec<Browser> = found.iter().map(|entry| entry.browser).collect();
    assert_eq!(
        browsers,
        [
            Browser::Chrome,
            Browser::Chromium,
            Browser::Opera,
            Browser::Firefox
        ]
    );
    assert_eq!(found[0].version.as_deref(), Some("130.0.1.2"));
    assert!(
        found
            .iter()
            .all(|e| e.packaging == BrowserPackaging::Native)
    );
}

#[test]
fn command_lines_are_reduced_to_the_executable() {
    assert_eq!(
        executable_of(r#""C:\a b\msedge.exe" --x"#).as_deref(),
        Some(r"C:\a b\msedge.exe")
    );
    assert_eq!(
        executable_of(r"C:\b\Brave.EXE --y").as_deref(),
        Some(r"C:\b\Brave.EXE")
    );
    assert_eq!(executable_of("\"\""), None);
    assert_eq!(executable_of("no program here"), None);
}

#[test]
fn unknown_programs_are_ignored() {
    assert_eq!(browser_of(r"C:\x\iexplore.exe"), None);
    assert_eq!(browser_of(r"C:\x\launcher.exe"), None);
    assert_eq!(
        browser_of(r"C:\Users\u\AppData\Local\Programs\Opera GX\launcher.exe"),
        Some(Browser::Opera)
    );
}

#[test]
fn an_empty_registry_finds_nothing() {
    assert!(detect(&MemoryRegistry::new(), &|_| None).is_empty());
}
