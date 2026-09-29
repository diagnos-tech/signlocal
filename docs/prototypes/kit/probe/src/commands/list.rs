//! `websign-probe list`.

use std::process::ExitCode;

use super::inventory::Inventory;
use super::view;
use crate::keystores::Options;

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
    for failure in &inventory.opened.failures {
        eprintln!("warning: {}: {}", failure.source, failure.error);
    }
    let groups = inventory.deduped();
    if groups.is_empty() {
        println!("No signing certificates found.");
    }
    for (index, group) in groups.iter().enumerate() {
        println!("{:>2}. {}", index + 1, view::describe(group.primary, true));
        for alternate in &group.alternates {
            let label = view::source_label(alternate);
            if args.every_path {
                println!("    also via {label}");
            }
        }
        if !args.every_path && !group.alternates.is_empty() {
            println!(
                "    (+{} other path(s); --every-path to show)",
                group.alternates.len()
            );
        }
    }
    Ok(ExitCode::SUCCESS)
}
