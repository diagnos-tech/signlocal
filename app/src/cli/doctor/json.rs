//! `websign doctor --json`: the report's input as a stable JSON document
//! (keys never renamed; new keys may be added). Same data as the text,
//! so it carries no personal data either.

use serde_json::{Value, json};
use websign_ui_model::diagnostics::report::{
    BrowserLine, CertificateCounts, DeviceLine, ErrorLine, ModuleLine, ReportInput,
};

/// The document for `input`.
pub fn to_json(input: &ReportInput) -> Value {
    json!({
        "app": {
            "version": input.app_version,
            "packaging": input.packaging,
            "arch": input.arch,
            "protocol": input.protocol,
            "locale": input.locale,
        },
        "os": input.os,
        "browsers": input.browsers.iter().map(browser).collect::<Vec<_>>(),
        "devices": input.devices.iter().map(device).collect::<Vec<_>>(),
        "pkcs11": input.modules.iter().map(module).collect::<Vec<_>>(),
        "certificates": certificates(&input.certificates),
        "complement": input.complement,
        "recentErrors": input.recent_errors.iter().map(error).collect::<Vec<_>>(),
    })
}

fn browser(line: &BrowserLine) -> Value {
    json!({
        "name": line.name,
        "version": line.version,
        "extensionVersion": line.extension_version,
        "hostRegistered": line.host_registered,
        "lastPing": line.last_ping,
    })
}

fn device(line: &DeviceLine) -> Value {
    match line {
        DeviceLine::Usb {
            vid_pid,
            hint_id,
            certs,
        } => json!({
            "kind": "usb", "vidPid": vid_pid, "hint": hint_id, "certificates": certs,
        }),
        // An unknown card's ATR is masked exactly as in the text report.
        DeviceLine::Reader {
            name,
            hint_id,
            atr,
            certs,
        } => json!({
            "kind": "reader",
            "name": name,
            "hint": hint_id,
            "atr": if hint_id.is_none() {
                atr.as_deref().map(websign_ui_model::diagnostics::atr_mask::mask_atr)
            } else {
                None
            },
            "certificates": certs,
        }),
    }
}

fn module(line: &ModuleLine) -> Value {
    let (tokens, failure) = match &line.result {
        Ok((_, tokens)) => (Some(*tokens), None),
        Err(reason) => (None, Some(reason)),
    };
    json!({ "module": line.path, "userAdded": line.user_added, "tokens": tokens, "failure": failure })
}

fn certificates(counts: &CertificateCounts) -> Value {
    let tally = |entries: &[(String, u32)]| -> Value {
        entries
            .iter()
            .map(|(name, count)| (name.clone(), Value::from(*count)))
            .collect()
    };
    json!({
        "usableOs": counts.usable_os,
        "usablePkcs11": counts.usable_pkcs11,
        "deduplicated": counts.deduplicated,
        "hiddenExpired": counts.hidden_expired,
        "hiddenLoginOnly": counts.hidden_login_only,
        "hiddenOther": counts.hidden_other,
        "kinds": tally(&counts.kinds),
        "keys": tally(&counts.keys),
        "expiringWithin30Days": counts.expiring_within_30_days,
    })
}

fn error(line: &ErrorLine) -> Value {
    json!({
        "at": line.at, "operation": line.operation, "code": line.code,
        "source": line.source, "native": line.native,
    })
}
