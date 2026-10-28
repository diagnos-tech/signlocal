//! Turning an all-uppercase certificate name into how a person writes it.

/// Lowercase connectors, unless they open the name.
const PARTICLES: [&str; 13] = [
    "da", "das", "de", "di", "do", "dos", "du", "e", "del", "la", "van", "von", "y",
];

/// Company suffixes, in their official spelling (`LTDA` is not one: it
/// becomes `Ltda`).
const COMPANY_SUFFIXES: [&str; 5] = ["ME", "EPP", "EIRELI", "S.A.", "S/A"];

/// `"JOAO DA SILVA DOS SANTOS"` → `"Joao da Silva dos Santos"`.
///
/// A word is a maximal run of non-whitespace; whitespace is kept exactly.
/// Particles (`da das de di do dos du e del la van von y`) become lowercase
/// unless they are the first word; company suffixes (`ME EPP EIRELI S.A.
/// S/A`) take their official spelling. Both match case-insensitively. Every
/// other word is lowercased, then the first letter after the word start or
/// after any punctuation is capitalized, so `MARIA-CLARA` → `Maria-Clara`,
/// `D'ÁVILA` → `D'Ávila`, `A.B.` → `A.B.` and `R2:D2` → `R2:D2`. Digits
/// neither take a case nor start a part (`3M` → `3M`).
pub fn title_case(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut first_word = true;
    let mut rest = name;
    while !rest.is_empty() {
        let space_len = rest
            .find(|c: char| !c.is_whitespace())
            .unwrap_or(rest.len());
        let (spaces, tail) = rest.split_at(space_len);
        out.push_str(spaces);
        let word_len = tail.find(char::is_whitespace).unwrap_or(tail.len());
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
    if let Some(suffix) = COMPANY_SUFFIXES.iter().find(|s| **s == upper) {
        out.push_str(suffix);
        return;
    }
    let lower = word.to_lowercase();
    if !first_word && PARTICLES.contains(&lower.as_str()) {
        out.push_str(&lower);
        return;
    }
    let mut capitalize_next = true;
    for c in word.chars() {
        if c.is_alphabetic() {
            if capitalize_next {
                out.extend(c.to_uppercase());
            } else {
                out.extend(c.to_lowercase());
            }
            capitalize_next = false;
        } else {
            out.push(c);
            if !c.is_numeric() && !is_combining_mark(c) {
                capitalize_next = true;
            }
        }
    }
}

/// Combining diacritics (U+0300–U+036F) belong to the letter before them:
/// a decomposed `Á` must not start a new part.
fn is_combining_mark(c: char) -> bool {
    ('\u{300}'..='\u{36f}').contains(&c)
}
