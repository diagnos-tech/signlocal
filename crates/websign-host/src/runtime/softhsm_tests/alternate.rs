//! "Try through the token driver" against SoftHSM2 (`docs/ux.md` §4.6,
//! §5.11): the certificate's primary path is an OS store that fails, its
//! alternate the SoftHSM2 module. The OS store asked for no PIN of ours, so
//! the window must show its PIN field for the driver before signing, and
//! the PIN must reach `C_Login`.

use std::io::pipe;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use websign_core::{Fingerprint, HashAlgorithm, SignatureAlgorithm, SourceKind};
use websign_keystores::{
    FoundKey, Keystore, KeystoreError, KeystoreHub, Options, PinPrompt, SignRequest, Signature,
    open_all,
};
use websign_protocol::{AppMessage, base64};
use websign_ui_model::confirm::ConfirmModel;

use super::Token;
use super::child::{NoLauncher, config, hello, receive, send, watchdog};
use super::model_window::ModelWindow;
use crate::runtime::{Parts, WriterOutbound, serve_with};
use crate::store::MemoryStores;

/// The Windows certificate store holding the same certificate, whose
/// middleware fails every signature.
struct FailingStore {
    der: Vec<u8>,
}

impl Keystore for FailingStore {
    fn name(&self) -> String {
        "windows".to_owned()
    }

    fn list(&mut self) -> Result<Vec<FoundKey>, KeystoreError> {
        Ok(vec![FoundKey {
            cert_der: self.der.clone(),
            keystore: self.name(),
            kind: SourceKind::System,
            provider: "Microsoft Smart Card Key Storage Provider".to_owned(),
            hardware: Some(true),
            locator: "container".to_owned(),
            pin: PinPrompt::System,
            device: None,
        }])
    }

    fn sign(&mut self, _: &FoundKey, _: &SignRequest<'_>) -> Result<Signature, KeystoreError> {
        Err(KeystoreError::Native {
            api: "NCryptSignHash",
            code: 0x8010_0001,
            message: "SCARD_F_INTERNAL_ERROR".to_owned(),
        })
    }
}

#[test]
#[ignore = "needs the SoftHSM2 token of serves_a_signature_with_softhsm2"]
fn the_driver_path_gets_our_pin_after_the_store_failed() {
    let Some(token) = Token::in_child() else {
        return;
    };
    watchdog();
    let der = token.certificate("rsa-issued");
    let target = Fingerprint::of(&der);
    let (host_input, mut to_host) = pipe().unwrap();
    let (mut from_host, host_output) = pipe().unwrap();
    let signs = Arc::new(Mutex::new(Vec::new()));

    let (pin, module, store_der, window_signs) = (
        token.pin.clone(),
        token.module.clone(),
        der.clone(),
        signs.clone(),
    );
    let host = std::thread::spawn(move || {
        let parts = Parts {
            input: host_input,
            outbound: Box::new(WriterOutbound::new(host_output)),
            config: config(),
            launcher: Box::new(NoLauncher),
            stores: Box::new(MemoryStores::new()),
        };
        let make_ui = move |events| -> Box<dyn crate::ports::ConfirmUi> {
            Box::new(ModelWindow {
                events,
                target,
                pin,
                signs: window_signs,
                model: ConfirmModel::new(),
                now: Instant::now(),
                key: None,
            })
        };
        let options = Options {
            extra_modules: vec![module],
            no_known_modules: true,
            no_p11_kit: true,
            ..Options::default()
        };
        serve_with(parts, make_ui, options, move |options| {
            // The OS store first, as `open_all` opens it on Windows, so
            // the de-duplication makes it the primary path.
            let mut opened = open_all(&options);
            opened
                .keystores
                .insert(0, Box::new(FailingStore { der: store_der }));
            KeystoreHub::with_sources(opened)
        })
    });

    hello(&mut to_host, &mut from_host);
    send(
        &mut to_host,
        r#"{"v":1,"id":"s","type":"sign.begin","hash":"SHA-256","algorithms":["RSASSA-PKCS1-v1_5"]}"#,
    );
    let AppMessage::NeedDigest(need) = receive(&mut from_host, Some(1)) else {
        panic!("expected sign.need_digest");
    };
    let digest = HashAlgorithm::Sha256.digest(b"signed through the token driver");
    send(
        &mut to_host,
        &format!(
            r#"{{"v":1,"id":"s","type":"sign.digest","seq":{},"digest":"{}"}}"#,
            need.seq,
            base64::encode(&digest)
        ),
    );
    let AppMessage::SignResult(result) = receive(&mut from_host, Some(1)) else {
        panic!("expected sign.result");
    };
    websign_core::verify(
        &der,
        HashAlgorithm::Sha256,
        SignatureAlgorithm::RsaPkcs1v15,
        &digest,
        result.signature.as_bytes(),
    )
    .expect("the driver's signature verifies");
    assert_eq!(
        *signs.lock().unwrap(),
        [(0, false), (1, true)],
        "no PIN for the store's own dialog; our field's PIN for the driver"
    );

    drop(to_host);
    assert_eq!(host.join().unwrap(), 0);
}
