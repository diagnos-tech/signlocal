//! Reading the `Effect`s a flow returns, for the flow-level tests.

use websign_host::flow::Effect;
use websign_host::ports::KeyCommand;
use websign_protocol::{AppMessage, ErrorCode};
use websign_ui_model::confirm::UiCommand;
use websign_ui_model::confirm::port::{Failure, Finish};

/// One short word per effect, in order: `send:need_digest`, `ui:open`,
/// `keys:list`, `consent`, `error`.
pub fn words(effects: &[Effect]) -> Vec<String> {
    effects.iter().map(word).collect()
}

fn word(effect: &Effect) -> String {
    match effect {
        Effect::Send(message) => format!("send:{}", message_word(message)),
        Effect::Ui(command) => format!("ui:{}", ui_word(command)),
        Effect::Keys(command) => format!("keys:{}", keys_word(command)),
        Effect::RecordConsent { .. } => "consent".to_owned(),
        Effect::RecordError { .. } => "error".to_owned(),
    }
}

fn message_word(message: &AppMessage) -> &'static str {
    match message {
        AppMessage::Hello(_) => "hello",
        AppMessage::Status(_) => "status",
        AppMessage::ChooseResult(_) => "choose_result",
        AppMessage::NeedDigest(_) => "need_digest",
        AppMessage::SignResult(_) => "sign_result",
        AppMessage::Done(_) => "done",
        AppMessage::Error(_) => "error",
    }
}

fn ui_word(command: &UiCommand) -> &'static str {
    match command {
        UiCommand::Open(_) => "open",
        UiCommand::Certificates { .. } => "certificates",
        UiCommand::SlowListing { .. } => "slow_listing",
        UiCommand::DigestPending { .. } => "digest_pending",
        UiCommand::DigestReady { .. } => "digest_ready",
        UiCommand::Signing { .. } => "signing",
        UiCommand::Failed { .. } => "failed",
        UiCommand::Finished { .. } => "finished",
        UiCommand::Queue { .. } => "queue",
        UiCommand::Hide => "hide",
    }
}

fn keys_word(command: &KeyCommand) -> &'static str {
    match command {
        KeyCommand::List { .. } => "list",
        KeyCommand::Sign { .. } => "sign",
        KeyCommand::Chain { .. } => "chain",
        KeyCommand::Invalidate => "invalidate",
        KeyCommand::EndSessions => "end_sessions",
    }
}

/// The code of the first `Send(Error)`.
pub fn sent_error(effects: &[Effect]) -> Option<ErrorCode> {
    effects.iter().find_map(|e| match e {
        Effect::Send(AppMessage::Error(error)) => Some(error.code),
        _ => None,
    })
}

/// The first `Ui(Failed)`.
pub fn failure(effects: &[Effect]) -> Option<Failure> {
    effects.iter().find_map(|e| match e {
        Effect::Ui(UiCommand::Failed { failure, .. }) => Some(failure.clone()),
        _ => None,
    })
}

/// The first `Ui(Finished)`.
pub fn finish(effects: &[Effect]) -> Option<Finish> {
    effects.iter().find_map(|e| match e {
        Effect::Ui(UiCommand::Finished { finish, .. }) => Some(*finish),
        _ => None,
    })
}

/// The sequence number of the first `Send(NeedDigest)`.
pub fn need_digest_seq(effects: &[Effect]) -> Option<u32> {
    effects.iter().find_map(|e| match e {
        Effect::Send(AppMessage::NeedDigest(need)) => Some(need.seq),
        _ => None,
    })
}

/// The first `Keys(Sign)` as `(tag, digest, via)`.
pub fn sign_command(effects: &[Effect]) -> Option<(u64, Vec<u8>, usize)> {
    effects.iter().find_map(|e| match e {
        Effect::Keys(KeyCommand::Sign {
            tag, digest, key, ..
        }) => Some((*tag, digest.clone(), key.path)),
        _ => None,
    })
}
