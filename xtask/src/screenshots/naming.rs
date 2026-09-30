//! `<window>-<state>-<theme>.png` names.

/// A screenshot file and what its name says.
#[derive(Debug, PartialEq, Eq)]
pub struct Shot {
    pub file: String,
    /// `confirm` in `confirm-waiting-light.png`; `None` when the name does
    /// not follow the pattern.
    pub window: Option<String>,
    pub state: String,
    /// `light` or `dark`.
    pub theme: Option<String>,
}

impl Shot {
    pub fn parse(file: &str) -> Shot {
        let stem = file.trim_end_matches(".png");
        let (rest, theme) = match stem.rsplit_once('-') {
            Some((rest, theme @ ("light" | "dark"))) => (rest, Some(theme.to_owned())),
            _ => (stem, None),
        };
        match (theme, rest.split_once('-')) {
            (Some(theme), Some((window, state))) => Shot {
                file: file.to_owned(),
                window: Some(window.to_owned()),
                state: state.to_owned(),
                theme: Some(theme),
            },
            (theme, _) => Shot {
                file: file.to_owned(),
                window: None,
                state: rest.to_owned(),
                theme,
            },
        }
    }

    /// One sentence for `SUMMARY.md`.
    pub fn description(&self) -> String {
        match (&self.window, &self.theme) {
            (Some(window), Some(theme)) => {
                format!(
                    "{window} window, {}, {theme} theme",
                    self.state.replace('-', " ")
                )
            }
            _ => format!("screenshot `{}`", self.state.replace('-', " ")),
        }
    }
}

/// Lowercase ASCII names only: they become paths and Markdown links.
pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "._-".contains(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_window_state_and_theme() {
        let shot = Shot::parse("confirm-need-digest-dark.png");
        assert_eq!(shot.window.as_deref(), Some("confirm"));
        assert_eq!(shot.state, "need-digest");
        assert_eq!(shot.theme.as_deref(), Some("dark"));
        assert_eq!(
            shot.description(),
            "confirm window, need digest, dark theme"
        );
    }

    #[test]
    fn other_names_are_kept_whole() {
        let shot = Shot::parse("startup.png");
        assert_eq!(
            (shot.window, shot.state.as_str(), shot.theme),
            (None, "startup", None)
        );
    }

    #[test]
    fn names_with_capitals_slashes_or_dots_first_are_refused() {
        assert!(valid_name("a-b_1.png"));
        assert!(!valid_name("A.png"));
        assert!(!valid_name("../a.png"));
        assert!(!valid_name(".hidden.png"));
    }
}
