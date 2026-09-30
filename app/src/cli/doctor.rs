//! `websign doctor`: the diagnostics report on stdout, for support and CI.
//!
//! The data comes from the diagnostics window's own collector
//! ([`crate::ui::diagnostics::report::terminal_input`]), so the terminal and
//! the window's "Copy diagnostics" can never disagree.

pub mod json;

use std::process::ExitCode;

use websign_ui_model::diagnostics::report::render;

use super::output::{SUCCESS, print_json};

/// `websign doctor`.
#[derive(Debug, clap::Args)]
pub struct DoctorArgs {
    /// Print the report's data as JSON instead of the text.
    #[arg(long)]
    pub json: bool,
}

/// Prints `websign_ui_model::diagnostics::report::render` (or its input as
/// JSON). Exit 0 even when things are red: the report is the answer.
pub fn run(args: &DoctorArgs) -> ExitCode {
    let input = crate::ui::diagnostics::report::terminal_input();
    if args.json {
        print_json(&json::to_json(&input).to_string());
    } else {
        print!("{}", render(&input));
    }
    ExitCode::from(SUCCESS)
}
