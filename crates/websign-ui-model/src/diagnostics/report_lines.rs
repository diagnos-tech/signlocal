//! One formatter per section of the diagnostics report.

use super::atr_mask::mask_atr;
use super::report::{
    BrowserLine, CertificateCounts, DeviceLine, ErrorLine, ModuleLine, ReportInput,
};

/// How many recent errors the report carries.
const MAX_ERRORS: usize = 20;

pub(super) fn header(input: &ReportInput) -> [String; 4] {
    [
        "WebeSign diagnostics v1".to_owned(),
        format!(
            "app: {} ({}, {}) · protocol {} · locale {} · scale {}%",
            input.app_version,
            input.packaging,
            input.arch,
            input.protocol,
            input.locale,
            input.scale_percent
        ),
        format!("os: {}", input.os),
        format!("render: {}", input.render),
    ]
}

pub(super) fn browser(line: &BrowserLine) -> String {
    let mut text = format!(
        "{} {} · extension {} · host {}",
        line.name,
        line.version.as_deref().unwrap_or("?"),
        line.extension_version.as_deref().unwrap_or("not seen"),
        if line.host_registered {
            "registered"
        } else {
            "not registered"
        }
    );
    if let Some(ping) = &line.last_ping {
        text.push_str(&format!(" · last ping {ping}"));
    }
    text
}

pub(super) fn device(line: &DeviceLine) -> String {
    match line {
        DeviceLine::Usb {
            vid_pid,
            hint_id,
            certs,
        } => format!(
            "usb {vid_pid} {} · certs {certs}",
            hint_id.as_deref().unwrap_or("unknown")
        ),
        DeviceLine::Reader {
            name,
            hint_id,
            atr,
            certs,
        } => {
            let card = match (hint_id, atr) {
                (Some(id), _) => format!("card {id}"),
                (None, Some(atr)) => format!("atr {}", mask_atr(atr)),
                (None, None) => "atr none".to_owned(),
            };
            format!("reader \"{name}\" · {card} · certs {certs}")
        }
    }
}

pub(super) fn module(line: &ModuleLine) -> String {
    let outcome = match &line.result {
        Ok((slots, tokens)) => format!("loaded · slots {slots} · tokens {tokens}"),
        Err(reason) => format!("failed: {reason}"),
    };
    let suffix = if line.user_added { " (user-added)" } else { "" };
    format!("{} · {outcome}{suffix}", line.path)
}

pub(super) fn certificates(counts: &CertificateCounts) -> [String; 3] {
    let hidden = counts.hidden_expired + counts.hidden_login_only + counts.hidden_other;
    let other = if counts.hidden_other > 0 {
        format!(", other {}", counts.hidden_other)
    } else {
        String::new()
    };
    [
        format!(
            "usable {} (os {}, pkcs11 {}, deduplicated {}) · hidden {hidden} (expired {}, login-only {}{other})",
            counts.usable_os + counts.usable_pkcs11,
            counts.usable_os,
            counts.usable_pkcs11,
            counts.deduplicated,
            counts.hidden_expired,
            counts.hidden_login_only,
        ),
        format!(
            "kinds: {} · keys: {}",
            tally(&counts.kinds),
            tally(&counts.keys)
        ),
        format!("expiring<=30d {}", counts.expiring_within_30_days),
    ]
}

fn tally(entries: &[(String, u32)]) -> String {
    if entries.is_empty() {
        return "none".to_owned();
    }
    let parts: Vec<String> = entries
        .iter()
        .map(|(name, count)| format!("{name} {count}"))
        .collect();
    parts.join(", ")
}

/// The newest [`MAX_ERRORS`] errors, oldest first.
pub(super) fn recent_errors(errors: &[ErrorLine]) -> impl Iterator<Item = String> + '_ {
    let skip = errors.len().saturating_sub(MAX_ERRORS);
    errors.iter().skip(skip).map(|error| {
        let mut text = format!(
            "{} {} {} {}",
            error.at, error.operation, error.code, error.source
        );
        if let Some(native) = &error.native {
            text.push(' ');
            text.push_str(native);
        }
        text
    })
}
