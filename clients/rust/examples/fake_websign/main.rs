//! A stand-in for `websign connect`, used by the crate's tests.
//!
//! The behavior is chosen by the executable's file name (`fake-<scenario>`)
//! because the client only lets the caller pick the executable: the tests
//! link this binary under one name per scenario. Well-behaved scenarios
//! follow the protocol; the others misbehave in one specific way.

mod wire;

use std::io::{stdin, stdout};
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use serde_json::{Value, json};
use wire::{Peer, certificate, hello_reply};

fn scenario() -> String {
    let exe = std::env::current_exe().unwrap_or_default();
    let stem = exe.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    stem.strip_prefix("fake-").unwrap_or("ok").to_owned()
}

fn main() {
    if std::env::args().nth(1).as_deref() != Some("connect") {
        exit(64);
    }
    let name = scenario();
    let mut peer = Peer::new(stdin().lock(), stdout().lock());
    if name == "early_exit" {
        exit(1);
    }
    let hello = peer.receive().unwrap_or_else(|| exit(2));
    let id = hello["id"].clone();
    // A refusal of `hello` carries the hello's own `v` (protocol SPEC §5.1).
    let refusal = |v: Value| json!({"v":v,"id":id,"type":"error","code":"ClientOutdated","message":"too old"});
    match name.as_str() {
        "silent" => sleep(Duration::from_secs(60)),
        "hello_error" => peer.send(refusal(hello["v"].clone())),
        "hello_error_wrong_v" => peer.send(refusal(json!(hello["v"].as_u64().unwrap_or(1) + 1))),
        "newer_app" => peer.send(hello_reply(&id, 2, (2, 3))),
        "bad_protocol" => peer.send(hello_reply(&id, 3, (1, 1))),
        "garbage_hello" => peer.send_raw(b"not json"),
        _ => {
            peer.send(hello_reply(&id, 1, (1, 1)));
            if name == "deaf" {
                sleep(Duration::from_secs(60));
            }
            serve(&name, &mut peer);
        }
    }
}

fn serve(name: &str, peer: &mut Peer) {
    while let Some(request) = peer.receive() {
        let id = request["id"].clone();
        match request["type"].as_str().unwrap_or("") {
            "status" => status(name, peer, &id),
            "choose" => peer
                .send(json!({"v":1,"id":id,"type":"choose.result","certificates":[certificate()]})),
            "diagnostics.open" => peer.send(json!({"v":1,"id":id,"type":"done"})),
            "sign.begin" => sign(name, peer, &id, &request),
            _ => exit(3),
        }
    }
}

fn status(name: &str, peer: &mut Peer, id: &Value) {
    let reply = |id: &Value| {
        json!({"v":1,"id":id,"type":"status","remembered":true,
        "app":hello_reply(id, 1, (1, 1))["app"].clone()})
    };
    match name {
        "garbage_after" => peer.send_raw(b"{"),
        "unknown_field" => {
            let mut value = reply(id);
            value["extra"] = json!(1);
            peer.send(value);
        }
        "wrong_id" => peer.send(reply(&json!("zz"))),
        "oversized" => peer.send_raw_header(2 * 1024 * 1024),
        "truncated" => {
            peer.send_raw_header(100);
            peer.send_raw_body(b"abc");
            exit(0);
        }
        _ => peer.send(reply(id)),
    }
}

fn sign(name: &str, peer: &mut Peer, id: &Value, begin: &Value) {
    let hash = match name {
        "other_hash" => json!("SHA-512"),
        _ => begin["hash"].clone(),
    };
    let need = |seq: u32| {
        json!({"v":1,"id":id,"type":"sign.need_digest","seq":seq,
        "hash":hash,"algorithm":"ECDSA","certificate":certificate()})
    };
    if name == "user_cancel" {
        peer.send(json!({"v":1,"id":id,"type":"error","code":"UserCancelled","message":"no"}));
        return;
    }
    peer.send(need(1));
    match name {
        "exit_mid_sign" => exit(0),
        "cancel" | "other_hash" => match peer.receive() {
            Some(cancel) if cancel["type"] == "cancel" => peer
                .send(json!({"v":1,"id":id,"type":"error","code":"Aborted","message":"cancelled"})),
            _ => exit(4),
        },
        "switch" => {
            peer.send(need(2));
            let _stale = peer.receive();
            finish(peer, id, begin);
        }
        _ => finish(peer, id, begin),
    }
}

/// Answers with the last digest as the "signature", so tests can see which
/// digest the client signed.
fn finish(peer: &mut Peer, id: &Value, begin: &Value) {
    let Some(digest) = peer.receive() else {
        exit(5)
    };
    peer.send(
        json!({"v":1,"id":id,"type":"sign.result","hash":begin["hash"],
        "algorithm":"ECDSA","certificate":certificate(),"signature":digest["digest"]}),
    );
}
