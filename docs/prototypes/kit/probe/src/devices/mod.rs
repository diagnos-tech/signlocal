//! Hardware that might hold a certificate: USB devices and smart card readers.
//!
//! Only descriptive data (IDs, names, ATR); nothing here talks to a card.

use std::process::ExitCode;

/// Arguments of `websign-probe devices`.
#[derive(Debug, clap::Args)]
pub struct Args {
    /// Print JSON instead of a table.
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &Args) -> anyhow::Result<ExitCode> {
    let _ = args;
    todo!("devices command")
}

/// A Markdown section (starting at `###`) describing every device found, for
/// `websign-probe report`. Must not contain serial numbers or card contents.
pub fn markdown_section() -> String {
    todo!("devices report section")
}
