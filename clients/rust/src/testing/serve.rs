//! The fake app's side of the pipes: reads requests, answers like the app.

use std::io::{PipeReader, PipeWriter};
use std::sync::{Arc, Mutex, PoisonError};

use websign_protocol::framing::{read_frame, write_frame};
use websign_protocol::messages::{
    ChooseResult, Done, HelloReply, NeedDigest, SignBegin, SignResult, StatusReply,
};
use websign_protocol::types::{
    AppInfo, Base64Bytes, Certificate, Channel, HashName, OsName, SignatureAlgorithmName,
};
use websign_protocol::{
    AppEnvelope, AppMessage, ClientEnvelope, ClientMessage, ErrorCode, ProtocolRange, RequestId,
    WireError, parse_client_message, to_json,
};

use super::app::{Config, Stage};

/// One signature waiting for its digest.
struct OpenSign {
    id: RequestId,
    hash: HashName,
    certificate: Certificate,
    algorithm: SignatureAlgorithmName,
}

struct Server {
    config: Arc<Config>,
    requests: Arc<Mutex<Vec<ClientMessage>>>,
    output: PipeWriter,
    version: Option<u32>,
    open: Option<OpenSign>,
}

/// Serves one connection until the client closes its end.
pub(super) fn serve(
    config: Arc<Config>,
    requests: Arc<Mutex<Vec<ClientMessage>>>,
    mut input: PipeReader,
    output: PipeWriter,
) {
    let mut server = Server {
        config,
        requests,
        output,
        version: None,
        open: None,
    };
    while let Ok(Some(frame)) = read_frame(&mut input) {
        match parse_client_message(&frame, server.version) {
            Ok(envelope) => server.handle(envelope),
            Err(error) => match error.id {
                Some(id) => server.fail(&id, error.code, &error.message),
                None => return,
            },
        }
    }
}

impl Server {
    fn handle(&mut self, request: ClientEnvelope) {
        let ClientEnvelope { id, message, .. } = request;
        self.requests
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(message.clone());
        match message {
            ClientMessage::Hello(_) => {
                self.version = Some(1);
                self.reply(
                    &id,
                    AppMessage::Hello(HelloReply {
                        app: app_info(),
                        protocol: 1,
                    }),
                );
            }
            ClientMessage::Status(_) => {
                let remembered = self.config.remembered;
                self.reply(
                    &id,
                    AppMessage::Status(StatusReply {
                        app: app_info(),
                        remembered,
                    }),
                );
            }
            ClientMessage::OpenDiagnostics(_) => self.reply(&id, AppMessage::Done(Done {})),
            ClientMessage::Cancel(_) => {
                self.open = None;
                self.fail(&id, ErrorCode::Aborted, "cancelled by the caller");
            }
            ClientMessage::Choose(choose) => {
                let wanted = choose.filter.and_then(|filter| filter.algorithms);
                self.choose(&id, wanted.as_deref());
            }
            ClientMessage::SignBegin(begin) => self.begin(&id, begin),
            ClientMessage::SignDigest(digest) => self.finish(&id, digest.digest),
        }
    }

    fn failure_at(&self, stage: Stage) -> Option<(ErrorCode, String)> {
        let failure = self.config.failure.as_ref().filter(|f| f.stage == stage)?;
        Some((failure.code, failure.message.clone()))
    }

    fn choose(&mut self, id: &RequestId, wanted: Option<&[SignatureAlgorithmName]>) {
        if let Some((code, message)) = self.failure_at(Stage::Choose) {
            return self.fail(id, code, &message);
        }
        let found: Vec<Certificate> = self.config.usable(wanted).cloned().collect();
        if found.is_empty() {
            return self.fail(
                id,
                ErrorCode::NoCertificates,
                "no certificate matches the filter",
            );
        }
        self.reply(
            id,
            AppMessage::ChooseResult(ChooseResult {
                certificates: found,
            }),
        );
    }

    fn begin(&mut self, id: &RequestId, begin: SignBegin) {
        if let Some((code, message)) = self.failure_at(Stage::Choose) {
            return self.fail(id, code, &message);
        }
        let wanted = begin.algorithms.as_deref();
        let certificate = self.config.usable(wanted).find(|c| {
            begin
                .certificate
                .as_ref()
                .is_none_or(|f| *f == c.fingerprint)
        });
        let Some(certificate) = certificate.cloned() else {
            return self.fail(id, ErrorCode::CertificateUnavailable, "no such certificate");
        };
        let algorithm = certificate
            .algorithms
            .iter()
            .copied()
            .find(|a| wanted.is_none_or(|set| set.contains(a)));
        let Some(algorithm) = algorithm else {
            return self.fail(
                id,
                ErrorCode::UnsupportedAlgorithm,
                "no acceptable algorithm",
            );
        };
        let need = NeedDigest {
            seq: 1,
            certificate: certificate.clone(),
            hash: begin.hash,
            algorithm,
        };
        self.open = Some(OpenSign {
            id: id.clone(),
            hash: begin.hash,
            certificate,
            algorithm,
        });
        self.reply(id, AppMessage::NeedDigest(need));
    }

    fn finish(&mut self, id: &RequestId, digest: Base64Bytes) {
        let Some(open) = self.open.take().filter(|open| open.id == *id) else {
            return self.fail(
                id,
                ErrorCode::InvalidRequest,
                "no signature is open under this id",
            );
        };
        if digest.as_bytes().len() != open.hash.digest_len() {
            return self.fail(
                id,
                ErrorCode::InvalidRequest,
                "the digest has the wrong length",
            );
        }
        if let Some((code, message)) = self.failure_at(Stage::Confirm) {
            return self.fail(id, code, &message);
        }
        let signature = match &self.config.signature {
            Some(bytes) => Base64Bytes::new(bytes.clone()),
            None => digest,
        };
        let result = SignResult {
            certificate: open.certificate,
            hash: open.hash,
            algorithm: open.algorithm,
            signature,
        };
        self.reply(id, AppMessage::SignResult(result));
    }

    fn fail(&mut self, id: &RequestId, code: ErrorCode, message: &str) {
        let error = WireError {
            code,
            message: message.to_owned(),
            details: None,
        };
        self.reply(id, AppMessage::Error(error));
    }

    fn reply(&mut self, id: &RequestId, message: AppMessage) {
        let v = self.version.unwrap_or(1);
        let envelope = AppEnvelope {
            v,
            id: id.clone(),
            message,
        };
        // A failed write means the client is gone; the read loop ends next.
        let _ = write_frame(&mut self.output, &to_json(&envelope));
    }
}

fn app_info() -> AppInfo {
    AppInfo {
        version: "0.0.0-fake".into(),
        protocols: ProtocolRange { min: 1, max: 1 },
        os: if cfg!(windows) {
            OsName::Windows
        } else if cfg!(target_os = "macos") {
            OsName::Macos
        } else {
            OsName::Linux
        },
        arch: if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "x86_64"
        }
        .into(),
        channel: Channel::Direct,
    }
}
