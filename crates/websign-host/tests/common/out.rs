//! What the engine produced during one step, with readers for each port.

use secrecy::ExposeSecret;
use websign_core::{HashAlgorithm, SignatureAlgorithm};
use websign_host::ports::KeyCommand;
use websign_keystores::KeyRef;
use websign_protocol::messages::{DiagnosticsTab, NeedDigest, SignResult};
use websign_protocol::{AppEnvelope, AppMessage, ErrorCode, WireError};
use websign_ui_model::confirm::UiCommand;
use websign_ui_model::confirm::port::{Failure, Finish, OpenRequest, RequestKey};

/// A `KeyCommand::Sign` with its secret opened for comparison.
#[derive(Debug, Clone)]
pub struct SignCall {
    pub tag: u64,
    pub key: KeyRef,
    pub hash: HashAlgorithm,
    pub algorithm: SignatureAlgorithm,
    pub digest: Vec<u8>,
    pub pin: Option<String>,
    pub parent_window: Option<isize>,
}

#[derive(Default)]
pub struct Out {
    pub frames: Vec<AppEnvelope>,
    pub ui: Vec<UiCommand>,
    pub keys: Vec<KeyCommand>,
    pub diagnostics: Vec<Option<DiagnosticsTab>>,
}

impl Out {
    /// Nothing at all was sent anywhere.
    pub fn is_silent(&self) -> bool {
        self.frames.is_empty() && self.ui.is_empty() && self.keys.is_empty()
    }

    // ---- frames ----

    pub fn errors(&self) -> Vec<(String, WireError)> {
        self.frames
            .iter()
            .filter_map(|f| match &f.message {
                AppMessage::Error(e) => Some((f.id.to_string(), e.clone())),
                _ => None,
            })
            .collect()
    }

    /// The codes of the error frames, in order.
    pub fn error_codes(&self) -> Vec<ErrorCode> {
        self.errors().into_iter().map(|(_, e)| e.code).collect()
    }

    /// The code of the only error frame; panics with context otherwise.
    pub fn only_error(&self) -> (String, ErrorCode) {
        let errors = self.errors();
        assert_eq!(
            errors.len(),
            1,
            "expected one error frame in {:?}",
            self.kinds()
        );
        (errors[0].0.clone(), errors[0].1.code)
    }

    pub fn need_digests(&self) -> Vec<(String, NeedDigest)> {
        self.frames
            .iter()
            .filter_map(|f| match &f.message {
                AppMessage::NeedDigest(n) => Some((f.id.to_string(), n.clone())),
                _ => None,
            })
            .collect()
    }

    pub fn results(&self) -> Vec<(String, SignResult)> {
        self.frames
            .iter()
            .filter_map(|f| match &f.message {
                AppMessage::SignResult(r) => Some((f.id.to_string(), r.clone())),
                _ => None,
            })
            .collect()
    }

    /// The wire `type` of each frame.
    pub fn kinds(&self) -> Vec<&'static str> {
        self.frames
            .iter()
            .map(|f| match &f.message {
                AppMessage::Hello(_) => "hello",
                AppMessage::Status(_) => "status",
                AppMessage::ChooseResult(_) => "choose.result",
                AppMessage::NeedDigest(_) => "sign.need_digest",
                AppMessage::SignResult(_) => "sign.result",
                AppMessage::Done(_) => "done",
                AppMessage::Error(_) => "error",
            })
            .collect()
    }

    /// Every frame as JSON text, for "this must never appear" checks.
    pub fn frames_json(&self) -> String {
        self.frames
            .iter()
            .map(|f| String::from_utf8(websign_protocol::to_json(f)).expect("utf8"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    // ---- window ----

    pub fn opens(&self) -> Vec<OpenRequest> {
        self.ui
            .iter()
            .filter_map(|c| match c {
                UiCommand::Open(o) => Some(o.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn finished(&self) -> Vec<(RequestKey, Finish)> {
        self.ui
            .iter()
            .filter_map(|c| match c {
                UiCommand::Finished { key, finish } => Some((*key, *finish)),
                _ => None,
            })
            .collect()
    }

    pub fn failures(&self) -> Vec<Failure> {
        self.ui
            .iter()
            .filter_map(|c| match c {
                UiCommand::Failed { failure, .. } => Some(failure.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn digest_ready_count(&self) -> usize {
        self.ui
            .iter()
            .filter(|c| matches!(c, UiCommand::DigestReady { .. }))
            .count()
    }

    pub fn digest_pending_count(&self) -> usize {
        self.ui
            .iter()
            .filter(|c| matches!(c, UiCommand::DigestPending { .. }))
            .count()
    }

    pub fn certificates_count(&self) -> usize {
        self.ui
            .iter()
            .filter(|c| matches!(c, UiCommand::Certificates { .. }))
            .count()
    }

    pub fn signing_count(&self) -> usize {
        self.ui
            .iter()
            .filter(|c| matches!(c, UiCommand::Signing { .. }))
            .count()
    }

    // ---- key store ----

    pub fn signs(&self) -> Vec<SignCall> {
        self.keys
            .iter()
            .filter_map(|c| match c {
                KeyCommand::Sign {
                    tag,
                    key,
                    hash,
                    algorithm,
                    digest,
                    pin,
                    parent_window,
                } => Some(SignCall {
                    tag: *tag,
                    key: *key,
                    hash: *hash,
                    algorithm: *algorithm,
                    digest: digest.clone(),
                    pin: pin.as_ref().map(|p| p.expose_secret().to_owned()),
                    parent_window: *parent_window,
                }),
                _ => None,
            })
            .collect()
    }

    /// The `refresh` flag of each `List`, in order.
    pub fn lists(&self) -> Vec<bool> {
        self.keys
            .iter()
            .filter_map(|c| match c {
                KeyCommand::List { refresh } => Some(*refresh),
                _ => None,
            })
            .collect()
    }

    /// Tags of the `Chain` commands.
    pub fn chain_tags(&self) -> Vec<u64> {
        self.keys
            .iter()
            .filter_map(|c| match c {
                KeyCommand::Chain { tag, .. } => Some(*tag),
                _ => None,
            })
            .collect()
    }

    /// The command kinds in order: `list`, `sign`, `chain`, `invalidate`, `end`.
    pub fn key_kinds(&self) -> Vec<&'static str> {
        self.keys
            .iter()
            .map(|c| match c {
                KeyCommand::List { .. } => "list",
                KeyCommand::Sign { .. } => "sign",
                KeyCommand::Chain { .. } => "chain",
                KeyCommand::Invalidate => "invalidate",
                KeyCommand::EndSessions => "end",
            })
            .collect()
    }
}
