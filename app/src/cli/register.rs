//! `websign register`: native messaging manifests only; the kit's
//! `websign-probe register`, kept for tests and support.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::ValueEnum;
use websign_registration::{Action, Request, Scope};

use super::install::manifest_steps;
use super::options::BrowserArg;
use super::report::Report;

/// `websign register`.
#[derive(Debug, clap::Args)]
pub struct RegisterArgs {
    /// Browser to register with (repeatable). Default: all.
    #[arg(long = "browser", value_enum, value_name = "BROWSER")]
    pub browsers: Vec<BrowserArg>,
    /// Remove the registration instead of writing it.
    #[arg(long)]
    pub uninstall: bool,
    /// Where manifests go.
    #[arg(long, value_enum, default_value_t = ScopeArg::User)]
    pub scope: ScopeArg,
    /// Register only in `<DIR>/NativeMessagingHosts/` (Chromium started with
    /// `--user-data-dir=<DIR>`; Linux and macOS). For tests.
    #[arg(long, value_name = "DIR")]
    pub user_data_dir: Option<PathBuf>,
    /// Windows: folder for the manifest files the registry keys point to.
    #[arg(long, value_name = "DIR")]
    pub manifest_dir: Option<PathBuf>,
    /// Print what would be done; change nothing.
    #[arg(long)]
    pub dry_run: bool,
    /// Print a JSON report instead of text.
    #[arg(long)]
    pub json: bool,
}

/// `--scope`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ScopeArg {
    User,
    System,
}

/// Runs `websign_registration::run` and prints one line (or JSON) per target.
pub fn run(args: &RegisterArgs) -> ExitCode {
    let report = Report {
        command: "register",
        system: args.scope == ScopeArg::System,
        dry_run: args.dry_run,
        steps: manifest_steps(&request(args)),
    };
    report.print(args.json)
}

/// The registration request the flags describe.
pub fn request(args: &RegisterArgs) -> Request {
    Request {
        action: if args.uninstall {
            Action::Uninstall
        } else {
            Action::Install
        },
        scope: match args.scope {
            ScopeArg::User => Scope::User,
            ScopeArg::System => Scope::System,
        },
        browsers: args.browsers.iter().map(|&b| b.into()).collect(),
        user_data_dir: args.user_data_dir.clone(),
        manifest_dir: args.manifest_dir.clone(),
        host: None,
        dry_run: args.dry_run,
    }
}
