//! `websign-probe list`.

use std::process::ExitCode;

use super::view;
use crate::keystores::Options;
use crate::keystores::inventory::Inventory;

/// Arguments of `websign-probe list`.
#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(flatten)]
    pub options: Options,
    /// Show every path to each certificate, not only the preferred one.
    #[arg(long)]
    pub every_path: bool,
}

pub fn run(args: &Args) -> anyhow::Result<ExitCode> {
    let inventory = Inventory::collect(&args.options);
    for warning in inventory.warnings() {
        eprintln!("warning: {warning}");
    }
    let groups = inventory.groups();
    if groups.is_empty() {
        println!("No signing certificates found.");
    }
    for (index, group) in groups.iter().enumerate() {
        let primary = &inventory.entries[group.primary];
        println!("{:>2}. {}", index + 1, view::describe(primary, true));
        if args.every_path {
            for &alternate in &group.alternates {
                let label = view::source_label(&inventory.entries[alternate]);
                println!("    also via {label}");
            }
        } else if !group.alternates.is_empty() {
            println!(
                "    (+{} other path(s); --every-path to show)",
                group.alternates.len()
            );
        }
    }
    Ok(ExitCode::SUCCESS)
}
