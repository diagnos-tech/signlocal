//! Hardware that might hold a certificate: USB devices and smart card readers.
//!
//! Only descriptive data (IDs, names, ATR); nothing here talks to a card.
//! USB serial numbers are never read, and the reports built from this data
//! are published, so nothing that identifies a person or a unit may enter.

mod format;
mod pcsc;
mod usb;

use std::process::ExitCode;

use serde::Serialize;

/// Arguments of `websign-probe devices`.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Print JSON instead of a table.
    #[arg(long)]
    pub json: bool,
    /// List every USB device, not only smart card readers and tokens (CCID).
    #[arg(long)]
    pub all_usb: bool,
}

/// What was found, and what could not be looked at (no PC/SC service, no
/// permission to enumerate USB): a machine without hardware is not an error.
#[derive(Debug, Serialize)]
pub struct Snapshot {
    pub usb: usb::UsbScan,
    pub readers: pcsc::ReaderScan,
}

impl Snapshot {
    fn scan(all_usb: bool) -> Snapshot {
        Snapshot {
            usb: usb::scan(all_usb),
            readers: pcsc::scan(),
        }
    }
}

pub fn run(args: &Args) -> anyhow::Result<ExitCode> {
    let snapshot = Snapshot::scan(args.all_usb);
    if args.json {
        println!("{}", serde_json::to_string_pretty(&snapshot)?);
    } else {
        print!("{}", format::text(&snapshot));
    }
    Ok(ExitCode::SUCCESS)
}

/// A Markdown section (starting at `###`) describing every device found, for
/// `websign-probe report`. Must not contain serial numbers or card contents.
pub fn markdown_section() -> String {
    format::markdown(&Snapshot::scan(false))
}
