//! Opt-in progress trace on stderr, switched on by `WEBSIGN_PROBE_TRACE=1`.
//!
//! When a native call never returns (a provider waiting for a dialog nobody
//! can see, a module stuck in `C_Initialize`), the last trace line says
//! which one. Lines name steps, APIs, providers and module file names only:
//! never a subject, token label, key container name, serial number or PIN.

use std::ffi::OsStr;
use std::fmt;
use std::sync::OnceLock;
use std::time::Instant;

/// The environment variable that turns the trace on.
pub const ENV: &str = "WEBSIGN_PROBE_TRACE";

/// Writes one trace line when the trace is on; formats nothing otherwise.
macro_rules! trace {
    ($($arg:tt)*) => {
        $crate::trace::emit(format_args!($($arg)*))
    };
}
pub(crate) use trace;

/// Use [`trace!`] instead.
pub fn emit(message: fmt::Arguments<'_>) {
    if let Some(started) = started() {
        // stderr is unbuffered: the line is out before the next native call.
        eprintln!("[trace +{:>6} ms] {message}", started.elapsed().as_millis());
    }
}

/// When the process started tracing, or `None` when the trace is off.
fn started() -> Option<Instant> {
    static STARTED: OnceLock<Option<Instant>> = OnceLock::new();
    *STARTED.get_or_init(|| is_on(std::env::var_os(ENV).as_deref()).then(Instant::now))
}

/// Any value but empty, `0`, `false` or `off` turns the trace on.
fn is_on(value: Option<&OsStr>) -> bool {
    value
        .and_then(OsStr::to_str)
        .map(str::trim)
        .is_some_and(|value| {
            !value.is_empty()
                && !["0", "false", "off"]
                    .iter()
                    .any(|off| value.eq_ignore_ascii_case(off))
        })
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::is_on;

    #[test]
    fn only_a_meaningful_value_turns_the_trace_on() {
        for on in ["1", "true", "yes", " 1 "] {
            assert!(is_on(Some(OsStr::new(on))), "{on:?}");
        }
        for off in ["", "0", "false", "OFF", "  "] {
            assert!(!is_on(Some(OsStr::new(off))), "{off:?}");
        }
        assert!(!is_on(None));
    }
}
