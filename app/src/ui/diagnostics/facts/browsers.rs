//! Browsers on this computer, their registration and the extension's last
//! connection (`docs/ux.md` §8.3). Detected from install locations, never
//! from browser profiles.

use websign_host::store::ConnectionRecord;
use websign_protocol::types::BrowserName;
use websign_registration::Browser;
use websign_registration::detect::installed_browsers;
use websign_registration::status::registration_state;

use super::BrowserFact;

/// Every installed browser with its registration and connection.
pub fn collect(connections: &[ConnectionRecord]) -> Vec<BrowserFact> {
    installed_browsers()
        .into_iter()
        .map(|installed| BrowserFact {
            browser: installed.browser,
            version: installed.version,
            packaging: installed.packaging,
            registration: registration_state(installed.browser),
            connection: connection_of(installed.browser, connections),
        })
        .collect()
}

/// The newest connection record of `browser`.
pub fn connection_of(browser: Browser, records: &[ConnectionRecord]) -> Option<ConnectionRecord> {
    records
        .iter()
        .filter(|record| protocol_name(browser) == record.browser)
        .max_by_key(|record| record.last_seen)
        .cloned()
}

/// The name the extension reports for `browser`.
pub fn protocol_name(browser: Browser) -> BrowserName {
    match browser {
        Browser::Chrome => BrowserName::Chrome,
        Browser::Chromium => BrowserName::Chromium,
        Browser::Edge => BrowserName::Edge,
        Browser::Brave => BrowserName::Brave,
        Browser::Vivaldi => BrowserName::Vivaldi,
        Browser::Opera => BrowserName::Opera,
        Browser::Firefox => BrowserName::Firefox,
    }
}

/// The report's short name (`chrome`, `edge`, `firefox`).
pub fn short_name(browser: Browser) -> &'static str {
    match browser {
        Browser::Chrome => "chrome",
        Browser::Chromium => "chromium",
        Browser::Edge => "edge",
        Browser::Brave => "brave",
        Browser::Vivaldi => "vivaldi",
        Browser::Opera => "opera",
        Browser::Firefox => "firefox",
    }
}

/// `"129.0.6668.59"` → `"129.0"`: enough to know the release, and what the
/// report prints.
pub fn short_version(version: &str) -> String {
    let parts: Vec<&str> = version.trim().split('.').take(2).collect();
    match parts.as_slice() {
        [major] => format!("{major}.0"),
        _ => parts.join("."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(browser: BrowserName, last_seen: i64) -> ConnectionRecord {
        ConnectionRecord {
            browser,
            browser_version: "129".to_owned(),
            extension_version: "1.4.2".to_owned(),
            last_seen,
        }
    }

    #[test]
    fn the_newest_record_of_the_same_browser_wins() {
        let records = [
            record(BrowserName::Chrome, 5),
            record(BrowserName::Edge, 9),
            record(BrowserName::Chrome, 7),
        ];
        let found = connection_of(Browser::Chrome, &records);
        assert_eq!(found.map(|r| r.last_seen), Some(7));
        assert_eq!(connection_of(Browser::Firefox, &records), None);
    }

    #[test]
    fn versions_keep_major_and_minor() {
        assert_eq!(short_version("129.0.6668.59"), "129.0");
        assert_eq!(short_version("131"), "131.0");
        assert_eq!(short_version("131.0"), "131.0");
    }
}
