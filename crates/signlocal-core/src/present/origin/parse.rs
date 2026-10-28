//! Splitting a serialized origin into scheme, host and port.

use super::OriginError;
use super::host::Host;

/// Longest origin accepted; a DNS name is at most 253 characters.
const MAX_LEN: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Scheme {
    Http,
    Https,
}

impl Scheme {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
        }
    }

    pub(super) fn default_port(self) -> u16 {
        match self {
            Self::Http => 80,
            Self::Https => 443,
        }
    }
}

#[derive(Debug)]
pub(super) struct Parsed {
    pub scheme: Scheme,
    pub host: Host,
    pub port: Option<u16>,
}

/// Parses `scheme://host[:port]`, with at most one trailing `/`.
///
/// A scheme other than `http`/`https` is `Insecure`, once the text is known
/// to look like an origin at all.
pub(super) fn parse(origin: &str) -> Result<Parsed, OriginError> {
    if origin.len() > MAX_LEN {
        return Err(OriginError::Malformed);
    }
    let (scheme, rest) = origin.split_once("://").ok_or(OriginError::Malformed)?;
    if !is_scheme(scheme) {
        return Err(OriginError::Malformed);
    }
    let scheme = match scheme.to_ascii_lowercase().as_str() {
        "https" => Scheme::Https,
        "http" => Scheme::Http,
        _ => return Err(OriginError::Insecure),
    };
    let authority = rest.strip_suffix('/').unwrap_or(rest);
    if authority.chars().any(is_forbidden) {
        return Err(OriginError::Malformed);
    }
    let (host, port) = split_port(authority)?;
    Ok(Parsed {
        scheme,
        host: Host::parse(host)?,
        port,
    })
}

fn is_scheme(text: &str) -> bool {
    text.starts_with(|c: char| c.is_ascii_alphabetic())
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

/// Characters that mean the text is a URL rather than an origin (path, query,
/// fragment, credentials) or cannot appear in a host at all.
fn is_forbidden(c: char) -> bool {
    matches!(c, '/' | '\\' | '?' | '#' | '@' | '%') || c.is_whitespace() || c.is_control()
}

fn split_port(authority: &str) -> Result<(&str, Option<u16>), OriginError> {
    let host_end = if authority.starts_with('[') {
        authority.find(']').ok_or(OriginError::Malformed)? + 1
    } else {
        authority.find(':').unwrap_or(authority.len())
    };
    let (host, tail) = authority.split_at(host_end);
    if tail.is_empty() {
        return Ok((host, None));
    }
    let digits = tail.strip_prefix(':').ok_or(OriginError::Malformed)?;
    Ok((host, Some(parse_port(digits)?)))
}

fn parse_port(digits: &str) -> Result<u16, OriginError> {
    if !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(OriginError::Malformed);
    }
    match digits.parse::<u16>() {
        Ok(port) if port != 0 => Ok(port),
        _ => Err(OriginError::Malformed),
    }
}
