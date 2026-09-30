//! An engine wired to fakes, with one-line steps for the scenarios.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use websign_core::present::caller::DesktopCaller;
use websign_core::{HashAlgorithm, SignatureAlgorithm};
use websign_devices::monitor::DeviceEvent;
use websign_host::launch::{BrowserFamily, BrowserLaunch};
use websign_host::ports::KeyReply;
use websign_host::session::Transport;
use websign_host::{Control, Engine, EngineConfig, EngineEvent};
use websign_keystores::KeystoreError;
use websign_protocol::ProtocolRange;
use websign_protocol::types::{AppInfo, Channel, OsName};
use websign_ui_model::confirm::port::RequestKey;
use websign_ui_model::confirm::{UiCommand, UiEvent};

use super::certs::{Cert, digest, snapshot};
use super::fakes::{FakeKeys, FakeLauncher, FakeOutbound, FakeUi, ManualClock, Recorded, Shared};
use super::out::Out;
use super::stores::{MemStores, State, StoreState};
use super::{ORIGIN, wire};

/// The wire spelling of a hash.
pub fn hash_str(hash: HashAlgorithm) -> &'static str {
    match hash {
        HashAlgorithm::Sha256 => "SHA-256",
        HashAlgorithm::Sha384 => "SHA-384",
        HashAlgorithm::Sha512 => "SHA-512",
    }
}

/// A Chromium launch by an extension the project allows.
pub fn allowed_launch() -> BrowserLaunch {
    let id = websign_project::chromium_extension_ids()
        .first()
        .map(|id| (*id).to_owned())
        .unwrap_or_default();
    BrowserLaunch {
        origin: format!("chrome-extension://{id}/"),
        family: BrowserFamily::Chromium,
        extension_id: id,
        parent_window: None,
    }
}

/// A Chromium launch by an extension nobody allowed.
pub fn foreign_launch() -> BrowserLaunch {
    let id = "a".repeat(32);
    BrowserLaunch {
        origin: format!("chrome-extension://{id}/"),
        family: BrowserFamily::Chromium,
        extension_id: id,
        parent_window: None,
    }
}

pub fn desktop_caller() -> DesktopCaller {
    DesktopCaller {
        executable: PathBuf::from("/opt/tools/invoicer"),
        product_name: Some("Invoicer".to_owned()),
        signer: None,
    }
}

pub fn app_info(min: u32, max: u32) -> AppInfo {
    AppInfo {
        version: "1.4.0".to_owned(),
        protocols: ProtocolRange { min, max },
        os: OsName::Linux,
        arch: "x86_64".to_owned(),
        channel: Channel::Direct,
    }
}

pub struct Harness {
    pub engine: Engine,
    pub rec: Shared,
    pub clock: ManualClock,
    pub state: State,
    /// Chain lookups [`Harness::take`] already answered.
    answered: Vec<u64>,
}

impl Harness {
    pub fn new(transport: Transport, protocols: (u32, u32)) -> Harness {
        let rec: Shared = Rc::new(RefCell::new(Recorded::default()));
        let state: State = Rc::new(RefCell::new(StoreState::default()));
        let clock = ManualClock::new();
        let ports = websign_host::engine::Ports {
            clock: Box::new(clock.clone()),
            outbound: Box::new(FakeOutbound(rec.clone())),
            ui: Box::new(FakeUi(rec.clone())),
            keys: Box::new(FakeKeys(rec.clone())),
            launcher: Box::new(FakeLauncher(rec.clone())),
            stores: Box::new(MemStores::new(&state)),
        };
        let config = EngineConfig {
            app: app_info(protocols.0, protocols.1),
            transport,
        };
        Harness {
            engine: Engine::new(config, ports),
            rec,
            clock,
            state,
            answered: Vec::new(),
        }
    }

    /// Native messaging, before `hello`.
    pub fn native() -> Harness {
        Harness::new(
            Transport::NativeMessaging {
                launch: allowed_launch(),
            },
            (1, 1),
        )
    }

    /// Native messaging after a successful `hello`, output drained.
    pub fn native_ready() -> Harness {
        let mut h = Harness::native();
        h.send(wire::hello_native("h", 1, 1));
        h.take();
        h
    }

    /// `websign connect`, before `hello`.
    pub fn desktop() -> Harness {
        Harness::new(
            Transport::Desktop {
                caller: desktop_caller(),
            },
            (1, 1),
        )
    }

    pub fn desktop_ready() -> Harness {
        let mut h = Harness::desktop();
        h.send(wire::hello_desktop("h", 1, 1));
        h.take();
        h
    }

    // ---- inputs ----

    pub fn send(&mut self, frame: Vec<u8>) -> Control {
        self.engine.handle(EngineEvent::Frame(frame))
    }

    pub fn ui(&mut self, event: UiEvent) -> Control {
        self.engine.handle(EngineEvent::Ui(event))
    }

    pub fn keys(&mut self, reply: KeyReply) -> Control {
        self.engine.handle(EngineEvent::Keys(reply))
    }

    pub fn device(&mut self, event: DeviceEvent) -> Control {
        self.engine.handle(EngineEvent::Device(event))
    }

    pub fn tick(&mut self) -> Control {
        self.engine.handle(EngineEvent::Tick)
    }

    /// Moves the clock and delivers one tick.
    pub fn pass(&mut self, seconds: u64) -> Control {
        self.clock.advance(Duration::from_secs(seconds));
        self.tick()
    }

