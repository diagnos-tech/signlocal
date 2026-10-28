//! Case- and accent-insensitive text for matching names and issuers.

/// Latin letters with diacritics and the ASCII letter they read as.
const ACCENTS: &[(&str, char)] = &[
    ("àáâãäåāăą", 'a'),
    ("çćĉċč", 'c'),
    ("ďđ", 'd'),
    ("èéêëēĕėęě", 'e'),
    ("ĝğġģ", 'g'),
    ("ìíîïĩīĭįı", 'i'),
    ("ĵ", 'j'),
    ("ķ", 'k'),
    ("ĺļľŀł", 'l'),
    ("ñńņňŉ", 'n'),
    ("òóôõöøōŏő", 'o'),
    ("ŕŗř", 'r'),
    ("śŝşš", 's'),
    ("ţťŧ", 't'),
    ("ùúûüũūŭůűų", 'u'),
    ("ŵ", 'w'),
    ("ýÿŷ", 'y'),
    ("źżž", 'z'),
];

/// Lowercases `text` and reduces accented letters to their base letter.
/// Combining marks (already decomposed input) are dropped.
pub(super) fn fold(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .filter(|c| !('\u{300}'..='\u{36f}').contains(c))
        .map(|c| {
            ACCENTS
                .iter()
                .find(|(group, _)| group.contains(c))
                .map_or(c, |&(_, base)| base)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::fold;

    #[test]
    fn folds_case_and_accents() {
        assert_eq!(fold("Cartão de CIDADÃO"), "cartao de cidadao");
        assert_eq!(fold("DIRECCIÓN"), "direccion");
        assert_eq!(fold("Jose\u{301}"), "jose");
    }
}
