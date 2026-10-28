//! Host classification: names, IP literals, loopback, eTLD+1.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use idna::AsciiDenyList;

use super::{OriginError, OriginWarning};

/// A validated host in its canonical (lowercase, ASCII) spelling.
///
/// IPv4-mapped IPv6 literals (`[::ffff:127.0.0.1]`) are classified as the
/// IPv4 address they carry, and keep their IPv6 spelling.
#[derive(Debug)]
pub(super) enum Host {
    Ip(IpAddr),
    Name(String),
}

impl Host {
    /// `text` is the host as written: a name, a dotted IPv4 or `[IPv6]`.
    pub(super) fn parse(text: &str) -> Result<Self, OriginError> {
        if let Some(inner) = text.strip_prefix('[') {
            let inner = inner.strip_suffix(']').ok_or(OriginError::Malformed)?;
            let ip = inner
                .parse::<Ipv6Addr>()
                .map_err(|_| OriginError::Malformed)?;
            return Ok(Self::Ip(IpAddr::V6(ip)));
        }
        if text.is_empty() {
            return Err(OriginError::Malformed);
        }
        let ascii = idna::domain_to_ascii_cow(text.as_bytes(), AsciiDenyList::URL)
            .map_err(|_| OriginError::Malformed)?;
        if ascii.split('.').any(str::is_empty) {
            return Err(OriginError::Malformed);
        }
        if ends_in_a_number(&ascii) {
            // Browsers serialize every IPv4 host as a dotted quad, so the
            // other WHATWG spellings (`127.1`, `0x7f.0.0.1`) are refused.
            let ip = ascii
                .parse::<Ipv4Addr>()
                .map_err(|_| OriginError::Malformed)?;
            return Ok(Self::Ip(IpAddr::V4(ip)));
        }
        Ok(Self::Name(ascii.into_owned()))
    }

    /// The host as it appears in the canonical origin.
    pub(super) fn text(&self) -> String {
        match self {
            Self::Ip(IpAddr::V6(ip)) => format!("[{ip}]"),
            Self::Ip(ip) => ip.to_string(),
            Self::Name(name) => name.clone(),
        }
    }

    /// Whether plain `http` is acceptable: the host cannot leave the machine.
    pub(super) fn is_loopback(&self) -> bool {
        match self {
            Self::Ip(ip) => ip.to_canonical().is_loopback(),
            Self::Name(name) => is_localhost(name),
        }
    }

    /// The part a person must read: eTLD+1 from the Public Suffix List, or
    /// the whole host for IPs, `localhost` and public suffixes themselves.
    pub(super) fn registrable(&self) -> String {
        match self {
            Self::Name(name) if !is_localhost(name) => {
                psl::domain_str(name).unwrap_or(name).to_owned()
            }
            _ => self.text(),
        }
    }

    /// The Unicode spelling, when some label is punycode.
    pub(super) fn unicode(&self) -> Option<String> {
        let Self::Name(name) = self else {
            return None;
        };
        name.split('.')
            .any(|label| label.starts_with("xn--"))
            .then(|| idna::domain_to_unicode(name).0)
    }

    pub(super) fn warning(&self, is_idn: bool) -> Option<OriginWarning> {
        if is_idn {
            return Some(OriginWarning::Idn);
        }
        if self.is_loopback() {
            return Some(OriginWarning::Localhost);
        }
        match self {
            Self::Ip(ip) if is_local(&ip.to_canonical()) => Some(OriginWarning::LocalIp),
            Self::Ip(_) => Some(OriginWarning::PublicIp),
            Self::Name(_) => None,
        }
    }
}

/// The WHATWG URL "ends in a number" check: a host whose last label is all
/// digits, or `0x` and hex digits, is an IPv4 address, never a name.
fn ends_in_a_number(host: &str) -> bool {
    let last = host.rsplit('.').next().unwrap_or(host);
    let hex = last.strip_prefix("0x").or_else(|| last.strip_prefix("0X"));
    match hex {
        Some(digits) => digits.bytes().all(|b| b.is_ascii_hexdigit()),
        None => !last.is_empty() && last.bytes().all(|b| b.is_ascii_digit()),
    }
}

fn is_localhost(name: &str) -> bool {
    name == "localhost" || name.ends_with(".localhost")
}

/// RFC 1918, link-local and unique-local (RFC 4193) addresses.
fn is_local(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => ip.is_private() || ip.is_link_local(),
        IpAddr::V6(ip) => {
            let first = ip.segments()[0];
            first & 0xffc0 == 0xfe80 || first & 0xfe00 == 0xfc00
        }
    }
}
