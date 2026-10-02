//! `[popup]`, `[store]`, `[extension]` → `chrome.i18n` message catalogs.
//!
//! Format: <https://developer.chrome.com/docs/extensions/reference/api/i18n>.
//! `popup.ready_title` becomes `popup_ready_title`; `{version}` becomes
//! `$version$` with a `placeholders` entry.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

use super::text::summary;
use crate::i18n_files::{Locale, load_locales};
use crate::plan::Plan;

const SECTIONS: [&str; 3] = ["popup", "store", "extension"];
const DIR: &str = "extension/public/_locales";

pub fn plan(root: &Path) -> Result<Plan, String> {
    let locales = load_locales(root)?;
    let mut plan = Plan {
        owned_dirs: vec![PathBuf::from(DIR)],
        ..Plan::default()
    };
    let mut folders = Vec::new();
    for locale in &locales {
        let folder = locale.name.replace('-', "_");
        plan.add(format!("{DIR}/{folder}/messages.json"), catalog(locale)?);
        let path = format!("{DIR}/{folder}");
        plan.add(
            format!("{path}/SUMMARY.md"),
            summary(
                &path,
                &[(
                    "messages.json",
                    "the popup, store and extension texts for this language",
                )],
            ),
        );
        folders.push(format!("{folder}/"));
    }
    let entries: Vec<(&str, &str)> = folders
        .iter()
        .map(|f| (f.as_str(), "one language"))
        .collect();
    plan.add(format!("{DIR}/SUMMARY.md"), summary(DIR, &entries));
    Ok(plan)
}

fn catalog(locale: &Locale) -> Result<String, String> {
    let mut messages = BTreeMap::new();
    for section in SECTIONS {
        if let Some(table) = locale.table.get(section).and_then(toml::Value::as_table) {
            for (key, value) in table {
                let text = value.as_str().ok_or_else(|| {
                    format!("i18n/{}.toml: {section}.{key} must be a string (chrome.i18n has no plurals)", locale.name)
                })?;
                let name = format!("{section}_{key}");
                if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    return Err(format!(
                        "i18n/{}.toml: {section}.{key}: chrome.i18n message names allow only \
                         ASCII letters, digits and `_`",
                        locale.name
                    ));
                }
                messages.insert(name, entry(text));
            }
        }
    }
    let map: Map<String, Value> = messages.into_iter().collect();
    let mut text = serde_json::to_string_pretty(&Value::Object(map)).map_err(|e| e.to_string())?;
    text.push('\n');
    Ok(text)
}

/// One `messages.json` entry.
///
/// `$n` follows the alphabetical order of the placeholder names, not their
/// order in the text: translations reorder words, and the code passes its
/// substitutions in one fixed order for every language.
fn entry(text: &str) -> Value {
    let names = placeholder_names(text);
    let mut message = text.replace('$', "$$");
    let mut placeholders = Map::new();
    for (index, name) in names.iter().enumerate() {
        message = message.replace(&format!("{{{name}}}"), &format!("${name}$"));
        placeholders.insert(
            name.clone(),
            json!({ "content": format!("${}", index + 1) }),
        );
    }
    if placeholders.is_empty() {
        json!({ "message": message })
    } else {
        json!({ "message": message, "placeholders": placeholders })
    }
}

/// Distinct `{name}` placeholders (`[a-z_]+`), sorted.
fn placeholder_names(text: &str) -> Vec<String> {
    let mut names: Vec<String> = text
        .split('{')
        .skip(1)
        .filter_map(|rest| rest.split_once('}'))
        .map(|(name, _)| name)
        .filter(|name| !name.is_empty() && name.chars().all(|c| c.is_ascii_lowercase() || c == '_'))
        .map(str::to_owned)
        .collect();
    names.sort();
    names.dedup();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholders_are_numbered_alphabetically() {
        let value = entry("You have {installed}; {required} needed");
        assert_eq!(value["message"], "You have $installed$; $required$ needed");
        assert_eq!(value["placeholders"]["installed"]["content"], "$1");
        assert_eq!(value["placeholders"]["required"]["content"], "$2");
    }

    #[test]
    fn a_dollar_sign_is_escaped_and_plain_text_has_no_placeholders() {
        let value = entry("Costs $5");
        assert_eq!(value["message"], "Costs $$5");
        assert!(value.get("placeholders").is_none());
    }

    #[test]
    fn keys_join_section_and_name() {
        let locale = Locale {
            name: "pt-BR".into(),
            table: "[popup]\nready_title = \"Pronto\"".parse().unwrap(),
        };
        assert!(catalog(&locale).unwrap().contains("\"popup_ready_title\""));
    }

    #[test]
    fn keys_outside_chrome_names_are_refused() {
        let locale = Locale {
            name: "en".into(),
            table: "[popup]\n\"bad-key\" = \"x\"".parse().unwrap(),
        };
        assert!(catalog(&locale).unwrap_err().contains("popup.bad-key"));
    }

    #[test]
    fn plural_tables_are_refused() {
        let locale = Locale {
            name: "en".into(),
            table: "[popup.x]\none = \"a\"\nother = \"b\"".parse().unwrap(),
        };
        assert!(catalog(&locale).unwrap_err().contains("popup.x"));
    }
}
