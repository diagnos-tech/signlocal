//! `sign.begin` and the digest exchange that follows.

use websign_protocol::AppMessage;
use websign_protocol::ErrorCode;
use websign_protocol::messages::{ClientMessage, NeedDigest, SignBegin, SignDigest, SignResult};
use websign_protocol::types::{Base64Bytes, Certificate, HashName, SignatureAlgorithmName};

use crate::client::Client;
use crate::error::ClientError;
use crate::options::{PrepareContext, SignOptions};

impl Client {
    /// `sign.begin`, answering each `sign.need_digest` with `prepare`.
    /// `prepare` may run more than once (the person switched certificate);
    /// only the last digest is signed.
    ///
    /// When `prepare` fails or returns a digest of the wrong length the
    /// request is cancelled in the app, so its window closes instead of
    /// waiting for a digest that will never come. So is a digest request for
    /// another hash or for an algorithm outside `options.algorithms`
    /// ([`ErrorCode::InvalidRequest`]), before `prepare` runs.
    pub fn sign(
        &mut self,
        options: SignOptions,
        mut prepare: impl FnMut(&Certificate, PrepareContext) -> Result<Vec<u8>, String>,
    ) -> Result<SignResult, ClientError> {
        let hash = options.hash;
        let algorithms = options.algorithm_set();
        let begin = SignBegin {
            web: None,
            hash,
            algorithms: algorithms.clone(),
            certificate: options.certificate,
        };
        let id = self.send_request(ClientMessage::SignBegin(begin))?;

        loop {
            match self.next_message(&id)? {
                AppMessage::NeedDigest(need) => {
                    if let Err(reason) = check_need(&need, hash, algorithms.as_deref()) {
                        return Err(self.cancel(&id, reason));
                    }
                    let context = PrepareContext {
                        hash,
                        algorithm: need.algorithm,
                    };
                    let digest = match prepare(&need.certificate, context) {
                        Ok(digest) => digest,
                        Err(why) => return Err(self.cancel(&id, ClientError::Prepare(why))),
                    };
                    if digest.len() != hash.digest_len() {
                        let reason = invalid(format!(
                            "prepare returned {} bytes; {hash:?} needs {}",
                            digest.len(),
                            hash.digest_len()
                        ));
                        return Err(self.cancel(&id, reason));
                    }
                    let reply = SignDigest {
                        seq: need.seq,
                        digest: Base64Bytes::new(digest),
                    };
                    self.send(&id, ClientMessage::SignDigest(reply))?;
                }
                AppMessage::SignResult(result) => return Ok(result),
                other => return Err(self.unexpected(&other)),
            }
        }
    }
}

/// Refuses a digest request that contradicts the request: the caller would
/// otherwise build signed attributes for a signature it never asked for.
/// `None` means the caller accepted any algorithm.
fn check_need(
    need: &NeedDigest,
    hash: HashName,
    algorithms: Option<&[SignatureAlgorithmName]>,
) -> Result<(), ClientError> {
    if need.hash != hash {
        return Err(invalid(format!(
            "the app asked for a {:?} digest; the request was for {hash:?}",
            need.hash
        )));
    }
    if algorithms.is_some_and(|set| !set.contains(&need.algorithm)) {
        return Err(invalid(format!(
            "the app chose {:?}, which the request did not accept",
            need.algorithm
        )));
    }
    Ok(())
}

fn invalid(message: String) -> ClientError {
    ClientError::App {
        code: ErrorCode::InvalidRequest,
        message,
    }
}
