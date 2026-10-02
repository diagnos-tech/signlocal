//! The person's home folder and login name, which paths reveal (a login is
//! often the person's name: `C:\Users\ana.souza`, `/home/asouza`), and the
//! same for other accounts' profile folders and network shares.

use super::spans::{Span, is_word, run_end, starts_word};

/// Folders whose next component is an account name, compared lowercase.
const PROFILE_ROOTS: [&str; 4] = ["/home/", "/users/", "\\users\\", "\\home\\"];
/// Profile folders that belong to nobody in particular.
const SHARED_PROFILES: [&str; 4] = ["public", "shared", "default", "all users"];
/// Longest account name followed when it contains spaces (`Ana Souza`).
const MAX_NAME: usize = 64;

/// Shorter logins (`me`, `a`) would redact ordinary words.
const MIN_LOGIN_LEN: usize = 3;

/// `home` (in either slash style) becomes `~`; afterwards any whole-word
/// occurrence of `login` becomes `[user]`.
pub fn redact(text: &str, home: Option<&str>, login: Option<&str>) -> String {
    let mut out = text.to_owned();
    if let Some(home) = home.filter(|h| h.len() > 1) {
        let home = home.trim_end_matches(['/', '\\']);
        for form in [
            home.to_owned(),
            home.replace('\\', "/"),
            home.replace('/', "\\"),
        ] {
            out = replace_ignore_case(&out, &form, "~");
        }
    }
    match login.filter(|l| l.len() >= MIN_LOGIN_LEN) {
        Some(login) => super::spans::replace(&out, &whole_words(&out, login)),
        None => out,
    }
}

/// `needle` → `with`, ASCII case-insensitively (Windows paths ignore case).
fn replace_ignore_case(text: &str, needle: &str, with: &str) -> String {
    if needle.is_empty() {
        return text.to_owned();
    }
    let lower = text.to_ascii_lowercase();
    let needle = needle.to_ascii_lowercase();
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    for (start, _) in lower.match_indices(&needle) {
        if start < at {
            continue;
        }
        out.push_str(&text[at..start]);
        out.push_str(with);
        at = start + needle.len();
    }
    out.push_str(&text[at..]);
    out
}

/// Whole-word, case-insensitive occurrences of `word`.
fn whole_words(text: &str, word: &str) -> Vec<Span> {
    let bytes = text.as_bytes();
    let lower = text.to_ascii_lowercase();
    lower
        .match_indices(&word.to_ascii_lowercase())
        .filter(|(start, found)| {
            let end = start + found.len();
            starts_word(bytes, *start) && (end == bytes.len() || !is_word(bytes[end]))
        })
        .map(|(start, found)| Span {
            start,
            end: start + found.len(),
            with: "[user]",
        })
        .collect()
}

/// The account name after `/home/`, `/Users/` or `X:\Users\` (any case,
/// either slash) becomes `[user]`, and the server of a `\\server\share` path
/// becomes `[host]` (machines are often named after their owner).
pub fn profiles(text: &str) -> Vec<Span> {
    let lower = text.to_ascii_lowercase();
    let mut spans: Vec<Span> = PROFILE_ROOTS
        .iter()
        .flat_map(|root| lower.match_indices(root).map(|(at, root)| at + root.len()))
        .filter_map(|start| component(text, start, "[user]"))
        .filter(|span| {
            let name = text[span.start..span.end].to_ascii_lowercase();
            !SHARED_PROFILES.contains(&name.as_str())
        })
        .collect();
    let bytes = text.as_bytes();
    // `\\server`, not the `\\` of an escaped path (`C:\\Users`, `a\\b`).
    let unc_start =
        |at: usize| at == 0 || !(is_word(bytes[at - 1]) || b":\\/".contains(&bytes[at - 1]));
    spans.extend(
        lower
            .match_indices("\\\\")
            .filter(|(at, _)| unc_start(*at))
            .filter_map(|(at, _)| component(text, at + 2, "[host]")),
    );
    spans.sort_by_key(|span| span.start);
    spans.dedup_by(|later, earlier| later.start < earlier.end);
    spans
}

/// The path component starting at `start`: up to the next separator, and
/// across spaces only when a separator follows soon (`Ana Souza\AppData`);
/// otherwise up to the first space (`C:\Users\ana failed`). Stops at `:`
/// so `/home/ana:/bin/sh`-like lists keep their next field.
fn component(text: &str, start: usize, with: &'static str) -> Option<Span> {
    let bytes = text.as_bytes();
    // Escaped paths double their separators (`C:\\Users\\ana`).
    let start = run_end(bytes, start, |b| b == b'\\' || b == b'/');
    let stop = |b: u8| matches!(b, b'/' | b'\\' | b'"' | b'\'' | b'\n' | b':' | b',' | b';');
    let to_separator = run_end(bytes, start, |b| !stop(b));
    let separated = bytes
        .get(to_separator)
        .is_some_and(|&b| b == b'/' || b == b'\\');
    let end = if separated && to_separator - start <= MAX_NAME {
        to_separator
    } else {
        run_end(bytes, start, |b| !stop(b) && !b.is_ascii_whitespace())
    };
    (end > start).then_some(Span { start, end, with })
}
