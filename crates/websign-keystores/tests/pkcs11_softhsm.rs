//! The PKCS#11 adapter against a real module: SoftHSM2 with RSA, NIST and
//! Brainpool EC keys, a `CKA_ALWAYS_AUTHENTICATE` key and a CA chain.
//!
//! `softhsm_token` creates the token and re-runs this binary for the
//! `on_token::` tests (see `support/softhsm.rs`).

mod support;

#[test]
fn softhsm_token() {
    support::softhsm::run_children("on_token::");
}

mod on_token {
    use secrecy::SecretString;
    use websign_core::{Fingerprint, HashAlgorithm, SignatureAlgorithm, SourceKind};
    use websign_keystores::contract::{self, Fixture};
    use websign_keystores::{
        DeviceLink, FoundKey, Keystore, KeystoreError, PinPrompt, SignRequest,
    };

    use crate::support::softhsm::SoftHsm;

    fn token() -> Option<SoftHsm> {
        let token = SoftHsm::in_child();
        if token.is_none() {
            eprintln!("not started by softhsm_token: skipped");
        }
        token
    }

    fn listed(keystore: &mut dyn Keystore, der: &[u8]) -> FoundKey {
        keystore
            .list()
            .expect("list")
            .into_iter()
            .find(|key| key.cert_der == der)
            .expect("key is listed")
    }

    fn sign(
        keystore: &mut dyn Keystore,
        key: &FoundKey,
        pin: Option<&str>,
    ) -> Result<Vec<u8>, KeystoreError> {
        let digest = HashAlgorithm::Sha256.digest(b"on token");
        let pin = pin.map(|pin| SecretString::from(pin.to_owned()));
        let request = SignRequest {
            hash: HashAlgorithm::Sha256,
            algorithm: SignatureAlgorithm::RsaPkcs1v15,
            digest: &digest,
            pin: pin.as_ref(),
            parent_window: None,
        };
        keystore
            .sign(key, &request)
            .map(|signature| signature.bytes)
    }

    #[test]
    #[ignore = "needs the SoftHSM2 token of softhsm_token"]
    fn meets_the_keystore_contract() {
        let Some(token) = token() else { return };
        let expected = token
            .keys()
            .iter()
            .map(|(_, der)| Fingerprint::of(der).to_hex())
            .collect();
        let fixture = Fixture {
            expected,
            pin: Some(token.pin.clone()),
            wrong_pin: Some("000000".to_owned()),
            private_text: vec![token.label.clone(), token.serial.clone()],
        };
        let mut keystore = token.keystore();
        let violations = contract::run(keystore.as_mut(), &fixture);
        assert!(violations.is_empty(), "{violations:#?}");
    }

    #[test]
    #[ignore = "needs the SoftHSM2 token of softhsm_token"]
    fn lists_every_key_once_as_software_without_ca_certificates() {
        let Some(token) = token() else { return };
        let mut keystore = token.keystore();
        let listed = keystore.list().expect("list");
        let mut expected: Vec<Vec<u8>> = token.keys().into_iter().map(|(_, der)| der).collect();
        let mut found: Vec<Vec<u8>> = listed.iter().map(|key| key.cert_der.clone()).collect();
        expected.sort();
        found.sort();
        assert_eq!(found, expected, "CA certificates must not be listed");
        for key in &listed {
            assert_eq!(key.kind, SourceKind::Pkcs11);
            assert_eq!(key.hardware, Some(false));
            assert_eq!(
                key.pin,
                PinPrompt::App {
                    protected_path: false
                }
            );
            assert_eq!(
                key.device,
                Some(DeviceLink::Pkcs11Token {
                    model: "SoftHSM v2".to_owned(),
                    manufacturer: "SoftHSM project".to_owned(),
                })
            );
            assert!(key.provider.contains("libsofthsm2"), "{}", key.provider);
        }
    }

    #[test]
    #[ignore = "needs the SoftHSM2 token of softhsm_token"]
    fn pin_state_follows_the_session() {
        let Some(token) = token() else { return };
        let mut keystore = token.keystore();
        let key = listed(keystore.as_mut(), &token.key("rsa-2048"));
        let before = keystore.pin_state(&key).expect("PIN state");
        assert_eq!(before.length, Some((4, 255)));
        assert!(!before.unlocked && !before.locked && !before.final_try);
        assert!(matches!(
            sign(keystore.as_mut(), &key, None),
            Err(KeystoreError::PinRequired)
        ));

        sign(keystore.as_mut(), &key, Some(&token.pin)).expect("signs with the PIN");
        assert!(keystore.pin_state(&key).expect("PIN state").unlocked);
        let other = listed(keystore.as_mut(), &token.key("rsa-issued"));
        sign(keystore.as_mut(), &other, None).expect("the unlocked token signs any of its keys");

        let always = listed(keystore.as_mut(), &token.key("rsa-aa"));
        let state = keystore.pin_state(&always).expect("PIN state");
        assert!(state.unlocked && state.always_authenticate);
        assert!(matches!(
            sign(keystore.as_mut(), &always, None),
            Err(KeystoreError::PinRequired)
        ));
        assert!(matches!(
            sign(keystore.as_mut(), &always, Some("000000")),
            Err(KeystoreError::WrongPin)
        ));
        sign(keystore.as_mut(), &always, Some(&token.pin)).expect("re-authenticated signature");
        sign(keystore.as_mut(), &other, None)
            .expect("still unlocked after failed re-authentication");

        keystore.end_sessions();
        assert!(!keystore.pin_state(&key).expect("PIN state").unlocked);
    }

    #[test]
    #[ignore = "needs the SoftHSM2 token of softhsm_token"]
    fn chain_comes_from_the_ca_certificates_on_the_token() {
        let Some(token) = token() else { return };
        let mut keystore = token.keystore();
        let issued = listed(keystore.as_mut(), &token.key("rsa-issued"));
        assert_eq!(
            keystore.chain(&issued),
            [token.ca("intermediate"), token.ca("root")]
        );
        let self_signed = listed(keystore.as_mut(), &token.key("ec-p256"));
        assert!(keystore.chain(&self_signed).is_empty());
    }
}
