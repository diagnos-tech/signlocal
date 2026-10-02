//! What can go wrong, with the protocol's stable codes.

use std::error::Error;
use std::fmt;

use websign_protocol::ErrorCode;

use crate::hint::{docs_url, hint};

/// Why the app could not be started or the connection broke: a message for
/// developers plus the underlying failure, reachable through
/// [`std::error::Error::source`] of the enclosing [`ClientError`].
///
/// The message never names a path or a person: paths usually contain the
/// user's name.
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct Reason {
    message: String,
    #[source]
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl Reason {
    /// The message, without the underlying failure.
    pub fn message(&self) -> &str {
        &self.message
    }

    pub(crate) fn caused_by(
        message: impl Into<String>,
        source: impl Error + Send + Sync + 'static,
    ) -> Reason {
        Reason {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

impl From<String> for Reason {
    fn from(message: String) -> Reason {
        Reason {
            message,
            source: None,
        }
    }
}

impl From<&str> for Reason {
    fn from(message: &str) -> Reason {
        Reason::from(message.to_owned())
    }
}

/// A failed call. Branch on [`ClientError::code`], never on the message;
/// [`ClientError::hint`] says what to do about it.
///
/// `ClientError` implements [`std::error::Error`]: the underlying I/O or
/// protocol failure of [`ClientError::AppMissing`] and
/// [`ClientError::Connection`] is its `source()`, so `anyhow`, `eyre` and
/// `{:#}`-style printers show the whole chain.
///
/// ```
/// use websign_client::{ClientError, ErrorCode};
///
/// fn explain(error: &ClientError) -> String {
///     match error.code() {
///         ErrorCode::UserCancelled => "cancelled".into(),
///         code => format!("{code:?}: {error}\nhint: {}", error.hint()),
///     }
/// }
/// # let _ = explain;
/// ```
#[derive(Debug)]
pub enum ClientError {
    /// The `websign` executable was not found or could not start.
    AppMissing(Reason),
    /// The app answered with an error (`UserCancelled`, `PinLocked`, …).
    App {
        /// The protocol's stable code.
        code: ErrorCode,
        /// For developers, in English.
        message: String,
    },
    /// `prepare` failed; the request was cancelled in the app.
    Prepare(String),
    /// The pipe broke or the app sent something unparseable.
    Connection(Reason),
}

impl ClientError {
    /// The protocol code for this error (`AppMissing`, `Aborted` for a
    /// failed `prepare`, `Internal` for connection failures).
    pub fn code(&self) -> ErrorCode {
        match self {
            ClientError::AppMissing(_) => ErrorCode::AppMissing,
            ClientError::App { code, .. } => *code,
            ClientError::Prepare(_) => ErrorCode::Aborted,
            ClientError::Connection(_) => ErrorCode::Internal,
        }
    }

    /// What a developer can do about this error, in English.
    pub fn hint(&self) -> &'static str {
        hint(self.code())
    }

    /// The code's section on the project site.
    pub fn docs_url(&self) -> String {
        docs_url(self.code())
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::AppMissing(reason) => {
                write!(f, "the WebeSign app is not installed: {reason}")
            }
            ClientError::App { code, message } => write!(f, "{code:?}: {message}"),
            ClientError::Prepare(why) => write!(f, "prepare failed: {why}"),
            ClientError::Connection(reason) => {
                write!(f, "connection to the app failed: {reason}")
            }
        }
    }
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ClientError::AppMissing(reason) | ClientError::Connection(reason) => {
                // Skip `Reason` itself: its message is already in `Display`.
                reason.source.as_deref().map(|source| source as _)
            }
            ClientError::App { .. } | ClientError::Prepare(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_source_is_the_underlying_failure_not_the_reason() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let error = ClientError::AppMissing(Reason::caused_by("cannot start the app", io));
        assert_eq!(
            error.source().map(ToString::to_string).as_deref(),
            Some("gone")
        );
        assert!(error.to_string().ends_with("cannot start the app"));
    }

    #[test]
    fn errors_without_a_cause_have_no_source() {
        let error = ClientError::Connection("the app exited".into());
        assert!(error.source().is_none());
        assert_eq!(error.code(), ErrorCode::Internal);
    }

    #[test]
    fn every_code_has_a_hint() {
        for code in [
            ErrorCode::UserCancelled,
            ErrorCode::AppMissing,
            ErrorCode::Busy,
        ] {
            let error = ClientError::App {
                code,
                message: String::new(),
            };
            assert!(error.hint().len() > 20);
        }
    }

    #[test]
    fn errors_are_send_and_sync() {
        fn check<T: Send + Sync + 'static>() {}
        check::<ClientError>();
    }
}
