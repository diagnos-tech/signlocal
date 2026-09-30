//! `parse(to_json(x)) == x` for every message type over generated values (SPEC §5).

mod common;

use common::*;
use serde_json::{Value, json};
use websign_protocol::{
    AppEnvelope, ClientEnvelope, parse_app_message, parse_client_message, to_json,
};

/// xorshift64*: deterministic, no dependency.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len())]
    }
    fn chance(&mut self) -> bool {
        self.next() & 1 == 1
    }
    fn bytes_b64(&mut self, max: usize) -> String {
        let len = self.below(max + 1);
        let bytes: Vec<u8> = (0..len).map(|_| self.next() as u8).collect();
        websign_protocol::base64::encode(&bytes)
    }
    fn text(&mut self) -> String {
        let alphabet = [
            "a", "Z", "0", " ", "é", "日", "\"", "\\", "\n", "/", "😀", "<",
        ];
        (0..self.below(12))
            .map(|_| alphabet[self.below(alphabet.len())])
            .collect()
    }
    fn id(&mut self) -> String {
        let alphabet: Vec<char> = "abcXYZ019._:-".chars().collect();
        (0..1 + self.below(64))
            .map(|_| alphabet[self.below(alphabet.len())])
            .collect()
    }
    fn fingerprint(&mut self) -> String {
        (0..64)
            .map(|_| char::from_digit(self.below(16) as u32, 16).unwrap())
            .collect()
    }
}

/// One to three names: an empty list is refused (SPEC §5 step 7).
fn algorithms(rng: &mut Rng) -> Value {
    let all = ["ECDSA", "RSASSA-PKCS1-v1_5", "RSASSA-PSS"];
    json!(
        (0..1 + rng.below(3))
            .map(|_| rng.pick(&all))
            .collect::<Vec<_>>()
    )
}

fn web_context(rng: &mut Rng) -> Value {
    json!({ "origin": format!("https://{}.example", rng.id().to_lowercase().replace([':', '_', '.'], "a")),
        "topOrigin": "https://top.example" })
}

fn generated_certificate(rng: &mut Rng) -> Value {
    let key = if rng.chance() {
        json!({ "type": "RSA", "bits": rng.pick(&["2048", "3072", "4096"]).parse::<u32>().unwrap() })
    } else {
        let curves = [
            "P-256",
            "P-384",
            "P-521",
            "brainpoolP256r1",
            "brainpoolP384r1",
            "brainpoolP512r1",
        ];
        json!({ "type": "EC", "curve": rng.pick(&curves) })
    };
    let mut profile = json!({ "keyStorage": rng.pick(&["hardware", "software", "unknown"]) });
    if rng.chance() {
        profile["icpBrasil"] = json!(rng.pick(&["A1", "A3", "A4", "S3"]));
    }
    if rng.chance() {
        profile["eidas"] = json!({
            "qualified": rng.chance(), "qscd": rng.chance(),
            "types": [rng.pick(&["esign", "eseal", "web"])]
        });
    }
    json!({
        "der": rng.bytes_b64(40),
        "chain": (0..rng.below(4)).map(|_| rng.bytes_b64(20)).collect::<Vec<_>>(),
        "fingerprint": rng.fingerprint(),
        "displayName": rng.text(),
        "issuerName": rng.text(),
        "notBefore": rng.next() as i64 >> 20,
        "notAfter": rng.next() >> 20,
        "key": key,
        "algorithms": algorithms(rng),
        "profile": profile,
    })
}

fn generated_app_info(rng: &mut Rng) -> Value {
    json!({
        "version": rng.text(),
        "protocols": { "min": 1, "max": 1 + rng.below(3) },
        "os": rng.pick(&["windows", "macos", "linux"]),
        "arch": rng.pick(&["x86_64", "aarch64"]),
        "channel": rng.pick(&["direct", "store"]),
    })
}

fn generated_client(rng: &mut Rng) -> Value {
    let id = rng.id();
    let mut frame = match rng.below(7) {
        0 => {
            let mut hello = json!({
                "type": "hello",
                "client": { "name": rng.text(), "version": rng.text() },
                "protocols": { "min": 1, "max": 1 },
            });
            if rng.chance() {
                hello["browser"] = json!({
                    "name": rng.pick(&["chrome", "edge", "firefox", "safari", "other"]),
                    "version": rng.text(),
                    "reason": rng.pick(&["startup", "installed", "page", "popup"]),
                });
            }
            hello
        }
        1 => json!({ "type": "status" }),
        2 => {
            let mut choose = json!({ "type": "choose" });
            if rng.chance() {
                choose["filter"] = json!({ "algorithms": algorithms(rng) });
            }
            choose
        }
        3 => {
            let mut begin = json!({ "type": "sign.begin", "hash": rng.pick(&["SHA-256", "SHA-384", "SHA-512"]) });
            if rng.chance() {
                begin["algorithms"] = algorithms(rng);
            }
            if rng.chance() {
                begin["certificate"] = json!(rng.fingerprint());
            }
            begin
        }
        4 => {
            json!({ "type": "sign.digest", "seq": rng.next() as u32, "digest": rng.bytes_b64(64) })
        }
        5 => json!({ "type": "cancel" }),
        _ => {
            let mut open = json!({ "type": "diagnostics.open" });
            if rng.chance() {
                open["tab"] = json!(rng.pick(&["browsers", "devices", "certificates", "help"]));
            }
            open
        }
    };
    if matches!(
        frame["type"].as_str(),
        Some("status" | "choose" | "sign.begin")
    ) && rng.chance()
    {
        frame["web"] = web_context(rng);
    }
    frame["v"] = json!(1);
    frame["id"] = json!(id);
    frame
}

