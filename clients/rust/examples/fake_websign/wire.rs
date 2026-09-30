//! Framing and canned messages of the fake app.

use std::io::{Read, Write};

use serde_json::{Value, json};
use websign_protocol::framing::{read_frame, write_frame};

pub struct Peer<'a> {
    input: Box<dyn Read + 'a>,
    output: Box<dyn Write + 'a>,
}

impl<'a> Peer<'a> {
    pub fn new(input: impl Read + 'a, output: impl Write + 'a) -> Peer<'a> {
        Peer {
            input: Box::new(input),
            output: Box::new(output),
        }
    }

    pub fn receive(&mut self) -> Option<Value> {
        let frame = read_frame(&mut self.input).ok()??;
        serde_json::from_slice(&frame).ok()
    }

    pub fn send(&mut self, value: Value) {
        self.send_raw(value.to_string().as_bytes());
    }

    pub fn send_raw(&mut self, payload: &[u8]) {
        let _ = write_frame(&mut self.output, payload);
    }

    /// A header announcing `len` bytes, without the body.
    pub fn send_raw_header(&mut self, len: u32) {
        let _ = self.output.write_all(&len.to_ne_bytes());
        let _ = self.output.flush();
    }

    pub fn send_raw_body(&mut self, bytes: &[u8]) {
        let _ = self.output.write_all(bytes);
        let _ = self.output.flush();
    }
}

pub fn hello_reply(id: &Value, protocol: u32, range: (u32, u32)) -> Value {
    json!({"v":protocol,"id":id,"type":"hello","protocol":protocol,"app":{
        "version":"1.0.0","protocols":{"min":range.0,"max":range.1},
        "os":"linux","arch":"x86_64","channel":"direct"}})
}

pub fn certificate() -> Value {
    json!({"der":"AAEC","chain":["AwQ="],"fingerprint":"ab".repeat(32),
        "displayName":"Test Holder","issuerName":"Test CA",
        "notBefore":1,"notAfter":2,"key":{"type":"EC","curve":"P-256"},
        "algorithms":["ECDSA"],"profile":{"keyStorage":"software"}})
}
