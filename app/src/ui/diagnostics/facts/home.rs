//! Paths without the user's folder: `~/…` on macOS and Linux,
//! `%USERPROFILE%\…` on Windows (`docs/ux.md` §8.7). A home folder is
//! usually named after the person, and the report ends up in public issues.

use std::path::Path;

/// `path` with a leading `home` replaced by the OS's variable.
pub fn anonymize(path: &str, home: Option<&Path>) -> String {
    let Some(home) = home.and_then(Path::to_str).filter(|home| !home.is_empty()) else {
        return path.to_owned();
    };
    let home = home.trim_end_matches(['/', '\\']);
    let variable = if cfg!(windows) { "%USERPROFILE%" } else { "~" };
    replace_prefix(path, home, variable).unwrap_or_else(|| path.to_owned())
}

/// Every occurrence of `home` in a free text (an error message that quotes
/// a path) replaced the same way.
pub fn anonymize_text(text: &str, home: Option<&Path>) -> String {
    let Some(home) = home.and_then(Path::to_str).filter(|home| home.len() > 1) else {
        return text.to_owned();
    };
    let home = home.trim_end_matches(['/', '\\']);
    let variable = if cfg!(windows) { "%USERPROFILE%" } else { "~" };
    text.replace(home, variable)
}

/// The current user's home folder.
pub fn current() -> Option<std::path::PathBuf> {
    std::env::home_dir()
}

fn replace_prefix(path: &str, home: &str, variable: &str) -> Option<String> {
    // Windows paths compare without case; Unix ones exactly.
    let head = path.get(..home.len())?;
    let same = if cfg!(windows) {
        head.eq_ignore_ascii_case(home)
    } else {
        head == home
    };
    let rest = &path[home.len()..];
    let at_boundary = rest.is_empty() || rest.starts_with(['/', '\\']);
    (same && at_boundary).then(|| format!("{variable}{rest}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(windows))]
    #[test]
    fn the_home_folder_becomes_a_tilde() {
        let home = Path::new("/home/ana");
        assert_eq!(
            anonymize("/home/ana/Downloads/x.so", Some(home)),
            "~/Downloads/x.so"
        );
        assert_eq!(
            anonymize("/home/anabel/x.so", Some(home)),
            "/home/anabel/x.so"
        );
        assert_eq!(anonymize("/usr/lib/x.so", Some(home)), "/usr/lib/x.so");
        assert_eq!(
            anonymize_text("module file not found: /home/ana/x.so", Some(home)),
            "module file not found: ~/x.so"
        );
    }

    #[test]
    fn without_a_home_nothing_changes() {
        assert_eq!(anonymize("/a/b", None), "/a/b");
        assert_eq!(anonymize_text("/a/b", Some(Path::new("/"))), "/a/b");
    }
}