fn generated_app(rng: &mut Rng) -> Value {
    let id = rng.id();
    let mut frame = match rng.below(7) {
        0 => json!({ "type": "hello", "protocol": 1, "app": generated_app_info(rng) }),
        1 => {
            json!({ "type": "status", "app": generated_app_info(rng), "remembered": rng.chance() })
        }
        2 => {
            json!({ "type": "choose.result", "certificates": [generated_certificate(rng), generated_certificate(rng)] })
        }
        3 => json!({
            "type": "sign.need_digest", "seq": rng.next() as u32, "certificate": generated_certificate(rng),
            "hash": rng.pick(&["SHA-256", "SHA-384", "SHA-512"]),
            "algorithm": rng.pick(&["ECDSA", "RSASSA-PKCS1-v1_5", "RSASSA-PSS"]),
        }),
        4 => json!({
            "type": "sign.result", "certificate": generated_certificate(rng),
            "hash": rng.pick(&["SHA-256", "SHA-384", "SHA-512"]),
            "algorithm": rng.pick(&["ECDSA", "RSASSA-PKCS1-v1_5", "RSASSA-PSS"]),
            "signature": rng.bytes_b64(300),
        }),
        5 => json!({ "type": "done" }),
        _ => {
            let codes = websign_protocol::ErrorCode::ALL;
            let mut error = json!({ "type": "error", "code": codes[rng.below(codes.len())].as_str(), "message": rng.text() });
            if rng.chance() {
                error["details"] = json!({ "native": rng.text() });
            }
            error
        }
    };
    frame["v"] = json!(1);
    frame["id"] = json!(id);
    frame
}

#[test]
fn client_messages_round_trip() {
    let mut rng = Rng(0x1234_5678_9ABC_DEF1);
    for i in 0..1500 {
        let frame = generated_client(&mut rng);
        let negotiated = if frame["type"] == "hello" {
            None
        } else {
            Some(1)
        };
        let envelope =
            parse_client(&frame, negotiated).unwrap_or_else(|e| panic!("#{i} {frame}: {e:?}"));
        let bytes = to_json(&envelope);
        let again =
            parse_client_message(&bytes, negotiated).unwrap_or_else(|e| panic!("#{i}: {e:?}"));
        assert_eq!(again, envelope, "#{i}");
        assert_eq!(from_bytes(&bytes), frame, "#{i}");
        assert_eq!(
            to_json(&again),
            bytes,
            "#{i}: serialization must be deterministic"
        );
    }
}

#[test]
fn app_messages_round_trip() {
    let mut rng = Rng(0x0FED_CBA9_8765_4321);
    for i in 0..1500 {
        let frame = generated_app(&mut rng);
        let negotiated = if frame["type"] == "hello" {
            None
        } else {
            Some(1)
        };
        let envelope =
            parse_app(&frame, negotiated).unwrap_or_else(|e| panic!("#{i} {frame}: {e:?}"));
        let bytes = to_json(&envelope);
        let again = parse_app_message(&bytes, negotiated).unwrap_or_else(|e| panic!("#{i}: {e:?}"));
        assert_eq!(again, envelope, "#{i}");
        assert_eq!(from_bytes(&bytes), frame, "#{i}");
    }
}

#[test]
fn round_trip_holds_through_frames() {
    use websign_protocol::framing::{read_frame, write_frame};
    let mut rng = Rng(42);
    let originals: Vec<ClientEnvelope> = (0..50)
        .map(|_| generated_client(&mut rng))
        .filter(|frame| frame["type"] != "hello")
        .map(|frame| parse_client(&frame, Some(1)).unwrap())
        .collect();
    let mut stream = Vec::new();
    for envelope in &originals {
        write_frame(&mut stream, &to_json(envelope)).unwrap();
    }
    let mut reader = stream.as_slice();
    for envelope in &originals {
        let frame = read_frame(&mut reader).unwrap().unwrap();
        assert_eq!(&parse_client_message(&frame, Some(1)).unwrap(), envelope);
    }
    assert!(read_frame(&mut reader).unwrap().is_none());
}

#[test]
fn unicode_and_escapes_survive() {
    let mut frame = hello();
    frame["client"]["name"] = json!("cliente \"ação\" \\ 日本 \u{1F600} \n\t");
    let envelope = parse_client(&frame, None).unwrap();
    assert_eq!(from_bytes(&to_json(&envelope)), frame);
}

#[test]
fn compact_json_and_pretty_json_parse_identically() {
    let pretty = serde_json::to_vec_pretty(&sign_begin()).unwrap();
    let from_pretty = parse_client_message(&pretty, Some(1)).unwrap();
    let from_compact = parse_client(&sign_begin(), Some(1)).unwrap();
    assert_eq!(from_pretty, from_compact);
}

#[test]
fn key_order_does_not_matter() {
    let reordered = br#"{"hash":"SHA-256","type":"sign.begin","id":"3","web":{"topOrigin":"https://app.example","origin":"https://app.example"},"v":1}"#;
    assert_eq!(
        parse_client_message(reordered, Some(1)).unwrap(),
        parse_client(&sign_begin(), Some(1)).unwrap()
    );
}

#[test]
fn envelope_types_are_directional_values() {
    let client: ClientEnvelope = parse_client(&sign_digest(), Some(1)).unwrap();
    let app: AppEnvelope = parse_app(&need_digest(), Some(1)).unwrap();
    assert_eq!(client.id, app.id);
    assert_eq!(client.v, 1);
}
