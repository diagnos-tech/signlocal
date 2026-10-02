//! Command-line entry point. Each subcommand owns its arguments in its module.

use std::process::ExitCode;

use clap::{Parser, Subcommand};

use crate::{commands, devices, nm};

#[derive(Debug, Parser)]
#[command(name = "websign-probe", version, about, propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// List signing certificates from every key source.
    List(commands::list::Args),
    /// Sign random digests with the selected certificates and verify them.
    Sign(commands::sign::Args),
    /// List USB devices and smart card readers (with ATR).
    Devices(devices::Args),
    /// Register this binary as native messaging host in every browser.
    Register(nm::register::Args),
    /// Run as native messaging host on stdin/stdout (for manual tests).
    Host(nm::host::Args),
    /// Run everything and write a Markdown report without personal data.
    Report(commands::report::Args),
}

pub fn run() -> ExitCode {
    let result = match Cli::parse().command {
        Command::List(args) => commands::list::run(&args),
        Command::Sign(args) => commands::sign::run(&args),
        Command::Devices(args) => devices::run(&args),
        Command::Register(args) => nm::register::run(&args),
        Command::Host(args) => nm::host::run(&args),
        Command::Report(args) => commands::report::run(&args),
    };
    match result {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}
