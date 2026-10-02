//! Generates `keys.rs` from `i18n/en.toml`, the reference locale: one
//! constant per message, so a misspelled key is a compile error, and the
//! embedded sources of every locale file.
//!
//! A table whose keys are all CLDR plural categories (`zero`, `one`, `two`,
//! `few`, `many`, `other`) is one plural message ([`PluralKey`]); any other
//! string is a [`Key`].

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

const LOCALES: [&str; 7] = ["en", "pt-BR", "pt-PT", "es", "fr", "it", "de"];
const PLURAL_CATEGORIES: [&str; 6] = ["zero", "one", "two", "few", "many", "other"];

// A build script reports failure by panicking; there is no caller to return to.
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]
fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../i18n");
    let dir = dir
        .canonicalize()
        .expect("i18n/ folder at the repository root");
    let reference = read(&dir.join("en.toml"));

    let mut keys = Vec::new();
    collect(&reference, "", &mut keys);

    let mut out = String::from(
        "/// Every message key, generated from `i18n/en.toml`.\npub mod k {\n    use crate::{Key, PluralKey};\n",
    );
    for (path, plural) in &keys {
        let name = path.replace(['.', '-'], "_").to_uppercase();
        let kind = if *plural { "PluralKey" } else { "Key" };
        writeln!(out, "    pub const {name}: {kind} = {kind}(\"{path}\");").unwrap();
    }
    out.push_str("}\n\n/// Every key path, in file order.\npub const ALL_KEYS: &[&str] = &[\n");
    for (path, _) in &keys {
        writeln!(out, "    \"{path}\",").unwrap();
    }
    out.push_str("];\n\n/// `(locale tag, TOML source)` of every shipped locale.\npub const SOURCES: [(&str, &str); 7] = [\n");
    for locale in LOCALES {
        let file = dir.join(format!("{locale}.toml"));
        println!("cargo:rerun-if-changed={}", file.display());
        writeln!(
            out,
            "    (\"{locale}\", include_str!({:?})),",
            file.display().to_string()
        )
        .unwrap();
    }
    out.push_str("];\n");

    let target = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("keys.rs");
    std::fs::write(target, out).unwrap();
}

fn read(path: &Path) -> toml::Table {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    text.parse()
        .unwrap_or_else(|e| panic!("{} is not valid TOML: {e}", path.display()))
}

fn collect(table: &toml::Table, prefix: &str, keys: &mut Vec<(String, bool)>) {
    for (name, value) in table {
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };
        match value {
            toml::Value::String(_) => keys.push((path, false)),
            toml::Value::Table(inner) if is_plural(inner) => keys.push((path, true)),
            toml::Value::Table(inner) => collect(inner, &path, keys),
            _ => panic!("i18n/en.toml: {path} must be a string or a table"),
        }
    }
}

fn is_plural(table: &toml::Table) -> bool {
    table.contains_key("other")
        && table
            .iter()
            .all(|(name, value)| PLURAL_CATEGORIES.contains(&name.as_str()) && value.is_str())
}
