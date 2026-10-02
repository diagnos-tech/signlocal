use serde_json::{Value, json};

use super::*;
use crate::nm::backend::{CertificateList, SignJob, SignedDigest};
use crate::nm::framing::{MAX_INCOMING, read_frame, write_frame};
use crate::nm::launch::BrowserFamily;

struct NoKeys;

impl Backend for NoKeys {
    fn certificates(&mut self) -> Result<CertificateList, ProtocolError> {
        Ok(CertificateList::default())
    }

    fn sign(&mut self, _job: &SignJob) -> Result<SignedDigest, ProtocolError> {
        Err(ProtocolError::new(ErrorCode::NotFound, "none"))
    }
}

fn run(input: Vec<u8>) -> (Ended, Vec<Value>) {
    let launch = BrowserLaunch {
        origin: "manual".into(),
        family: BrowserFamily::Manual,
        extension_id: "manual".into(),
        parent_window: None,
    };
    let log = HostLog::disabled();
    let mut handler = Handler::new(launch, NoKeys, log.clone());
    let mut output = Vec::new();
    let ended = serve(&mut input.as_slice(), &mut output, &mut handler, &log);
    let mut replies = Vec::new();
    let mut reader = output.as_slice();
    while let Some(frame) = read_frame(&mut reader).unwrap() {
        replies.push(serde_json::from_slice(&frame).unwrap());
    }
    (ended, replies)
}

fn framed(messages: &[Value]) -> Vec<u8> {
    let mut stream = Vec::new();
    for message in messages {
        write_frame(&mut stream, message.to_string().as_bytes()).unwrap();
    }
    stream
}

#[test]
fn answers_every_message_in_order_on_one_connection() {
    let input = framed(&[
        json!({ "v": 1, "id": "1", "type": "ping" }),
        json!({ "v": 1, "id": "2", "type": "list" }),
        json!({ "v": 1, "id": "3", "type": "ping" }),
    ]);
    let (ended, replies) = run(input);
    assert_eq!(ended, Ended::Disconnected);
    let ids: Vec<_> = replies.iter().map(|r| r["id"].as_str().unwrap()).collect();
    assert_eq!(ids, ["1", "2", "3"]);
    assert!(replies.iter().all(|r| r["ok"] == true));
}

#[test]
fn clean_eof_ends_the_loop_quietly() {
    let (ended, replies) = run(Vec::new());
    assert_eq!(ended, Ended::Disconnected);
    assert!(replies.is_empty());
}

#[test]
fn invalid_json_is_answered_and_the_connection_survives() {
    let mut input = Vec::new();
    write_frame(&mut input, b"{ nope").unwrap();
    input.extend(framed(&[json!({ "v": 1, "id": "ok", "type": "ping" })]));
    let (ended, replies) = run(input);
    assert_eq!(ended, Ended::Disconnected);
    assert_eq!(replies[0]["error"]["code"], "bad_request");
    assert_eq!(replies[1]["id"], "ok");
}

#[test]
fn a_half_written_header_ends_the_loop_as_a_failure() {
    let mut input = framed(&[json!({ "v": 1, "id": "1", "type": "ping" })]);
    input.extend_from_slice(&[7, 0]);
    let (ended, replies) = run(input);
    assert_eq!(ended, Ended::Failed);
    assert_eq!(replies.len(), 1);
}

#[test]
fn an_oversized_request_is_refused_once_and_the_loop_stops() {
    let mut input = ((MAX_INCOMING + 1) as u32).to_ne_bytes().to_vec();
    input.extend_from_slice(b"junk that must never be parsed");
    let (ended, replies) = run(input);
    assert_eq!(ended, Ended::Failed);
    assert_eq!(replies.len(), 1);
    assert_eq!(replies[0]["error"]["code"], "bad_request");
}
