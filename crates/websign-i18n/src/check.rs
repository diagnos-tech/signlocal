//! Consistency rules between a locale file and the reference, used by
//! `cargo xtask check i18n` and by this crate's tests.

/// One problem in a locale file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// A reference key the locale lacks.
    Missing { key: String },
    /// A key the reference does not have.
    Extra { key: String },
    /// Placeholders differ from the reference's (as sets).
    Placeholders {
        key: String,
        expected: Vec<String>,
        found: Vec<String>,
    },
    /// A plural lacks `other`, or has categories the locale never uses.
    Plural { key: String, detail: String },
    /// A `[popup]`/`[extension]` message has a plural (`chrome.i18n` has none)
    /// or the store description exceeds 132 characters.
    Extension { key: String, detail: String },
    /// Not valid TOML, or a value that is neither a string nor a table.
    Syntax { detail: String },
}

/// Compares `candidate` (a locale file's text) with `reference` (`en.toml`).
pub fn check_locale(reference: &str, candidate: &str) -> Vec<Problem> {
    let _ = (reference, candidate);
    todo!("SPEC.md §7")
}
