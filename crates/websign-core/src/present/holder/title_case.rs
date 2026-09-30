//! Turning an all-uppercase certificate name into how a person writes it.

/// Lowercase connectors, unless they open the name.
const PARTICLES: [&str; 13] = [
    "da", "das", "de", "di", "do", "dos", "du", "e", "del", "la", "van", "von", "y",
];

/// Company suffixes that stay as written (`LTDA` is not one: it becomes
/// `Ltda`).
const KEPT_SUFFIXES: [&str; 5] = ["ME", "EPP", "EIRELI", "S.A.", "S/A"];

/// `"JOAO DA SILVA DOS SANTOS"` → `"Joao da Silva dos Santos"`.
///
/// Each word gets an uppercase first letter and lowercase rest, except the
/// particles `da das de di do dos du e del la van von y` (lowercase unless
/// first) and the company suffixes `ME EPP EIRELI S.A. S/A` (kept as written;
/// `LTDA` becomes `Ltda`). Word separators are kept exactly.
pub fn title_case(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut first_word = true;
    let mut rest = name;
    while !rest.is_empty() {
        let is_space = |c: char| c.is_whitespace();
        let space_len = rest.find(|c| !is_space(c)).unwrap_or(rest.len());
        let (spaces, tail) = rest.split_at(space_len);
        out.push_str(spaces);
        let word_len = tail.find(is_space).unwrap_or(tail.len());
        let (word, tail) = tail.split_at(word_len);
        if !word.is_empty() {
            push_word(&mut out, word, first_word);
            first_word = false;
        }
        rest = tail;
    }
    out
}

fn push_word(out: &mut String, word: &str, first_word: bool) {
    let upper = word.to_uppercase();
    if KEPT_SUFFIXES.contains(&upper.as_str()) {
        out.push_str(word);
        return;
    }
    let lower = word.to_lowercase();
    if !first_word && PARTICLES.contains(&lower.as_str()) {
        out.push_str(&lower);
        return;
    }
    let mut at_part_start = true;
    for c in word.chars() {
        if at_part_start {
            out.extend(c.to_uppercase());
        } else {
            out.extend(c.to_lowercase());
        }
        at_part_start = matches!(c, '-' | '\'' | '\u{2019}');
    }
}
