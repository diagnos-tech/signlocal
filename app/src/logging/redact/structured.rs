//! Personal data recognizable by its shape or its label: PEM blocks, web
//! addresses (sites), e-mail addresses, distinguished-name attributes and
//! `key=value` pairs whose key names something private.

use super::spans::{Span, is_word, run_end, starts_word};

/// `CN=…`, `O=…`: the attributes of a certificate subject or issuer that
/// name a person, an organization or an identifier.
const DN_ATTRIBUTES: [&str; 14] = [
    "cn",
    "sn",
    "gn",
    "g",
    "o",
    "ou",
    "l",
    "st",
    "e",
    "street",
    "title",
    "uid",
    "emailaddress",
    "serialnumber",
];

/// Keys whose value is one secret or identifying token (`pin=1234`).
const SECRET_KEYS: [&str; 11] = [
    "pin",
    "puk",
    "password",
    "passphrase",
    "secret",
    "serial",
    "fingerprint",
    "digest",
    "signature",
    "origin",
    "site",
];

/// Keys whose value is a name, possibly of several words (`holder: Ana
/// Souza`).
const NAME_KEYS: [&str; 5] = ["name", "subject", "issuer", "holder", "label"];

/// `-----BEGIN …-----` through the matching `-----END …-----`.
pub fn pem(text: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut from = 0;
    while let Some(offset) = text[from..].find("-----BEGIN") {
        let start = from + offset;
        let end = text[start..]
            .find("-----END")
            .and_then(|end| {
                let after = start + end + "-----END".len();
                text[after..].find("-----").map(|close| after + close + 5)
            })
            .unwrap_or(text.len());
        spans.push(Span {
            start,
            end,
            with: "[certificate]",
        });
        from = end;
    }
    spans
}

/// `http://…` and `https://…` up to the next space or delimiter.
pub fn urls(text: &str) -> Vec<Span> {
    let bytes = text.as_bytes();
    let lower = text.to_ascii_lowercase();
    let mut spans = Vec::new();
    let mut from = 0;
    while let Some(offset) = lower[from..].find("http") {
        let start = from + offset;
        let rest = &lower[start..];
        let scheme = ["https://", "http://"]
            .iter()
            .find(|s| rest.starts_with(**s));
        let Some(scheme) = scheme.filter(|_| starts_word(bytes, start)) else {
            from = start + 4;
            continue;
        };
        let end = run_end(bytes, start + scheme.len(), |b| {
            !b.is_ascii_whitespace() && !matches!(b, b'"' | b'\'' | b'<' | b'>' | b')' | b',')
        });
        spans.push(Span {
            start,
            end,
            with: "[site]",
        });
        from = end;
    }
    spans
}

/// `local@domain.tld`.
pub fn emails(text: &str) -> Vec<Span> {
    let bytes = text.as_bytes();
    let local = |b: u8| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'%' | b'+' | b'-');
    let domain = |b: u8| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-');
    let mut spans: Vec<Span> = Vec::new();
    for (at, _) in text.match_indices('@') {
        let start = bytes[..at]
            .iter()
            .rposition(|&b| !local(b))
            .map_or(0, |i| i + 1);
        let end = run_end(bytes, at + 1, domain);
        let has_dot = bytes[at + 1..end].contains(&b'.');
        let overlaps = spans.last().is_some_and(|last| last.end > start);
        if start < at && has_dot && !overlaps {
            spans.push(Span {
                start,
                end,
                with: "[email]",
            });
        }
    }
    spans
}

/// The values of distinguished-name attributes (`CN=Ana Souza,O=…`), up to
/// the next attribute separator.
pub fn dn_attributes(text: &str) -> Vec<Span> {
    labelled_values(text, &DN_ATTRIBUTES, b"=", |b| {
        !matches!(b, b',' | b'/' | b'+' | b';' | b'"' | b'\n')
    })
}

/// The values after secret keys (`pin=1234`, `password: "a b"`): one
/// token, or the quoted text.
pub fn secret_values(text: &str) -> Vec<Span> {
    labelled_values(text, &SECRET_KEYS, b"=:", |b| !b.is_ascii_whitespace())
}

/// The values after name keys (`holder: Ana Souza, …`): up to the end of
/// the clause.
pub fn name_values(text: &str) -> Vec<Span> {
    labelled_values(text, &NAME_KEYS, b"=:", |b| {
        !matches!(b, b',' | b';' | b')' | b'\n')
    })
}

/// For every whole-word `key` of `keys` (any case) followed by optional
/// spaces, one of `separators` and optional spaces: the value, quoted or up
/// to the first byte `in_value` refuses, becomes `[redacted]`.
fn labelled_values(
    text: &str,
    keys: &[&str],
    separators: &[u8],
    in_value: impl Fn(u8) -> bool,
) -> Vec<Span> {
    let bytes = text.as_bytes();
    let mut spans = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let key_end = run_end(bytes, i, is_word);
        if key_end == i {
            i += 1;
            continue;
        }
        let word = &text[i..key_end];
        let labelled = starts_word(bytes, i) && keys.iter().any(|k| word.eq_ignore_ascii_case(k));
        match labelled
            .then(|| value_after(bytes, key_end, separators, &in_value))
            .flatten()
        {
            Some((start, end)) => {
                spans.push(Span {
                    start,
                    end,
                    with: "[redacted]",
                });
                i = end;
            }
            None => i = key_end,
        }
    }
    spans
}

/// The value's range after a key ending at `at`, when a separator follows.
fn value_after(
    bytes: &[u8],
    at: usize,
    separators: &[u8],
    in_value: &impl Fn(u8) -> bool,
) -> Option<(usize, usize)> {
    let sep = run_end(bytes, at, |b| b == b' ');
    if !bytes.get(sep).is_some_and(|b| separators.contains(b)) {
        return None;
    }
    let start = run_end(bytes, sep + 1, |b| b == b' ');
    let quote = *bytes.get(start)?;
    let end = if quote == b'"' || quote == b'\'' {
        let close = run_end(bytes, start + 1, |b| b != quote);
        (close + 1).min(bytes.len())
    } else {
        let end = run_end(bytes, start, in_value);
        // A trailing space before the delimiter stays outside the value.
        start + bytes[start..end].trim_ascii_end().len()
    };
    (end > start).then_some((start, end))
}
