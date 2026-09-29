//! Turns one request frame into one reply, applying the protocol's rules.
//!
//! Validation happens here, before a key source is touched: a malformed or
//! mis-sized request must fail the same way whether or not a token is plugged
//! in.
//!
//! Spike-only safety net: a panic while serving a request (a key source or
//! library that is unfinished or crashes) becomes an `internal` error reply
//! instead of ending the process, so one bad request cannot cut a whole proof
//! session short.

use probe_core::{Fingerprint, HashAlgorithm, SignatureAlgorithm};
use std::panic::{AssertUnwindSafe, catch_unwind};

use serde_json::{Value, json};

use super::backend::{Backend, SignJob};
use super::base64;
use super::launch::BrowserLaunch;
use super::log::HostLog;
use super::protocol::{
    ErrorCode, ProtocolError, Request, SignFields, failure, parse_request, success,
};

/// A reply ready to serialize, and the one-word outcome for the log.
#[derive(Debug)]
pub struct Reply {
    pub message: Value,
    /// `ok` or a stable error code.
    pub outcome: &'static str,
}

/// The reply for a request that cannot be answered at all, such as an
/// oversized frame: there is no id to echo.
pub fn refuse(error: &ProtocolError) -> Reply {
    Reply {
        message: failure(None, error),
        outcome: error.code.as_str(),
    }
}

pub struct Handler<B> {
    launch: BrowserLaunch,
    backend: B,
    log: HostLog,
}

impl<B: Backend> Handler<B> {
    pub fn new(launch: BrowserLaunch, backend: B, log: HostLog) -> Self {
        Self {
            launch,
            backend,
            log,
        }
    }

    /// Handles one frame. Never fails: every problem becomes an error reply.
    pub fn handle(&mut self, frame: &[u8]) -> Reply {
        let parsed = parse_request(frame);
        let id = parsed.id.as_deref();
        let kind = parsed.request.as_ref().map_or("invalid", Request::kind);
        self.log.request(kind, frame.len());
        let outcome = parsed.request.and_then(|request| {
            catch_unwind(AssertUnwindSafe(|| self.dispatch(request))).unwrap_or_else(|_| {
                Err(ProtocolError::new(
                    ErrorCode::Internal,
                    "the host crashed while handling the request",
                ))
            })
        });
        match outcome {
            Ok((reply_type, fields)) => Reply {
                message: success(id, reply_type, fields),
                outcome: "ok",
            },
            Err(error) => Reply {
                message: failure(id, &error),
                outcome: error.code.as_str(),
            },
        }
    }

    fn dispatch(&mut self, request: Request) -> Result<(&'static str, Value), ProtocolError> {
        match request {
            Request::Ping(client) => {
                if let Some(client) = &client {
                    self.log.client(client);
                }
                Ok(("pong", self.pong()))
            }
            Request::List => {
                // TODO(gustavo): the product asks the user once per site before
                // listing, and keeps source warnings (which name local paths)
                // for the diagnostics window instead of sending them to pages.
                let list = self.backend.certificates()?;
                self.log
                    .certificates(list.certificates.len(), list.warnings.len());
                Ok((
                    "certificates",
                    json!({ "certificates": list.certificates, "warnings": list.warnings }),
                ))
            }
            Request::Sign(fields) => {
                let job = self.prepare_sign(fields)?;
                self.log
                    .signing(job.hash.name(), job.algorithm.name(), job.digest.len());
                let signed = self.backend.sign(&job)?;
                self.log.signed(signed.api, signed.verified);
                Ok((
                    "signature",
                    json!({
                        "signature": base64::encode(&signed.signature),
                        "api": signed.api,
                        "elapsed_ms": signed.elapsed_ms,
                        "verified": signed.verified,
                    }),
                ))
            }
        }
    }

    fn pong(&self) -> Value {
        json!({
            "app": {
                "name": env!("CARGO_PKG_NAME"),
                "version": env!("CARGO_PKG_VERSION"),
            },
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "launch": {
                "family": self.launch.family.as_str(),
                "extension_id": self.launch.extension_id,
            },
        })
    }

    /// Checks every field of a `sign` request. The digest must be exactly as
    /// long as the declared hash: anything else could be a truncated or
    /// forged "digest".
    fn prepare_sign(&self, fields: SignFields) -> Result<SignJob, ProtocolError> {
        let bad = |message: String| ProtocolError::new(ErrorCode::BadRequest, message);
        let hash: HashAlgorithm = fields.hash.parse().map_err(|e| bad(format!("{e}")))?;
        let algorithm: SignatureAlgorithm =
            fields.algorithm.parse().map_err(|e| bad(format!("{e}")))?;
        let fingerprint: Fingerprint = fields
            .fingerprint
            .parse()
            .map_err(|_| bad("fingerprint must be 64 hexadecimal digits".to_owned()))?;
        let digest = base64::decode(&fields.digest)
            .map_err(|_| bad("digest is not valid base64".to_owned()))?;
        hash.check_digest(&digest)
            .map_err(|e| ProtocolError::new(ErrorCode::DigestLength, e.to_string()))?;
        Ok(SignJob {
            fingerprint,
            hash,
            algorithm,
            digest,
            parent_window: self.launch.parent_window,
        })
    }
}

#[cfg(test)]
mod tests;
