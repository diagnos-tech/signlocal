//! How a web origin is judged and shown (`docs/ux.md` §4.3, vectors §16.2).
//!
//! The registrable domain (eTLD+1, Public Suffix List) is what a person must
//! read, because the classic phishing host is
//! `yourbank.com.evil.example`. Everything else is dimmed.

mod host;
mod parse;

use parse::Scheme;

/// An origin split for display and policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormattedOrigin {
    /// Canonical serialization used as the consent key: lowercase scheme and
    /// host, punycode (ASCII) host, default port omitted
    /// (`"https://app.diagnos.health"`).
    pub canonical: String,
    /// Dimmed text before the emphasized part: scheme, `://` and subdomains
    /// with their trailing dot (`"https://app."`).
    pub prefix: String,
    /// Emphasized part: registrable domain, or the whole host for IP
    /// addresses, `localhost` and hosts that are themselves a public suffix.
    pub registrable: String,
    /// Non-default port, shown dimmed after the host (`Some(8443)`).
    pub port: Option<u16>,
    /// Unicode rendering when the host has an IDN label (`"diаgnos.health"`),
    /// shown under the punycode form.
    pub unicode: Option<String>,
    pub warning: Option<OriginWarning>,
    /// Whether "Remember this site" may be offered: false for IP addresses
    /// and IDN hosts, which are hard to check by eye.
    pub can_remember: bool,
}

/// A non-blocking alert shown under the origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginWarning {
    /// A label is not ASCII (shown as punycode).
    Idn,
    /// A public IP literal.
    PublicIp,
    /// A private, link-local or unique-local IP literal (RFC 1918, RFC 4193…).
    LocalIp,
    /// `localhost`, `*.localhost`, `127.0.0.0/8` or `[::1]`.
    Localhost,
}

/// Origins the app must refuse to sign for.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OriginError {
    /// Not a serialized `scheme://host[:port]` origin (paths, credentials,
    /// `null`, over-long, empty host…).
    #[error("malformed origin")]
    Malformed,
    /// Not a secure context: `http:` other than localhost, or any scheme
    /// other than `http`/`https` (`file:`, `data:`, extensions).
    #[error("insecure origin")]
    Insecure,
}

/// Parses, classifies and splits a serialized origin.
pub fn format_origin(origin: &str) -> Result<FormattedOrigin, OriginError> {
    let parsed = parse::parse(origin)?;
    let host = &parsed.host;
    if parsed.scheme == Scheme::Http && !host.is_loopback() {
        return Err(OriginError::Insecure);
    }

    let scheme = parsed.scheme.as_str();
    let text = host.text();
    let port = parsed
        .port
        .filter(|port| *port != parsed.scheme.default_port());
    let canonical = match port {
        Some(port) => format!("{scheme}://{text}:{port}"),
        None => format!("{scheme}://{text}"),
    };

    let registrable = host.registrable();
    let subdomains = text.strip_suffix(registrable.as_str()).unwrap_or_default();
    let unicode = host.unicode();
    let warning = host.warning(unicode.is_some());
    Ok(FormattedOrigin {
        canonical,
        prefix: format!("{scheme}://{subdomains}"),
        registrable,
        port,
        unicode,
        warning,
        can_remember: !matches!(
            warning,
            Some(OriginWarning::Idn | OriginWarning::LocalIp | OriginWarning::PublicIp)
        ),
    })
}
