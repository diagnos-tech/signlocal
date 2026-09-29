//! `websign-probe report`: one Markdown file with everything a proof needs,
//! and nothing personal — safe to paste into `docs/prototypes/`.

use std::fmt::Write as _;
use std::process::ExitCode;

use anyhow::Context;

use super::{sign, view};
use crate::devices;
use crate::keystores::inventory::{Entry, Inventory};

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

    let report = without_home(&report, std::env::home_dir());
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

/// `text` with the home directory spelled `~`: module paths in source errors
/// often sit under it, and the directory is named after the user.
fn without_home(text: &str, home: Option<std::path::PathBuf>) -> String {
    let home = home.as_deref().and_then(std::path::Path::to_str);
    match home.map(|home| home.trim_end_matches(['/', '\\'])) {
        // A home of "/" (or none) would turn every path into nonsense.
        Some(home) if home.len() > 1 => text.replace(home, "~"),
        _ => text.to_owned(),
    }
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
    for group in inventory.groups() {
        let others: Vec<String> = group
            .alternates
            .iter()
            .map(|&alternate| view::source_label(&inventory.entries[alternate]))
            .collect();
        // Numbered by entry, like the signature rows below.
        let _ = writeln!(
            report,
            "| {} | {} | {} |",
            group.primary + 1,
            row(&inventory.entries[group.primary]),
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
/// (Italics, not `<hidden>`: Markdown renderers swallow unknown tags.)
fn row(entry: &Entry) -> String {
    let source = view::source_label(entry);
    match &entry.info {
        Ok(info) => format!(
            "_hidden_ | {} | {} | {} | {source}",
            view::certificate_kind(info),
            view::key_label(&info.key),
            view::date(info.not_after),
        ),
        Err(error) => format!("_unreadable: {error}_ | | | | {source}"),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::without_home;

    #[test]
    fn the_home_directory_never_reaches_the_report() {
        let home = Some(PathBuf::from("/home/ana/"));
        assert_eq!(
            without_home("module file not found: /home/ana/lib/x.so", home),
            "module file not found: ~/lib/x.so"
        );
        let windows = Some(PathBuf::from(r"C:\Users\ana"));
        assert_eq!(
            without_home(r"cannot load C:\Users\ana\x.dll", windows),
            r"cannot load ~\x.dll"
        );
        assert_eq!(without_home("/x", Some(PathBuf::from("/"))), "/x");
        assert_eq!(without_home("/x", None), "/x");
    }
}
