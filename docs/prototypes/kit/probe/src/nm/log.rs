//! Diagnostic log of the native host.
//!
//! Browsers start the host without a terminal and discard its stderr, so this
//! file is the only evidence of what happened between the extension and the
//! app. It records who connected and what happened, and nothing else: never a
//! PIN, digest, signature, certificate, name or the site that asked. Each
//! recording method takes only values that cannot carry such data.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::launch::BrowserLaunch;
use super::protocol::ClientInfo;
use crate::config::SLUG;

/// The file is emptied at startup when it grows past this size.
const MAX_LOG_BYTES: u64 = 1024 * 1024;
/// Client-supplied text is cut to this many characters before it is logged.
const MAX_FIELD_CHARS: usize = 160;

/// Where the host log lives: the same path for every host process, so one
/// file tells the story of all connections.
pub fn log_path() -> PathBuf {
    std::env::temp_dir().join(format!("{SLUG}-probe-host.log"))
}

/// Append-only log with the process context prefixed to every line.
#[derive(Debug, Clone)]
pub struct HostLog {
    path: Option<PathBuf>,
    context: String,
}

impl HostLog {
    /// Opens the shared log for a host started by `launch`.
    pub fn open(launch: &BrowserLaunch) -> Self {
        let path = log_path();
        if fs::metadata(&path).is_ok_and(|meta| meta.len() > MAX_LOG_BYTES) {
            let _ = fs::remove_file(&path);
        }
        Self {
            path: Some(path),
            context: format!(
                "pid={} family={} ext={}",
                std::process::id(),
                launch.family,
                sanitize(&launch.extension_id)
            ),
        }
    }

    /// A log that records nothing, for tests.
    #[cfg(test)]
    pub fn disabled() -> Self {
        Self {
            path: None,
            context: String::new(),
        }
    }

    pub fn started(&self, parent_window: bool) {
        self.line(format_args!(
            "event=start version={} os={} arch={} parent_window={parent_window}",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH,
        ));
    }

    /// `kind` is one of the fixed request types, never client text.
    pub fn request(&self, kind: &'static str, bytes: usize) {
        self.line(format_args!("event=request type={kind} bytes={bytes}"));
    }

    /// `outcome` is `ok` or a stable error code.
    pub fn reply(&self, outcome: &'static str, bytes: usize, elapsed: Duration) {
        self.line(format_args!(
            "event=reply outcome={outcome} bytes={bytes} ms={}",
            elapsed.as_millis()
        ));
    }

    /// The extension's self-description from a `ping`: one line per connection.
    pub fn client(&self, client: &ClientInfo) {
        self.line(format_args!(
            "event=client extension_version={} reason={} user_agent={}",
            field(client.extension_version.as_deref()),
            field(client.reason.as_deref()),
            field(client.user_agent.as_deref()),
        ));
    }

    /// What the host tried to sign with, without identifying the certificate.
    pub fn signing(&self, hash: &str, algorithm: &str, digest_bytes: usize) {
        self.line(format_args!(
            "event=sign hash={hash} algorithm={algorithm} digest_bytes={digest_bytes}"
        ));
    }

    /// The native call that produced a signature and whether it verified.
    pub fn signed(&self, api: &str, verified: bool) {
        self.line(format_args!("event=signed api={api} verified={verified}"));
    }

    pub fn certificates(&self, count: usize, warnings: usize) {
        self.line(format_args!(
            "event=certificates count={count} warnings={warnings}"
        ));
    }

    /// Why the host stopped: `eof`, `write-error`, `frame-error`, ...
    pub fn stopped(&self, reason: &'static str) {
        self.line(format_args!("event=stop reason={reason}"));
    }

    fn line(&self, message: std::fmt::Arguments<'_>) {
        let Some(path) = &self.path else { return };
        let line = format!(
            "{} {} {message}\n",
            timestamp(SystemTime::now()),
            self.context
        );
        // A log that cannot be written must never take the host down.
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = file.write_all(line.as_bytes());
        }
    }
}

/// Client text made safe for a one-line, space-separated record.
fn field(value: Option<&str>) -> String {
    value.map_or_else(|| "-".to_owned(), |text| format!("\"{}\"", sanitize(text)))
}

fn sanitize(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control() && *c != '"')
        .take(MAX_FIELD_CHARS)
        .collect()
}

/// `2026-09-29T18:04:05.123Z`, without a date library.
fn timestamp(time: SystemTime) -> String {
    let since_epoch = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = since_epoch.as_secs();
    let (year, month, day) = civil_from_days(i64::try_from(secs / 86_400).unwrap_or(0));
    let rest = secs % 86_400;
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60,
        since_epoch.subsec_millis()
    )
}

/// Howard Hinnant's days-to-civil algorithm.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(month <= 2), month, day)
}

#[cfg(test)]
mod tests;
