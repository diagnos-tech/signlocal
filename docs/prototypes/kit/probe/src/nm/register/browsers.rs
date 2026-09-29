//! The browsers the host can be registered with.

/// What `--browser` accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum BrowserChoice {
    All,
    Chrome,
    Chromium,
    Edge,
    Brave,
    Vivaldi,
    Opera,
    Firefox,
}

/// A browser with its own manifest locations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Browser {
    Chrome,
    Chromium,
    Edge,
    Brave,
    Vivaldi,
    Opera,
    Firefox,
}

/// Engines differ in the manifest they read: Chromium-based browsers check
/// `allowed_origins`, Firefox checks `allowed_extensions`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Chromium,
    Firefox,
}

impl Browser {
    pub const ALL: [Browser; 7] = [
        Self::Chrome,
        Self::Chromium,
        Self::Edge,
        Self::Brave,
        Self::Vivaldi,
        Self::Opera,
        Self::Firefox,
    ];

    pub const fn family(self) -> Family {
        match self {
            Self::Firefox => Family::Firefox,
            _ => Family::Chromium,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Chrome => "Google Chrome",
            Self::Chromium => "Chromium",
            Self::Edge => "Microsoft Edge",
            Self::Brave => "Brave",
            Self::Vivaldi => "Vivaldi",
            Self::Opera => "Opera",
            Self::Firefox => "Firefox",
        }
    }
}

/// Expands `--browser` values; an empty list or `all` means every browser.
pub fn selected(choices: &[BrowserChoice]) -> Vec<Browser> {
    if choices.is_empty() || choices.contains(&BrowserChoice::All) {
        return Browser::ALL.to_vec();
    }
    Browser::ALL
        .into_iter()
        .filter(|browser| {
            choices.iter().any(|choice| {
                matches!(
                    (choice, browser),
                    (BrowserChoice::Chrome, Browser::Chrome)
                        | (BrowserChoice::Chromium, Browser::Chromium)
                        | (BrowserChoice::Edge, Browser::Edge)
                        | (BrowserChoice::Brave, Browser::Brave)
                        | (BrowserChoice::Vivaldi, Browser::Vivaldi)
                        | (BrowserChoice::Opera, Browser::Opera)
                        | (BrowserChoice::Firefox, Browser::Firefox)
                )
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_choice_and_all_mean_every_browser() {
        assert_eq!(selected(&[]), Browser::ALL);
        assert_eq!(
            selected(&[BrowserChoice::Edge, BrowserChoice::All]),
            Browser::ALL
        );
    }

    #[test]
    fn choices_are_deduplicated_and_ordered() {
        let picked = selected(&[
            BrowserChoice::Firefox,
            BrowserChoice::Chrome,
            BrowserChoice::Firefox,
        ]);
        assert_eq!(picked, [Browser::Chrome, Browser::Firefox]);
    }

    #[test]
    fn only_firefox_uses_the_firefox_family() {
        for browser in Browser::ALL {
            assert_eq!(
                browser.family() == Family::Firefox,
                browser == Browser::Firefox
            );
        }
    }
}
