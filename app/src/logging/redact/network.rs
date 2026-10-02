//! IP addresses: a network address can locate or identify the person
//! (a home connection, a machine on a hospital network).
//!
//! Only text with the full shape counts, so what logs carry stays: versions
//! (`1.4.0`), times (`12:30:45`), Rust paths (`websign_host::engine`).

use super::spans::{Span, is_word, run_end};

const WITH: &str = "[ip]";

/// IPv4 (`10.0.0.1`) and IPv6 (`fe80::1`, `2001:db8:0:0:0:0:0:7`)
/// addresses standing alone as words; a port or zone after them stays.
pub fn addresses(text: &str) -> Vec<Span> {
    let bytes = text.as_bytes();
    let in_address = |b: u8| b.is_ascii_hexdigit() || b == b':' || b == b'.';
    let mut spans = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let starts = in_address(bytes[i]) && (i == 0 || !is_word(bytes[i - 1]));
        if !starts {
            i += 1;
            continue;
        }
        let end = run_end(bytes, i, in_address);
        let glued = end < bytes.len() && is_word(bytes[end]);
        // A trailing `.` or `:` ends a sentence, not the address.
        let candidate = text[i..end].trim_end_matches(['.', ':']);
        if let Some(len) = address_len(candidate).filter(|_| !glued) {
            spans.push(Span {
                start: i,
                end: i + len,
                with: WITH,
            });
        }
        i = end.max(i + 1);
    }
    spans
}

/// The length of the address `candidate` starts with: all of it, or the
/// IPv4 address before a `:port`.
fn address_len(candidate: &str) -> Option<usize> {
    if is_ipv4(candidate) || is_ipv6(candidate) {
        return Some(candidate.len());
    }
    let (host, port) = candidate.rsplit_once(':')?;
    let is_port = !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit());
    (is_port && is_ipv4(host)).then_some(host.len())
}

/// Four decimal groups of one to three digits, each at most 255.
fn is_ipv4(text: &str) -> bool {
    let groups: Vec<&str> = text.split('.').collect();
    groups.len() == 4
        && groups.iter().all(|group| {
            (1..=3).contains(&group.len())
                && group.bytes().all(|b| b.is_ascii_digit())
                && group.parse::<u16>().is_ok_and(|value| value <= 255)
        })
}

/// Hex groups of at most four digits joined by `:`, with `::` or all eight
/// groups, at least one decimal digit (so `a::b`-like words stay), and an
/// optional embedded IPv4 tail (`::ffff:10.0.0.1`).
fn is_ipv6(text: &str) -> bool {
    let (head, tail_is_ipv4) = match text.rfind(':') {
        Some(at) if text[at + 1..].contains('.') => (&text[..at + 1], is_ipv4(&text[at + 1..])),
        _ => (text, true),
    };
    if !tail_is_ipv4 || head.contains('.') || head.contains(":::") {
        return false;
    }
    let colons = head.matches(':').count();
    let compressed = head.contains("::");
    let groups_ok = head
        .split(':')
        .all(|group| group.len() <= 4 && group.bytes().all(|b| b.is_ascii_hexdigit()));
    groups_ok
        && (compressed && colons >= 2 || colons >= 7)
        && text.bytes().any(|b| b.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(text: &str) -> Vec<&str> {
        addresses(text)
            .iter()
            .map(|span| &text[span.start..span.end])
            .collect()
    }

    #[test]
    fn finds_addresses_of_both_families() {
        assert_eq!(found("from 10.0.0.1."), ["10.0.0.1"]);
        assert_eq!(found("at 192.168.100.200:8080"), ["192.168.100.200"]);
        assert_eq!(found("peer ::1 up"), ["::1"]);
        assert_eq!(found("[2001:db8::7]:443"), ["2001:db8::7"]);
        assert_eq!(
            found("fe80::1ff:fe23:4567:890a%en0"),
            ["fe80::1ff:fe23:4567:890a"]
        );
        assert_eq!(
            found("2001:0db8:85a3:0000:0000:8a2e:0370:7334"),
            ["2001:0db8:85a3:0000:0000:8a2e:0370:7334"]
        );
        assert_eq!(found("::ffff:10.1.2.3"), ["::ffff:10.1.2.3"]);
    }

    #[test]
    fn leaves_versions_times_and_paths() {
        for text in [
            "v1.4.0",
            "1.4.0",
            "12:30:45",
            "websign_host::engine",
            "a::b",
            "999.1.1.1",
            "0x000000A0",
            "2026-09-30",
            "cafe:babe",
        ] {
            assert!(found(text).is_empty(), "{text}");
        }
    }
}
