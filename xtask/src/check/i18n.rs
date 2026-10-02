//! Every locale file against `en.toml`, using the checker the app tests use.

use std::path::Path;

use websign_i18n::check::{Problem, check_locale};

use crate::i18n_files::{SHIPPED, locale_texts};

pub fn check(root: &Path) -> Result<Vec<String>, String> {
    let texts = locale_texts(root)?;
    let mut problems = Vec::new();
    for locale in SHIPPED {
        if !texts.iter().any(|(name, _)| name == locale) {
            problems.push(format!("i18n/{locale}.toml: missing locale file"));
        }
    }
    for (name, _) in texts
        .iter()
        .filter(|(name, _)| !SHIPPED.contains(&name.as_str()))
    {
        problems.push(format!(
            "i18n/{name}.toml: not a shipped locale ({})",
            SHIPPED.join(", ")
        ));
    }
    let Some((_, reference)) = texts.iter().find(|(name, _)| name == "en") else {
        return Ok(problems);
    };
    // `en` is checked against itself too: the extension and plural rules
    // apply to the reference as well.
    for (name, text) in &texts {
        for problem in check_locale(reference, text) {
            problems.push(format!("i18n/{name}.toml: {}", describe(&problem)));
        }
    }
    Ok(problems)
}

fn describe(problem: &Problem) -> String {
    match problem {
        Problem::Missing { key } => format!("missing key `{key}`"),
        Problem::Extra { key } => format!("key `{key}` is not in en.toml"),
        Problem::Placeholders {
            key,
            expected,
            found,
        } => {
            format!("`{key}` has placeholders {found:?}, en.toml has {expected:?}")
        }
        Problem::Plural { key, detail } => format!("plural `{key}`: {detail}"),
        Problem::Extension { key, detail } => format!("extension text `{key}`: {detail}"),
        Problem::Syntax { detail } => format!("invalid file: {detail}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempDir;

    const EN: &str = "[a]\nx = \"Hi {name}\"\n";

    fn repo(files: &[(&str, &str)]) -> TempDir {
        let dir = TempDir::new();
        for (name, text) in files {
            dir.write(&format!("i18n/{name}.toml"), text);
        }
        dir
    }

    fn all_but(name: &str, text: &str) -> Vec<(&'static str, String)> {
        SHIPPED
            .iter()
            .map(|l| {
                (
                    *l,
                    if *l == name {
                        text.to_owned()
                    } else {
                        EN.to_owned()
                    },
                )
            })
            .collect()
    }

    fn run(files: &[(&str, String)]) -> Vec<String> {
        let refs: Vec<(&str, &str)> = files.iter().map(|(n, t)| (*n, t.as_str())).collect();
        check(repo(&refs).path()).unwrap()
    }

    #[test]
    fn identical_locales_pass() {
        assert!(run(&all_but("", "")).is_empty());
    }

    #[test]
    fn a_missing_key_names_file_and_key() {
        let found = run(&all_but("fr", "[a]\n"));
        assert_eq!(found, ["i18n/fr.toml: missing key `a.x`"]);
    }

    #[test]
    fn a_missing_locale_file_is_reported() {
        let mut files = all_but("", "");
        files.retain(|(name, _)| *name != "de");
        assert_eq!(run(&files), ["i18n/de.toml: missing locale file"]);
    }
}
