//! How the device snapshot is written: a plain-text table for people at a
//! terminal and a Markdown section for the reports.

use std::fmt::Write as _;

use super::Snapshot;
use super::pcsc::{CardState, Reader};
use super::usb::UsbDevice;

/// Uppercase hex without separators (`3B8F8001`), the ATR spelling used everywhere.
pub fn hex_upper(bytes: &[u8]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut text, byte| {
            let _ = write!(text, "{byte:02X}");
            text
        })
}

/// USB-IF class code as a word: `CCID`, `HID`, `hub`...
fn class_name(class: u8) -> String {
    match class {
        0x01 => "audio",
        0x02 => "CDC",
        0x03 => "HID",
        0x07 => "printer",
        0x08 => "mass storage",
        0x09 => "hub",
        0x0B => "CCID",
        0x0E => "video",
        0xE0 => "wireless",
        0xEF => "misc",
        0xFF => "vendor",
        other => return format!("0x{other:02X}"),
    }
    .to_owned()
}

fn class_list(device: &UsbDevice) -> String {
    device
        .classes
        .iter()
        .map(|class| class_name(*class))
        .collect::<Vec<_>>()
        .join("+")
}

fn card_text(reader: &Reader) -> String {
    let base = match reader.card {
        CardState::Empty => "empty",
        CardState::Present => "card present",
        CardState::Unresponsive => "card not responding",
        CardState::Unknown => "unknown",
    };
    if reader.in_use {
        format!("{base}, in use")
    } else {
        base.to_owned()
    }
}

fn names(device: &UsbDevice) -> String {
    match (&device.manufacturer, &device.product) {
        (Some(maker), Some(product)) => format!("{maker} {product}"),
        (Some(text), None) | (None, Some(text)) => text.clone(),
        (None, None) => "(unnamed)".to_owned(),
    }
}

/// The terminal output of `websign-probe devices`.
pub fn text(snapshot: &Snapshot) -> String {
    let mut out = String::from("USB smart card readers and tokens:\n");
    match &snapshot.usb.problem {
        Some(problem) => {
            let _ = writeln!(out, "  {problem}");
        }
        None if snapshot.usb.devices.is_empty() => out.push_str("  none found\n"),
        None => {
            for device in &snapshot.usb.devices {
                let _ = writeln!(
                    out,
                    "  {}  {:<14} {}",
                    device.id(),
                    class_list(device),
                    names(device)
                );
            }
        }
    }
    if snapshot.usb.hidden > 0 {
        let _ = writeln!(
            out,
            "  ({} other USB device(s) not shown; use --all-usb)",
            snapshot.usb.hidden
        );
    }

    out.push_str("\nPC/SC readers:\n");
    match &snapshot.readers.problem {
        Some(problem) => {
            let _ = writeln!(out, "  {problem}");
        }
        None if snapshot.readers.readers.is_empty() => out.push_str("  none found\n"),
        None => {
            for reader in &snapshot.readers.readers {
                let _ = writeln!(out, "  {}: {}", reader.name, card_text(reader));
                if let Some(atr) = &reader.atr {
                    let _ = writeln!(out, "      ATR {atr}");
                }
            }
        }
    }
    out
}

/// The `### Devices` section of the report, ending with a newline.
pub fn markdown(snapshot: &Snapshot) -> String {
    let mut out = String::from("### Devices\n\n**USB smart card readers and tokens**\n\n");
    match &snapshot.usb.problem {
        Some(problem) => {
            let _ = writeln!(out, "_{problem}_");
        }
        None if snapshot.usb.devices.is_empty() => out.push_str("_none found_\n"),
        None => {
            out.push_str("| VID:PID | Class | Manufacturer / product |\n|---|---|---|\n");
            for device in &snapshot.usb.devices {
                let _ = writeln!(
                    out,
                    "| `{}` | {} | {} |",
                    device.id(),
                    class_list(device),
                    names(device)
                );
            }
        }
    }
    if snapshot.usb.hidden > 0 {
        let _ = writeln!(
            out,
            "\n_{} other USB device(s) not listed._",
            snapshot.usb.hidden
        );
    }

    out.push_str("\n**PC/SC readers**\n\n");
    match &snapshot.readers.problem {
        Some(problem) => {
            let _ = writeln!(out, "_{problem}_");
        }
        None if snapshot.readers.readers.is_empty() => out.push_str("_none found_\n"),
        None => {
            out.push_str("| Reader | State | ATR |\n|---|---|---|\n");
            for reader in &snapshot.readers.readers {
                let atr = reader
                    .atr
                    .as_deref()
                    .map_or_else(|| "—".to_owned(), |atr| format!("`{atr}`"));
                let _ = writeln!(out, "| {} | {} | {atr} |", reader.name, card_text(reader));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests;
