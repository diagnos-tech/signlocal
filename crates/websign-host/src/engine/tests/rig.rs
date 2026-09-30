//! A host process over fake ports, driven with JSON frames.

use std::sync::Arc;
use std::time::Duration;

use websign_core::Fingerprint;
use websign_core::present::caller::DesktopCaller;
use websign_protocol::types::{AppInfo, Channel, OsName};
use websign_protocol::{AppMessage, ProtocolRange, base64};
use websign_ui_model::confirm::port::{OpenRequest, RequestKey};
use websign_ui_model::confirm::{UiCommand, UiEvent};

use crate::engine::{Control, Engine, EngineConfig, EngineEvent};
use crate::launch::BrowserLaunch;
use crate::ports::{KeyCommand, KeyReply, KeySnapshot};
use crate::session::Transport;
use crate::testing::{Handles, fake_ports, fixture};

pub const WEB: &str = r#""web":{"origin":"https://app.example","topOrigin":"https://app.example"}"#;

pub struct Rig {
    pub engine: Engine,
    pub h: Handles,
}

pub fn program_transport() -> Transport {
    Transport::Desktop {
        caller: DesktopCaller {
            executable: "/usr/bin/signer".into(),
            product_name: Some("Signer".into()),
            signer: None,
        },
    }
}

pub fn app_info() -> AppInfo {
    AppInfo {
        version: "1.0.0".into(),
        protocols: ProtocolRange { min: 1, max: 1 },
        os: OsName::Linux,
        arch: "x86_64".into(),
        channel: Channel::Direct,
    }
}

impl Rig {
    pub fn new(transport: Transport) -> Rig {
        let (ports, h) = fake_ports();
        let engine = Engine::new(
            EngineConfig {
                app: app_info(),
                transport,
            },
            ports,
        );
        Rig { engine, h }
    }

    /// Native messaging, `hello` already exchanged.
    pub fn browser() -> Rig {
        let mut rig = Rig::new(Transport::NativeMessaging {
            launch: BrowserLaunch::manual(),
        });
        rig.hello_browser();
        rig
    }

    /// A desktop program, `hello` already exchanged.
    pub fn program() -> Rig {
        let mut rig = Rig::new(program_transport());
        rig.frame(r#""id":"h","type":"hello","client":{"name":"t","version":"1"},"protocols":{"min":1,"max":1}"#);
        rig.h.outbound.take();
        rig
    }

    pub fn hello_browser(&mut self) {
        self.frame(
            r#""id":"h","type":"hello","client":{"name":"ext","version":"2"},"protocols":{"min":1,"max":1},"browser":{"name":"chrome","version":"130","reason":"page"}"#,
        );
        self.h.outbound.take();
    }

    /// Sends `{"v":1, <body>}`.
    pub fn frame(&mut self, body: &str) -> Control {
        self.engine.handle(EngineEvent::Frame(
            format!(r#"{{"v":1,{body}}}"#).into_bytes(),
        ))
    }

    pub fn sign_begin(&mut self, id: &str) -> Control {
        self.frame(&format!(
            r#""id":"{id}","type":"sign.begin",{WEB},"hash":"SHA-256""#
        ))
    }

    pub fn digest(&mut self, id: &str, seq: u32, digest: &[u8]) -> Control {
        let text = base64::encode(digest);
        self.frame(&format!(
            r#""id":"{id}","type":"sign.digest","seq":{seq},"digest":"{text}""#
        ))
    }

    pub fn ui(&mut self, event: UiEvent) {
        self.engine.handle(EngineEvent::Ui(event));
    }

    pub fn listed(&mut self, snapshot: KeySnapshot) {
        self.engine
            .handle(EngineEvent::Keys(KeyReply::Listed(Arc::new(snapshot))));
    }

    pub fn keys(&mut self, reply: KeyReply) {
        self.engine.handle(EngineEvent::Keys(reply));
    }

    /// Answers every chain lookup asked so far with an empty chain; other
    /// key commands are dropped.
    pub fn answer_chains(&mut self) {
        for command in self.h.keys.take() {
            if let KeyCommand::Chain { tag, .. } = command {
                self.keys(KeyReply::Chain {
                    tag,
                    chain: Vec::new(),
                });
            }
        }
    }

    pub fn tick_after(&mut self, by: Duration) -> Control {
        self.h.clock.advance(by);
        self.engine.handle(EngineEvent::Tick)
    }

    pub fn messages(&self) -> Vec<AppMessage> {
        self.h
            .outbound
            .take()
            .into_iter()
            .map(|e| e.message)
            .collect()
    }

    /// The `Open` command among `commands`, if any.
    pub fn opened(commands: &[UiCommand]) -> Option<OpenRequest> {
        commands.iter().find_map(|command| match command {
            UiCommand::Open(open) => Some(open.clone()),
            _ => None,
        })
    }

    /// Runs one `sign.begin` up to the digest being ready and returns the
    /// request key: list, continue (D11) unless the certificate is
    /// consented, chain, digest.
    pub fn until_ready(&mut self, id: &str) -> RequestKey {
        self.sign_begin(id);
        let key = Rig::opened(&self.h.ui.take()).expect("window opened").key;
        self.listed(fixture::snapshot());
        let ui = self.h.ui.take();
        assert!(
            ui.iter()
                .any(|c| matches!(c, UiCommand::Certificates { .. }))
        );
        let released = ui
            .iter()
            .any(|c| matches!(c, UiCommand::DigestPending { .. }));
        if !released {
            self.ui(UiEvent::Continue {
                key,
                fingerprint: fixture::fingerprint(),
            });
        }
        self.answer_chains();
        self.digest(id, 1, &fixture::DIGEST);
        key
    }

    /// The tag of the `Sign` command among the key commands sent.
    pub fn sign_tag(&self) -> u64 {
        self.h
            .keys
            .take()
            .into_iter()
            .find_map(|command| match command {
                KeyCommand::Sign { tag, .. } => Some(tag),
                _ => None,
            })
            .expect("a Sign command")
    }

    pub fn press_sign(&mut self, key: RequestKey, remember: bool) {
        self.ui(UiEvent::Sign {
            key,
            fingerprint: fixture::fingerprint(),
            via: 0,
            pin: None,
            remember,
        });
    }
}

pub fn fp() -> Fingerprint {
    fixture::fingerprint()
}
