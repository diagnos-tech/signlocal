//! How keys and certificates are described in command output.

use probe_core::{CertInfo, PublicKeyKind};

use super::inventory::Entry;
use crate::keystores::PinPrompt;

/// One line per key, with personal data only when `show_names` is set.
pub fn describe(entry: &Entry, show_names: bool) -> String {
    let source = source_label(entry);
    match &entry.info {
        Ok(info) => format!(
            "{name} | {kind} | {key} | until {until} | {source} | {fp}",
            name = if show_names {
                info.display_name()
            } else {
                redacted(info)
            },
            kind = certificate_kind(info),
            key = key_label(&info.key),
            until = date(info.not_after),
            fp = &info.fingerprint.to_hex()[..16],
        ),
        Err(error) => format!("<unreadable certificate: {error}> | {source}"),
    }
}

/// Where the key lives and who asks for its PIN.
pub fn source_label(entry: &Entry) -> String {
    let key = &entry.key;
    let hardware = match key.hardware {
        Some(true) => ", hardware",
        Some(false) => ", software",
        None => "",
    };
    let pin = match key.pin {
        PinPrompt::System => "PIN by OS",
        PinPrompt::App {
            protected_path: true,
        } => "PIN pad",
        PinPrompt::App {
            protected_path: false,
        } => "PIN by app",
    };
    format!("{} ({}{hardware}; {pin})", key.keystore, key.provider)
}

/// "ICP-Brasil A3", "eIDAS qualified (QSCD)", or "certificate".
pub fn certificate_kind(info: &CertInfo) -> String {
    let mut parts = Vec::new();
    if let Some(icp) = &info.icp_brasil {
        parts.push(match icp.level {
            Some(level) => format!("ICP-Brasil {level}"),
            None => "ICP-Brasil".to_owned(),
        });
    }
    if let Some(qualified) = info.qualified.as_ref().filter(|q| q.compliance) {
        parts.push(
            if qualified.sscd {
                "eIDAS qualified (QSCD)"
            } else {
                "eIDAS qualified"
            }
            .to_owned(),
        );
    }
    if !info.can_sign() {
        parts.push("cannot sign".to_owned());
    }
    if parts.is_empty() {
        "certificate".to_owned()
    } else {
        parts.join(", ")
    }
}

pub fn key_label(key: &PublicKeyKind) -> String {
    match key {
        PublicKeyKind::Rsa { bits } => format!("RSA-{bits}"),
        PublicKeyKind::Ec { curve } => format!("EC {}", curve.name()),
        PublicKeyKind::Unsupported { oid } => format!("unsupported key {oid}"),
    }
}

/// Issuer only: enough to tell certificates apart without naming anyone.
fn redacted(info: &CertInfo) -> String {
    let issuer = info
        .issuer
        .common_name
        .as_deref()
        .unwrap_or("unknown issuer");
    format!("<holder hidden, issued by {issuer}>")
}

/// `YYYY-MM-DD` (UTC) for Unix seconds, without a date library.
pub fn date(unix_secs: i64) -> String {
    // Howard Hinnant's days-to-civil algorithm.
    let days = unix_secs.div_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
mod tests {
    use super::date;

    #[test]
    fn formats_unix_dates() {
        assert_eq!(date(0), "1970-01-01");
        assert_eq!(date(951_782_400), "2000-02-29");
        assert_eq!(date(1_798_761_599), "2026-12-31");
        assert_eq!(date(-86_400), "1969-12-31");
    }
}
