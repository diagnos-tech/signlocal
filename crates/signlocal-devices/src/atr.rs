//! ATR pattern matching for `devices.json`.

/// Whether `atr` (hex, any case) matches `pattern` (uppercase hex where a
/// `..` pair stands for any one byte). Both must have the same length: an
/// ATR that only starts like a known card is a different card.
pub(crate) fn matches(pattern: &str, atr: &str) -> bool {
    let (pattern, atr) = (pattern.as_bytes(), atr.as_bytes());
    pattern.len() == atr.len()
        && pattern.len().is_multiple_of(2)
        && atr.iter().all(u8::is_ascii_hexdigit)
        && pattern
            .as_chunks::<2>()
            .0
            .iter()
            .zip(atr.as_chunks::<2>().0.iter())
            .all(|(wanted, found)| wanted == b".." || wanted.eq_ignore_ascii_case(found))
}

#[cfg(test)]
mod tests {
    use super::matches;

    #[test]
    fn exact_patterns_match_case_insensitively() {
        assert!(matches("3B8F80", "3B8F80"));
        assert!(matches("3B8F80", "3b8f80"));
        assert!(!matches("3B8F80", "3B8F81"));
    }

    #[test]
    fn a_wildcard_pair_matches_any_one_byte() {
        assert!(matches("3B....80", "3B12FF80"));
        assert!(!matches("3B....80", "3B12FF81"));
    }

    #[test]
    fn lengths_must_be_equal() {
        assert!(!matches("3B8F", "3B8F80"));
        assert!(!matches("3B8F80", "3B8F"));
        assert!(!matches("3B..", "3B"));
        assert!(matches("", ""));
    }

    #[test]
    fn odd_or_non_ascii_input_never_panics() {
        assert!(!matches("3B8", "3B8"));
        assert!(!matches("3Bé", "3Bé"));
        assert!(!matches("3B..", "3Bé"));
    }
}
