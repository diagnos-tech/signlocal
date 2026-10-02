//! The app's log: a size-capped file in the OS's per-user log folder
//! ([`log_dir`]), because browsers discard a native host's stderr. Level
//! from `WEBSIGN_LOG` (`off`, `error`…`trace`, default `info`).
//!
//! Privacy rule for every `log::` call in the workspace: steps, API names,
//! error codes, counts, sizes and module file names only — never a name,
//! document number, digest, signature, certificate, PIN, token label, serial
//! number or site (`docs/architecture/security.md` §Logs). Every line also
//! passes through [`redact::Redactor`] before it reaches the disk.

mod location;
mod redact;
mod sink;
#[cfg(test)]
mod tests;

use std::sync::Mutex;

use log::{LevelFilter, Log, Metadata, Record};

pub use location::log_dir;
use redact::Redactor;
use sink::Sink;

/// Environment variable with the level.
pub const LEVEL_ENV: &str = "WEBSIGN_LOG";

/// Installs the file logger and a panic hook that logs the panic. Never
/// fails: without a writable log folder the app runs unlogged.
pub fn init() {
    let level = level_from(std::env::var(LEVEL_ENV).ok().as_deref());
    if level == LevelFilter::Off {
        return;
    }
    let Some(sink) = log_dir().and_then(|dir| Sink::open(&dir)) else {
        return;
    };
    let logger = FileLogger {
        sink: Mutex::new(sink),
        redactor: Redactor::from_env(),
    };
    if log::set_boxed_logger(Box::new(logger)).is_ok() {
        log::set_max_level(level);
        install_panic_hook();
    }
}

/// `text` with personal data removed, for reports that must follow the log
/// rules (diagnostics): same filter as every log line.
pub fn redact_line(text: &str) -> String {
    Redactor::from_env().apply(text)
}

/// `WEBSIGN_LOG`'s level; anything unrecognized is the default.
fn level_from(value: Option<&str>) -> LevelFilter {
    value
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(LevelFilter::Info)
}

/// A panic in a helper thread otherwise vanishes with the discarded stderr.
fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let place = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_default();
        log::error!("panic at {place}: {info}");
        previous(info);
    }));
}

/// Writes redacted lines to the file.
struct FileLogger {
    sink: Mutex<Sink>,
    redactor: Redactor,
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format_line(&self.redactor, record);
        if let Ok(mut sink) = self.sink.lock() {
            sink.write_line(&line);
        }
    }

    fn flush(&self) {}
}

/// `2026-09-30T12:00:00Z INFO 4242 websign_host::engine: <redacted message>`.
/// The process id tells apart the lines of concurrent host processes.
fn format_line(redactor: &Redactor, record: &Record<'_>) -> String {
    let message = redactor.apply(&record.args().to_string());
    format!(
        "{} {:<5} {} {}: {}\n",
        jiff::Timestamp::now().strftime("%Y-%m-%dT%H:%M:%SZ"),
        record.level(),
        std::process::id(),
        record.target(),
        message.replace('\n', " | "),
    )
}
