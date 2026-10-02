//! Replacing byte ranges of a message, shared by every redaction pass.
//!
//! Passes only ever cut at ASCII positions, so the ranges always fall on
//! UTF-8 boundaries and non-ASCII text passes through untouched.

/// A range of the input and what it becomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub with: &'static str,
}

/// `text` with every span replaced. Spans must be sorted and disjoint, which
/// every pass guarantees by scanning left to right.
pub fn replace(text: &str, spans: &[Span]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    for span in spans {
        out.push_str(&text[at..span.start]);
        out.push_str(span.with);
        at = span.end;
    }
    out.push_str(&text[at..]);
    out
}

/// The end of the run of bytes from `start` that satisfy `keep`.
pub fn run_end(bytes: &[u8], start: usize, keep: impl Fn(u8) -> bool) -> usize {
    bytes[start..]
        .iter()
        .position(|&b| !keep(b))
        .map_or(bytes.len(), |offset| start + offset)
}

/// Whether the byte before `index` does not continue a word, so a pattern
/// found at `index` starts one.
pub fn starts_word(bytes: &[u8], index: usize) -> bool {
    index == 0 || !is_word(bytes[index - 1])
}

/// Letters, digits and `_`, plus every non-ASCII byte (part of a word in any
/// language).
pub fn is_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || !byte.is_ascii()
}
