//! The "Copy diagnostics" text (`docs/ux.md` §8.7): the facts of a scan
//! mapped onto `websign_ui_model::diagnostics::report`, which renders it.
//!
//! Only counts, versions, model IDs and codes cross over: the input types
//! of the report cannot hold names, sites or serial numbers, and this module
//! never reads certificate subjects.

use jiff::Timestamp;
use websign_host::store::ErrorRecord;
use websign_registration::status::RegistrationState;
use websign_ui_model::diagnostics::report::{
    BrowserLine, DeviceLine, ErrorLine, ModuleLine, ReportInput, render,
};

use super::counts::certificate_counts;
use super::facts::{BrowserFact, Facts, ScanInput, browsers, collect, packaging_name};
use crate::platform::channel::install_format;

/// What the report says about this copy of the app, known when the window
/// opens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    pub app_version: String,
    /// `"msix"`, `"direct"`, `"deb"`, … ([`super::facts::packaging_name`]).
    pub packaging: String,
    pub arch: String,
    pub locale: String,
    pub scale_percent: u32,
    /// `"wgpu/vulkan"`, `"glow"`.
    pub render: String,
    /// `"n/a"` outside the macOS store build.
    pub complement: String,
}

impl Environment {
    /// This copy of the app, drawn at `scale_percent` by `render`.
    pub fn of_this_app(scale_percent: u32, render: String) -> Environment {
        let app = crate::host_process::app_info();
        Environment {
            app_version: app.version,
            packaging: packaging_name(install_format()).to_owned(),
            arch: app.arch,
            locale: crate::ui::i18n::locale().tag().to_owned(),
            scale_percent,
            render,
            // Direct builds load drivers themselves (`docs/plan.md` D10).
            complement: "n/a".to_owned(),
        }
    }
}

/// The report of this machine for `websign doctor`: the window's collector,
/// without a window (nothing scaled, nothing rendered). Blocks while drivers
/// load and readers answer.
pub fn terminal_input() -> ReportInput {
    let scan = ScanInput::current();
    let facts = collect(&scan);
    input(
        &facts,
        &Environment::of_this_app(100, "n/a (terminal)".to_owned()),
        scan.now,
    )
}

/// The exact text "Copy diagnostics" puts on the clipboard.
pub fn text(facts: &Facts, environment: &Environment, now: Timestamp) -> String {
    render(&input(facts, environment, now))
}

/// The report's input for `facts`.
pub fn input(facts: &Facts, environment: &Environment, now: Timestamp) -> ReportInput {
    ReportInput {
        app_version: environment.app_version.clone(),
        packaging: environment.packaging.clone(),
        arch: environment.arch.clone(),
        protocol: websign_protocol::version::PROTOCOL_VERSION,
        locale: environment.locale.clone(),
        scale_percent: environment.scale_percent,
        os: facts.os.clone(),
        render: environment.render.clone(),
        browsers: facts.browsers.iter().map(browser_line).collect(),
        devices: device_lines(facts),
        modules: facts
            .drivers
            .iter()
            .map(|driver| ModuleLine {
                path: driver.path.clone(),
                // Slots are not reported by the key stores; each token that
                // brought certificates sits in one.
                result: driver.result.clone().map(|tokens| (tokens, tokens)),
                user_added: driver.user_added,
            })
            .collect(),
        certificates: certificate_counts(&facts.certificates, now),
        complement: environment.complement.clone(),
        recent_errors: facts.recent_errors.iter().map(error_line).collect(),
    }
}

fn browser_line(fact: &BrowserFact) -> BrowserLine {
    let connection = fact.connection.as_ref();
    BrowserLine {
        name: browsers::short_name(fact.browser).to_owned(),
        version: fact
            .version
            .as_deref()
            .or(connection.map(|record| record.browser_version.as_str()))
            .map(browsers::short_version),
        extension_version: connection.map(|record| record.extension_version.clone()),
        host_registered: fact.registration == RegistrationState::Registered,
        last_ping: connection.map(|record| minute(record.last_seen)),
    }
}

fn device_lines(facts: &Facts) -> Vec<DeviceLine> {
    let tokens = facts.devices.tokens.iter().map(|token| DeviceLine::Usb {
        vid_pid: token.vid_pid.clone(),
        hint_id: token.hint.as_ref().map(|hint| hint.id.clone()),
        certs: token.certificates.count(),
    });
    let readers = facts.devices.readers.iter().map(|reader| {
        let card = reader.card.as_ref();
        DeviceLine::Reader {
            name: reader.name.clone(),
            hint_id: card
                .and_then(|card| card.hint.as_ref())
                .map(|hint| hint.id.clone()),
            atr: card.and_then(|card| card.atr.clone()),
            certs: card.map_or(0, |card| card.certificates.count()),
        }
    });
    tokens.chain(readers).collect()
}

fn error_line(record: &ErrorRecord) -> ErrorLine {
    ErrorLine {
        at: minute(record.at),
        operation: record.operation.clone(),
        code: record.code.clone(),
        source: record.source.clone(),
        native: record.native.clone(),
    }
}

/// `2026-09-29T14:02Z`: UTC, minute precision, as every report time.
pub fn minute(unix_seconds: i64) -> String {
    Timestamp::from_second(unix_seconds)
        .map(|at| at.strftime("%Y-%m-%dT%H:%MZ").to_string())
        .unwrap_or_else(|_| "?".to_owned())
}
