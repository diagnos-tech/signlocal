//! Long runs of characters that only data produces: document numbers,
//! digests, fingerprints, serial numbers, Base64 certificates and signatures.
//!
//! The thresholds keep what logs legitimately carry — sizes, counts, dates,
//! `0x00000030`-style status codes, versions — and catch the shortest
//! personal numbers: a CPF has 11 digits, a CNPJ 14, a SHA-256 digest 64 hex
//! digits, a certificate serial usually 16 or more.

use super::spans::{Span, is_word, run_end};

/// Digits (with `.`, `-`, `/` between them) counted as a document number.
const MIN_DIGITS: usize = 9;
/// Hex digits (optionally `:`-separated) counted as a digest or serial.
const MIN_HEX_DIGITS: usize = 16;
/// Base64 characters counted as encoded binary.
const MIN_BASE64: usize = 40;

/// Runs of Base64 text with mixed case and digits (encoded binary).
pub fn base64(text: &str) -> Vec<Span> {
    let bytes = text.as_bytes();
    let is_b64 = |b: u8| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'/' | b'=');
    let mut spans = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if !is_b64(bytes[i]) {
            i += 1;
            continue;
        }
        let end = run_end(bytes, i, is_b64);
        let run = &bytes[i..end];
        let mixed = run.iter().any(u8::is_ascii_uppercase)
            && run.iter().any(u8::is_ascii_lowercase)
            && run.iter().any(u8::is_ascii_digit);
        if run.len() >= MIN_BASE64 && mixed {
            spans.push(Span {
                start: i,
                end,
                with: "[base64]",
            });
        }
        i = end;
    }
    spans
}

/// Runs of hex digits, `:`-separated or not, long enough to be a digest,
/// fingerprint or serial number, and not glued to a longer word.
pub fn hex(text: &str) -> Vec<Span> {
    grouped_runs(text, u8::is_ascii_hexdigit, b":", MIN_HEX_DIGITS, "[hex]")
}

/// Runs of decimal digits, with `.`, `-` or `/` between groups, long enough
/// to be a CPF, CNPJ, RG, phone or card number.
pub fn digits(text: &str) -> Vec<Span> {
    grouped_runs(text, u8::is_ascii_digit, b".-/", MIN_DIGITS, "[number]")
}

/// Runs of `digit` bytes where single `separators` may join groups; a run
/// with at least `min` digits that stands alone as a word becomes `with`.
fn grouped_runs(
    text: &str,
    digit: fn(&u8) -> bool,
    separators: &[u8],
    min: usize,
    with: &'static str,
) -> Vec<Span> {
    let bytes = text.as_bytes();
    let mut spans = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if !digit(&bytes[i]) || (i > 0 && is_word(bytes[i - 1])) {
            i += 1;
            continue;
        }
        let (end, count) = scan_group(bytes, i, digit, separators);
        let glued = end < bytes.len() && is_word(bytes[end]);
        if count >= min && !glued {
            spans.push(Span {
                start: i,
                end,
                with,
            });
        }
        i = end.max(i + 1);
    }
    spans
}

/// From `start`, the end of digits joined by single separators, and how many
/// digits it holds. A trailing separator is not part of the run.
fn scan_group(
    bytes: &[u8],
    start: usize,
    digit: fn(&u8) -> bool,
    separators: &[u8],
) -> (usize, usize) {
    let mut end = start;
    let mut count = 0;
    let mut i = start;
    while i < bytes.len() {
        if digit(&bytes[i]) {
            count += 1;
            i += 1;
            end = i;
        } else if separators.contains(&bytes[i]) && bytes.get(i + 1).is_some_and(digit) {
            i += 1;
        } else {
            break;
        }
    }
    (end, count)
}
