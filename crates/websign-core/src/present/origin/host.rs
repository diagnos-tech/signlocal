//! Host classification: names, IP literals, loopback, eTLD+1.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::{OriginError, OriginWarning};

/// A validated host in its canonical (lowercase, ASCII) spelling.
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
        if text.chars().all(|c| c.is_ascii_digit() || c == '.') {
            let ip = text
                .parse::<Ipv4Addr>()
                .map_err(|_| OriginError::Malformed)?;
            return Ok(Self::Ip(IpAddr::V4(ip)));
        }
        let ascii = idna::domain_to_ascii(text).map_err(|_| OriginError::Malformed)?;
        if ascii.split('.').any(str::is_empty) {
            return Err(OriginError::Malformed);
        }
        Ok(Self::Name(ascii))
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
            Self::Ip(ip) => ip.is_loopback(),
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
            Self::Ip(ip) if is_local(ip) => Some(OriginWarning::LocalIp),
            Self::Ip(_) => Some(OriginWarning::PublicIp),
            Self::Name(_) => None,
        }
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
