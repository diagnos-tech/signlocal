//! `sign.begin` and the digest exchange that follows.

use websign_protocol::AppMessage;
use websign_protocol::ErrorCode;
use websign_protocol::messages::{ClientMessage, SignBegin, SignDigest, SignResult};
use websign_protocol::types::{Base64Bytes, Certificate, SignatureAlgorithmName};

use crate::client::{Client, SignOptions};
use crate::error::ClientError;

impl Client {
    /// `sign.begin`, answering each `sign.need_digest` with `prepare`.
    /// `prepare` may run more than once (the person switched certificate);
    /// only the last digest is signed.
    ///
    /// When `prepare` fails or returns a digest of the wrong length the
    /// request is cancelled in the app, so its window closes instead of
    /// waiting for a digest that will never come.
    pub fn sign(
        &mut self,
        options: SignOptions,
        mut prepare: impl FnMut(&Certificate, SignatureAlgorithmName) -> Result<Vec<u8>, String>,
    ) -> Result<SignResult, ClientError> {
        let hash = options.hash;
        let begin = SignBegin {
            web: None,
            hash,
            algorithms: (!options.algorithms.is_empty()).then_some(options.algorithms),
            certificate: options.certificate,
        };
        let id = self.send_request(ClientMessage::SignBegin(begin))?;

        loop {
            match self.next_message(&id)? {
                AppMessage::NeedDigest(need) => {
                    if need.hash != hash {
                        let reason =
                            ClientError::Connection("the app asked for another hash".into());
                        return Err(self.cancel(&id, reason));
                    }
                    let digest = match prepare(&need.certificate, need.algorithm) {
                        Ok(digest) => digest,
                        Err(why) => return Err(self.cancel(&id, ClientError::Prepare(why))),
                    };
                    if digest.len() != hash.digest_len() {
                        let reason = ClientError::App {
                            code: ErrorCode::InvalidRequest,
                            message: format!(
                                "prepare returned {} bytes; {hash:?} needs {}",
                                digest.len(),
                                hash.digest_len()
                            ),
                        };
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
