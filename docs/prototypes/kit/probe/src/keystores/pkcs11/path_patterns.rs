//! Expansion of the path templates in the list of known modules: Windows
//! environment variables and one wildcard directory.

use std::path::PathBuf;

/// Every concrete path a template can stand for on this machine, best first.
/// A template with an unset variable stands for none.
pub fn expand(template: &str) -> Vec<PathBuf> {
    expand_variables(template).map_or_else(Vec::new, |pattern| expand_wildcard(&pattern))
}

/// Replaces `%NAME%` with the environment variable's value; `None` when a
/// variable is not set (the path cannot exist on this machine).
fn expand_variables(template: &str) -> Option<String> {
    let mut expanded = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find('%') {
        let (before, after) = rest.split_at(start);
        let end = after[1..].find('%')? + 1;
        expanded.push_str(before);
        expanded.push_str(&std::env::var(&after[1..end]).ok()?);
        rest = &after[end + 1..];
    }
    expanded.push_str(rest);
    Some(expanded)
}

/// Expands the one `*` a path may hold inside a directory name, newest
/// (alphabetically last) first, e.g. `/opt/ePass2003-Castle-*/x64/libx.so`.
/// A path without `*` is returned as it is.
fn expand_wildcard(pattern: &str) -> Vec<PathBuf> {
    let Some(star) = pattern.find('*') else {
        return vec![PathBuf::from(pattern)];
    };
    let Some(dir_start) = pattern[..star].rfind(['/', '\\']) else {
        return Vec::new();
    };
    let dir_end = pattern[star..]
        .find(['/', '\\'])
        .map_or(pattern.len(), |offset| star + offset);
    let (parent, prefix) = (&pattern[..dir_start], &pattern[dir_start + 1..star]);
    let (suffix, tail) = (&pattern[star + 1..dir_end], &pattern[dir_end..]);
    let Ok(entries) = std::fs::read_dir(if parent.is_empty() { "/" } else { parent }) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| {
            name.len() >= prefix.len() + suffix.len()
                && name.starts_with(prefix)
                && name.ends_with(suffix)
        })
        .collect();
    names.sort_by(|a, b| b.cmp(a));
    names
        .into_iter()
        .map(|name| {
            PathBuf::from(format!(
                "{parent}{}{name}{tail}",
                &pattern[dir_start..=dir_start]
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("websign-patterns-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn variables_are_expanded_and_unknown_ones_rule_the_path_out() {
        assert_eq!(
            expand_variables("plain/path"),
            Some("plain/path".to_owned())
        );
        assert_eq!(
            expand_variables(r"%WEBSIGN_TEST_UNSET_VARIABLE%\x.dll"),
            None
        );
        let path = std::env::var("PATH").unwrap();
        assert_eq!(expand_variables("a%PATH%b"), Some(format!("a{path}b")));
    }

    #[test]
    fn wildcard_directories_are_listed_newest_first() {
        let root = scratch_dir("wildcard");
        for name in ["vendor-20141128", "vendor-20190101", "other-1"] {
            std::fs::create_dir_all(root.join(name).join("x64")).unwrap();
        }
        let pattern = format!("{}/vendor-*/x64/lib.so", root.display());
        let expanded = expand_wildcard(&pattern);
        assert_eq!(
            expanded,
            [
                root.join("vendor-20190101/x64/lib.so"),
                root.join("vendor-20141128/x64/lib.so")
            ]
        );
        assert_eq!(
            expand_wildcard("/no/star/here.so"),
            [PathBuf::from("/no/star/here.so")]
        );
        assert!(expand_wildcard("/nonexistent-parent-dir/vendor-*/lib.so").is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn expand_combines_variables_and_wildcards() {
        assert!(expand(r"%WEBSIGN_TEST_UNSET_VARIABLE%\x.dll").is_empty());
        assert_eq!(expand("/plain/lib.so"), [PathBuf::from("/plain/lib.so")]);
    }
}
