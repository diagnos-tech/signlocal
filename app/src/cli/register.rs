//! `websign register`: native messaging manifests only; the kit's
//! `websign-probe register`, kept for tests and support.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::ValueEnum;

use super::options::BrowserArg;

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
    /// Also allow this Chromium extension ID (repeatable).
    #[arg(long = "extension-id", value_name = "ID")]
    pub extension_ids: Vec<String>,
    /// Windows: folder for the manifest files the registry keys point to.
    #[arg(long, value_name = "DIR")]
    pub manifest_dir: Option<PathBuf>,
    #[arg(long)]
    pub dry_run: bool,
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
    let _ = args;
    todo!("desktop-api.md §register")
}