    /// Everything sent since the last call.
    ///
    /// A sign release (`DigestPending`) waits for its issuer chain before
    /// `sign.need_digest`; the fake key store answers those lookups at once
    /// with an empty chain, as the real worker does in order, and what
    /// follows is part of the same step. `choose` lookups stay unanswered
    /// for the test to drive ([`Harness::chains`]); [`Harness::take_raw`]
    /// leaves sign lookups unanswered too.
    pub fn take(&mut self) -> Out {
        let mut out = self.take_raw();
        loop {
            let released = out
                .ui
                .iter()
                .any(|command| matches!(command, UiCommand::DigestPending { .. }));
            let tags: Vec<u64> = out
                .chain_tags()
                .into_iter()
                .filter(|tag| released && !self.answered.contains(tag))
                .collect();
            if tags.is_empty() {
                return out;
            }
            for tag in tags {
                self.answered.push(tag);
                self.keys(KeyReply::Chain {
                    tag,
                    chain: Vec::new(),
                });
            }
            let next = self.take_raw();
            out.frames.extend(next.frames);
            out.ui.extend(next.ui);
            out.keys.extend(next.keys);
            out.diagnostics.extend(next.diagnostics);
        }
    }

    /// Everything sent since the last call, with no chain answered.
    pub fn take_raw(&mut self) -> Out {
        let mut rec = self.rec.borrow_mut();
        Out {
            frames: std::mem::take(&mut rec.frames),
            ui: std::mem::take(&mut rec.ui),
            keys: std::mem::take(&mut rec.keys),
            diagnostics: std::mem::take(&mut rec.diagnostics),
        }
    }

    // ---- store seeding ----

    /// Marks `caller_key` as remembered with these certificates (newest last
    /// in the argument, newest first in the record).
    pub fn remember(&mut self, caller_key: &str, certs: &[&Cert]) {
        self.state
            .borrow_mut()
            .consent
            .push(websign_host::store::ConsentRecord {
                key: caller_key.to_owned(),
                remembered_at: 1_700_000_000,
                last_used_at: 1_700_000_000,
                certificates: certs.iter().map(|c| c.hex()).collect(),
            });
    }

    // ---- scenario steps ----

    /// `sign.begin` from `origin`, returning what it caused.
    pub fn begin(&mut self, id: &str, origin: &str, hash: HashAlgorithm) -> Out {
        self.send(wire::sign_begin(id, Some(origin), hash_str(hash)));
        self.take()
    }

    /// A window event, returning what it caused.
    pub fn ui_out(&mut self, event: UiEvent) -> Out {
        self.ui(event);
        self.take()
    }

    /// The key store answers a listing with these certificates.
    pub fn listed(&mut self, certs: &[&Cert]) -> Out {
        self.keys(KeyReply::Listed(snapshot(certs)));
        self.take()
    }

    /// A listing, with the chain lookups it causes left unanswered.
    pub fn listed_raw(&mut self, certs: &[&Cert]) -> Out {
        self.keys(KeyReply::Listed(snapshot(certs)));
        self.take_raw()
    }

    /// The key store answers every chain lookup in `asked` with `chain`.
    pub fn chains(&mut self, asked: &Out, chain: &[Vec<u8>]) -> Out {
        for tag in asked.chain_tags() {
            self.keys(KeyReply::Chain {
                tag,
                chain: chain.to_vec(),
            });
        }
        self.take()
    }

    /// A listing, then empty answers to the chain lookups it caused: what a
    /// `choose` needs before it replies.
    pub fn listed_with_chains(&mut self, certs: &[&Cert]) -> Out {
        let listed = self.listed(certs);
        let mut out = self.chains(&listed, &[]);
        out.ui.splice(0..0, listed.ui);
        out
    }

    /// The page answers `sign.need_digest` with the fixture digest.
    pub fn answer_digest(&mut self, id: &str, seq: u32, hash: HashAlgorithm) -> Out {
        self.send(wire::digest(id, seq, &digest(hash)));
        self.take()
    }

    /// Remembered `origin` with `cert`: runs `sign.begin` up to `DigestReady`
    /// and returns the window key of the request.
    pub fn drive_to_ready(&mut self, id: &str, origin: &str, cert: &Cert) -> RequestKey {
        self.remember(origin, &[cert]);
        let out = self.begin(id, origin, HashAlgorithm::Sha256);
        let key = out.opens().first().expect("window opened").key;
        let out = self.listed(&[cert]);
        assert_eq!(
            out.need_digests().len(),
            1,
            "remembered caller gets need_digest"
        );
        let out = self.answer_digest(id, 1, HashAlgorithm::Sha256);
        assert_eq!(out.digest_ready_count(), 1, "digest accepted");
        key
    }

    /// Presses Sign for `cert` (path 0) with an optional PIN.
    pub fn press_sign(&mut self, key: RequestKey, cert: &Cert, pin: Option<&str>) -> Out {
        self.ui(UiEvent::Sign {
            key,
            fingerprint: cert.fingerprint,
            via: 0,
            pin: pin.map(|p| secrecy::SecretString::from(p.to_owned())),
            remember: false,
        });
        self.take()
    }

    /// The key store signs the last `Sign` command correctly.
    pub fn signed_ok(
        &mut self,
        tag: u64,
        cert: &Cert,
        hash: HashAlgorithm,
        alg: SignatureAlgorithm,
    ) -> Out {
        self.keys(KeyReply::Signed {
            tag,
            result: Ok(cert.signature(hash, alg)),
        });
        self.take()
    }

    /// The key store fails the signature with `error`.
    pub fn signed_err(&mut self, tag: u64, error: KeystoreError) -> Out {
        self.keys(KeyReply::Signed {
            tag,
            result: Err(error),
        });
        self.take()
    }
}

/// The origin as the consent key of a web caller.
pub fn consent_key_of(origin: &str) -> String {
    origin.to_owned()
}

pub const DEFAULT_ORIGIN: &str = ORIGIN;
