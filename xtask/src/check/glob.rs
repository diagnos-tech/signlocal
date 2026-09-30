//! The two wildcards `SUMMARY.md` rows allow: `*` (any run of characters)
//! and `?` (one character), within a single file name.

/// Whether `name` matches `pattern`.
pub fn matches(pattern: &str, name: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let name: Vec<char> = name.chars().collect();
    matches_from(&pattern, &name)
}

fn matches_from(pattern: &[char], name: &[char]) -> bool {
    match pattern.split_first() {
        None => name.is_empty(),
        Some(('*', rest)) => (0..=name.len()).any(|skip| matches_from(rest, &name[skip..])),
        Some(('?', rest)) => !name.is_empty() && matches_from(rest, &name[1..]),
        Some((literal, rest)) => name.first() == Some(literal) && matches_from(rest, &name[1..]),
    }
}

/// Whether `name` uses a wildcard.
pub fn is_pattern(name: &str) -> bool {
    name.contains(['*', '?'])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn star_and_question_mark() {
        assert!(matches("*.der", "a.der"));
        assert!(matches("a*c", "abbbc"));
        assert!(matches("a?c", "abc"));
        assert!(!matches("a?c", "ac"));
        assert!(!matches("*.der", "a.pem"));
        assert!(matches("*", ""));
    }
}
