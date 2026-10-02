//! The part of the SoftHSM2 run that sees the token: a client on pipes talks
//! to `serve` with the real key worker, and a scripted window presses the
//! buttons a person would.

use std::io::{PipeReader, PipeWriter, pipe};
use std::time::Duration;

use secrecy::SecretString;
use websign_core::present::caller::DesktopCaller;
use websign_core::{Fingerprint, HashAlgorithm, SignatureAlgorithm};
use websign_keystores::Options;
use websign_protocol::framing::{read_frame, write_frame};
use websign_protocol::types::{AppInfo, Channel, OsName};
use websign_protocol::{AppMessage, ProtocolRange, base64, parse_app_message};
use websign_ui_model::confirm::{UiCommand, UiEvent};

use super::Token;
use crate::engine::EngineConfig;
use crate::ports::{ConfirmUi, Launcher};
use crate::runtime::{EventSender, WriterOutbound, serve};
use crate::session::Transport;
use crate::store::MemoryStores;
use websign_protocol::messages::DiagnosticsTab;

/// A window that continues with the chosen certificate and signs with the
/// token's PIN, as soon as it may.
struct ScriptedWindow {
    events: EventSender,
    target: Fingerprint,
    pin: String,
}

impl ConfirmUi for ScriptedWindow {
    fn command(&mut self, command: UiCommand) {
        let event = match command {
            UiCommand::Certificates { key, .. } => UiEvent::Continue {
                key,
                fingerprint: self.target,
            },
            UiCommand::DigestReady {
                key, fingerprint, ..
            } => UiEvent::Sign {
                key,
                fingerprint,
                via: 0,
                pin: Some(SecretString::from(self.pin.clone())),
                remember: false,
            },
            UiCommand::Failed { failure, .. } => panic!("the window was told of {failure:?}"),
            _ => return,
        };
        self.events
            .send(crate::engine::EngineEvent::Ui(event))
            .unwrap();
    }

    fn parent_window(&self) -> Option<isize> {
        None
    }
}

pub(super) struct NoLauncher;

impl Launcher for NoLauncher {
    fn open_diagnostics(&mut self, _: Option<DiagnosticsTab>) -> Result<(), String> {
        Err("not used".to_owned())
    }
}

/// A desktop client, so no browser rules apply.
pub(super) fn config() -> EngineConfig {
    EngineConfig {
        app: AppInfo {
            version: "1.0.0".into(),
            protocols: ProtocolRange { min: 1, max: 1 },
            os: OsName::Linux,
            arch: "x86_64".into(),
            channel: Channel::Direct,
        },
        transport: Transport::Desktop {
            caller: DesktopCaller {
                executable: "/usr/bin/test-client".into(),
                product_name: None,
                signer: None,
            },
        },
    }
}

/// `hello`, answered.
pub(super) fn hello(to_host: &mut PipeWriter, from_host: &mut PipeReader) {
    send(
        to_host,
        r#"{"v":1,"id":"h","type":"hello","client":{"name":"test","version":"1"},"protocols":{"min":1,"max":1}}"#,
    );
    assert!(matches!(receive(from_host, None), AppMessage::Hello(_)));
}

/// Fails the whole run if the host hangs.
pub(super) fn watchdog() {
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(120));
        eprintln!("timed out waiting for the host");
        std::process::exit(2);
    });
}

pub(super) fn send(writer: &mut PipeWriter, json: &str) {
    write_frame(writer, json.as_bytes()).unwrap();
}

pub(super) fn receive(reader: &mut PipeReader, negotiated: Option<u32>) -> AppMessage {
    let frame = read_frame(reader).unwrap().expect("the host closed early");
    parse_app_message(&frame, negotiated).unwrap().message
}

#[test]
#[ignore = "needs the SoftHSM2 token of serves_a_signature_with_softhsm2"]
fn signs_over_stdio_framing_through_the_key_worker() {
    let Some(token) = Token::in_child() else {
        return;
    };
    // A hung host must fail the run, not hang it.
    watchdog();

    let der = token.certificate("rsa-issued");
    let target = Fingerprint::of(&der);
    let (host_input, mut to_host) = pipe().unwrap();
    let (mut from_host, host_output) = pipe().unwrap();

    let pin = token.pin.clone();
    let module = token.module.clone();
    let host = std::thread::spawn(move || {
        serve(
            host_input,
            Box::new(WriterOutbound::new(host_output)),
            config(),
            move |events| {
                Box::new(ScriptedWindow {
                    events,
                    target,
                    pin,
                })
            },
            Box::new(NoLauncher),
            Box::new(MemoryStores::new()),
            Options {
                extra_modules: vec![module],
                no_known_modules: true,
                no_p11_kit: true,
                ..Options::default()
            },
        )
    });

    hello(&mut to_host, &mut from_host);

    send(
        &mut to_host,
        r#"{"v":1,"id":"s","type":"sign.begin","hash":"SHA-384","algorithms":["RSASSA-PSS"]}"#,
    );
    let AppMessage::NeedDigest(need) = receive(&mut from_host, Some(1)) else {
        panic!("expected sign.need_digest");
    };
    assert_eq!(need.seq, 1);
    assert_eq!(need.certificate.fingerprint.as_str(), target.to_hex());
    assert_eq!(need.certificate.der.as_bytes(), der);

    let digest = HashAlgorithm::Sha384.digest(b"the document, hashed by the caller");
    send(
        &mut to_host,
        &format!(
            r#"{{"v":1,"id":"s","type":"sign.digest","seq":1,"digest":"{}"}}"#,
            base64::encode(&digest)
        ),
    );
    let AppMessage::SignResult(result) = receive(&mut from_host, Some(1)) else {
        panic!("expected sign.result");
    };
    websign_core::verify(
        &der,
        HashAlgorithm::Sha384,
        SignatureAlgorithm::RsaPss,
        &digest,
        result.signature.as_bytes(),
    )
    .expect("the signature verifies against the certificate");
    assert_eq!(
        result.certificate.chain.len(),
        2,
        "issuer chain from the token"
    );

    drop(to_host);
    assert_eq!(
        host.join().unwrap(),
        0,
        "the host ends when the client hangs up"
    );
}
