//! `websign-probe report`: one Markdown file with everything a proof needs,
//! and nothing personal — safe to paste into `docs/prototypes/`.

use std::fmt::Write as _;
use std::process::ExitCode;

use anyhow::Context;

use super::inventory::{Entry, Inventory};
use super::{sign, view};
use crate::devices;

/// Arguments of `websign-probe report`.
#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(flatten)]
    pub sign: sign::Args,
    /// Also sign and verify with every selected certificate.
    #[arg(long)]
    pub run_signatures: bool,
    /// Write the report here instead of printing it.
    #[arg(long, value_name = "FILE")]
    pub out: Option<std::path::PathBuf>,
}

pub fn run(args: &Args) -> anyhow::Result<ExitCode> {
    let mut inventory = Inventory::collect(&args.sign.options);
    let mut report = String::new();
    header(&mut report);
    sources(&mut report, &inventory);
    certificates(&mut report, &inventory);
    report.push_str(&devices::markdown_section());
    report.push('\n');

    let mut failed = false;
    if args.run_signatures {
        let selected = sign::select(&inventory, &args.sign)?;
        let results = sign::run_cases(&mut inventory, &selected, &args.sign, |_| {}, |_| {})?;
        failed = results.iter().any(|result| result.outcome.is_err());
        report.push_str("### Signatures\n\n| # | Result |\n|---|---|\n");
        for result in &results {
            let _ = writeln!(
                report,
                "| {} | `{}` |",
                result.entry + 1,
                result.line().trim()
            );
        }
        report.push('\n');
    }

    match &args.out {
        Some(path) => std::fs::write(path, &report)
            .with_context(|| format!("cannot write {}", path.display()))?,
        None => print!("{report}"),
    }
    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

fn header(report: &mut String) {
    let _ = writeln!(
        report,
        "## websign-probe {} — {} {}\n",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
    );
}

fn sources(report: &mut String, inventory: &Inventory) {
    report.push_str("### Key sources\n\n");
    for keystore in &inventory.opened.keystores {
        let _ = writeln!(report, "- `{}`: opened", keystore.name());
    }
    for failure in &inventory.opened.failures {
        let _ = writeln!(report, "- `{}`: **{}**", failure.source, failure.error);
    }
    report.push('\n');
}

fn certificates(report: &mut String, inventory: &Inventory) {
    report.push_str("### Certificates\n\n");
    report.push_str("| # | Holder | Type | Key | Valid until | Source | Other paths |\n");
    report.push_str("|---|---|---|---|---|---|---|\n");
    for group in inventory.deduped() {
        let index = position(inventory, group.primary) + 1;
        let others: Vec<String> = group
            .alternates
            .iter()
            .map(|entry| view::source_label(entry))
            .collect();
        let _ = writeln!(
            report,
            "| {index} | {} | {} |",
            row(group.primary),
            if others.is_empty() {
                "—".to_owned()
            } else {
                others.join("<br>")
            },
        );
    }
    report.push('\n');
}

/// Holder is always redacted here: reports end up in a public repository.
fn row(entry: &Entry) -> String {
    let source = view::source_label(entry);
    match &entry.info {
        Ok(info) => format!(
            "<hidden> | {} | {} | {} | {source}",
            view::certificate_kind(info),
            view::key_label(&info.key),
            view::date(info.not_after),
        ),
        Err(error) => format!("<unreadable: {error}> | | | | {source}"),
    }
}

fn position(inventory: &Inventory, target: &Entry) -> usize {
    inventory
        .entries
        .iter()
        .position(|entry| std::ptr::eq(entry, target))
        .unwrap_or_default()
}
