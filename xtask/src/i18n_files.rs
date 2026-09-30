//! Reading `i18n/<locale>.toml`, shared by generation and checks.

use std::path::Path;

use crate::fsutil::{list_dir, read_text};

/// The locale files the product ships, in the order of `i18n.md`.
pub const SHIPPED: [&str; 7] = ["en", "pt-BR", "pt-PT", "es", "fr", "it", "de"];

/// One parsed locale file.
pub struct Locale {
    pub name: String,
    pub table: toml::Table,
}

/// `(locale, file text)` for every `i18n/*.toml`, shipped order first.
///
/// Line endings are normalized to `\n` so a Windows checkout generates the
/// same bytes (multi-line TOML strings keep the file's line endings).
pub fn locale_texts(root: &Path) -> Result<Vec<(String, String)>, String> {
    let folder = root.join("i18n");
    let mut found = Vec::new();
    for (file, is_dir) in list_dir(&folder)? {
        if let (false, Some(name)) = (is_dir, file.strip_suffix(".toml")) {
            let text = read_text(&folder.join(&file))?.replace("\r\n", "\n");
            found.push((name.to_owned(), text));
        }
    }
    found.sort_by_key(|(name, _)| {
        SHIPPED
            .iter()
            .position(|shipped| shipped == name)
            .unwrap_or(SHIPPED.len())
    });
    Ok(found)
}

/// Every locale file, parsed.
pub fn load_locales(root: &Path) -> Result<Vec<Locale>, String> {
    locale_texts(root)?
        .into_iter()
        .map(|(name, text)| {
            let table = text
                .parse()
                .map_err(|e| format!("i18n/{name}.toml is not valid TOML: {e}"))?;
            Ok(Locale { name, table })
        })
        .collect()
}
