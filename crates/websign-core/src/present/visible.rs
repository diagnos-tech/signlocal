//! Removing characters that change how the rest of a displayed name reads.

/// `text` without control characters and Unicode bidirectional formatting
/// characters; whitespace controls (tab, newline) become a space.
///
/// A certificate CN, a product name or a file name is written by whoever made
/// it. A right-to-left override (U+202E) makes `"invoice\u{202e}gpj.exe"`
/// read as `"invoiceexe.jpg"`, and a newline can push text out of its row, so
/// neither may reach the confirmation window.
pub(crate) fn visible(text: &str) -> String {
    text.chars()
        .filter_map(|c| match c {
            c if c.is_whitespace() && c.is_control() => Some(' '),
            c if c.is_control() || is_bidi_format(c) => None,
            c => Some(c),
        })
        .collect()
}

/// ALM, LRM, RLM, the embeddings and overrides (U+202A–U+202E) and the
/// isolates (U+2066–U+2069).
fn is_bidi_format(c: char) -> bool {
    matches!(c, '\u{61c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}
