//! The command line: the desktop API for scripts and programs, and the
//! installer's entry points (`docs/architecture/desktop-api.md`).
//!
//! Machine-facing commands (`sign`, `choose`, `connect`, and anything with
//! `--json`) write exactly one JSON document to stdout and nothing else;
//! people-facing text goes to stderr. Exit codes are stable
//! ([`websign_protocol::ErrorCode::exit_code`]; 2 for a usage error, which
//! clap reports itself).

pub mod activate;
mod choose;
mod connect;
mod diagnostics;
mod doctor;
mod help;
mod install;
mod local;
mod options;
mod output;
mod register;
mod report;
mod sign;
#[cfg(test)]
mod tests;
mod version;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

/// Sign with your certificate for any website or desktop program.
#[derive(Debug, Parser)]
#[command(
    name = "websign",
    version,
    about,
    disable_help_subcommand = true,
    after_help = help::MAIN
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Every command. No command = open diagnostics.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Register with every browser, the websign: URL scheme and the app menu.
    Install(install::InstallArgs),
    /// Undo `install` (settings are kept unless --purge).
    Uninstall(install::UninstallArgs),
    /// Write or remove native messaging manifests only (low level).
    Register(register::RegisterArgs),
    /// Print the diagnostics report (the "Copy diagnostics" text).
    Doctor(doctor::DoctorArgs),
    /// Open the diagnostics window.
    Diagnostics(diagnostics::DiagnosticsArgs),
    /// Sign a digest: opens the confirmation window, prints the result as JSON.
    #[command(after_help = help::SIGN)]
    Sign(sign::SignArgs),
    /// Let the person choose a certificate; prints it as JSON.
    #[command(after_help = help::CHOOSE)]
    Choose(choose::ChooseArgs),
    /// Speak the framed protocol on stdin/stdout (for client libraries).
    #[command(after_help = help::CONNECT)]
    Connect(connect::ConnectArgs),
    /// Print the version.
    Version(version::VersionArgs),
}

/// Parses the command line and runs the command.
pub fn run() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        None => diagnostics::run(&diagnostics::DiagnosticsArgs::default()),
        Some(Command::Install(args)) => install::run_install(&args),
        Some(Command::Uninstall(args)) => install::run_uninstall(&args),
        Some(Command::Register(args)) => register::run(&args),
        Some(Command::Doctor(args)) => doctor::run(&args),
        Some(Command::Diagnostics(args)) => diagnostics::run(&args),
        Some(Command::Sign(args)) => sign::run(&args),
        Some(Command::Choose(args)) => choose::run(&args),
        Some(Command::Connect(args)) => connect::run(&args),
        Some(Command::Version(args)) => version::run(&args),
    }
}
