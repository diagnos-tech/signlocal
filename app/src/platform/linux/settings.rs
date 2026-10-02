//! Reduce motion and dark mode on Linux desktops.
//!
//! There is no single API: the XDG settings portal answers for the color
//! scheme on current desktops (GNOME, KDE, and inside Flatpak); GNOME's
//! `gsettings` keys cover reduce motion and older GNOME versions; KDE keeps
//! its animation speed in `kdeglobals`.

use super::command::output;

const GNOME_INTERFACE: &str = "org.gnome.desktop.interface";

pub fn reduce_motion() -> bool {
    // KDE systems often have gsettings too, holding GNOME's untouched
    // default; ask the running desktop first.
    let kde = std::env::var("XDG_CURRENT_DESKTOP").is_ok_and(|desktop| desktop.contains("KDE"));
    let answer = if kde {
        kde_reduce_motion().or_else(gnome_reduce_motion)
    } else {
        gnome_reduce_motion()
    };
    answer.unwrap_or(false)
}

fn gnome_reduce_motion() -> Option<bool> {
    output("gsettings", &["get", GNOME_INTERFACE, "enable-animations"])
        .and_then(|text| parse_bool(&text))
        .map(|enabled| !enabled)
}

fn kde_reduce_motion() -> Option<bool> {
    ["kreadconfig6", "kreadconfig5"].iter().find_map(|tool| {
        output(
            tool,
            &["--group", "KDE", "--key", "AnimationDurationFactor"],
        )
        .map(|text| kde_animations_off(&text))
    })
}

pub fn dark_mode() -> Option<bool> {
    portal_color_scheme()
        .or_else(|| gnome_setting("color-scheme").and_then(|scheme| scheme_is_dark(&scheme)))
        .or_else(|| {
            gnome_setting("gtk-theme").and_then(|theme| theme_is_dark(&theme).then_some(true))
        })
}

fn portal_color_scheme() -> Option<bool> {
    let reply = output(
        "gdbus",
        &[
            "call",
            "--session",
            "--dest",
            "org.freedesktop.portal.Desktop",
            "--object-path",
            "/org/freedesktop/portal/desktop",
            "--method",
            "org.freedesktop.portal.Settings.ReadOne",
            "org.freedesktop.appearance",
            "color-scheme",
        ],
    )?;
    // 0: no preference, 1: dark, 2: light.
    match parse_portal_uint(&reply)? {
        1 => Some(true),
        2 => Some(false),
        _ => None,
    }
}

fn gnome_setting(key: &str) -> Option<String> {
    output("gsettings", &["get", GNOME_INTERFACE, key]).map(|text| unquote(&text))
}

fn parse_bool(text: &str) -> Option<bool> {
    match text.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// `gsettings` prints strings in GVariant form: `'prefer-dark'`.
fn unquote(text: &str) -> String {
    let text = text.trim();
    text.strip_prefix('\'')
        .and_then(|inner| inner.strip_suffix('\''))
        .unwrap_or(text)
        .to_owned()
}

fn scheme_is_dark(scheme: &str) -> Option<bool> {
    match scheme {
        "prefer-dark" => Some(true),
        "prefer-light" => Some(false),
        _ => None,
    }
}

/// Themes without a color-scheme setting mark their dark variant by name
/// (`Adwaita-dark`, `Yaru-dark`, `Breeze-Dark`).
fn theme_is_dark(theme: &str) -> bool {
    theme.to_ascii_lowercase().ends_with("-dark")
}

/// `gdbus` prints the variant: `(<uint32 1>,)`, or `(<<uint32 1>>,)` from
/// portals that only have the older `Read`.
fn parse_portal_uint(reply: &str) -> Option<u32> {
    let (_, rest) = reply.split_once("uint32 ")?;
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

/// KDE's "Animation speed" slider at "Instant" stores factor 0.
fn kde_animations_off(text: &str) -> bool {
    text.trim().parse::<f64>().is_ok_and(|factor| factor == 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_gsettings_values() {
        assert_eq!(parse_bool("false\n"), Some(false));
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("No such key"), None);
        assert_eq!(unquote("'prefer-dark'\n"), "prefer-dark");
        assert_eq!(unquote("default"), "default");
    }

    #[test]
    fn maps_color_schemes_and_theme_names() {
        assert_eq!(scheme_is_dark("prefer-dark"), Some(true));
        assert_eq!(scheme_is_dark("prefer-light"), Some(false));
        assert_eq!(scheme_is_dark("default"), None);
        assert!(theme_is_dark("Breeze-Dark"));
        assert!(!theme_is_dark("Adwaita"));
        assert!(!theme_is_dark("darkness"));
    }

    #[test]
    fn parses_portal_replies() {
        assert_eq!(parse_portal_uint("(<uint32 1>,)\n"), Some(1));
        assert_eq!(parse_portal_uint("(<<uint32 2>>,)\n"), Some(2));
        assert_eq!(parse_portal_uint("Error: GDBus.Error"), None);
    }

    #[test]
    fn only_instant_kde_animations_count_as_off() {
        assert!(kde_animations_off("0\n"));
        assert!(!kde_animations_off("1"));
        assert!(!kde_animations_off(""));
    }
}
