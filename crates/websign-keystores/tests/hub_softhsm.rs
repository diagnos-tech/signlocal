//! `KeystoreHub` over a real module: SoftHSM2 opened through `Options`,
//! routed by fingerprint, with the D5 session surviving a relisting.

mod support;

#[test]
fn softhsm_hub() {
    support::softhsm::run_children("through_the_hub::");
}

mod through_the_hub {
    use secrecy::SecretString;
    use websign_core::{Fingerprint, HashAlgorithm, SignatureAlgorithm};
    use websign_keystores::{KeyRef, KeystoreError, KeystoreHub, SignRequest};

    use crate::support::softhsm::SoftHsm;

    #[test]
    #[ignore = "needs the SoftHSM2 token of softhsm_hub"]
    fn the_hub_lists_signs_and_routes_by_fingerprint() {
        let Some(token) = SoftHsm::in_child() else {
            return;
        };
        let mut hub = KeystoreHub::new(token.options());
        let inventory = hub.inventory();
        assert!(
            inventory.warnings().is_empty(),
            "{:?}",
            inventory.warnings()
        );
        assert_eq!(inventory.groups().len(), token.keys().len());

        let der = token.key("rsa-issued");
        let key = KeyRef {
            fingerprint: Fingerprint::of(&der),
            path: 0,
        };
        let digest = HashAlgorithm::Sha384.digest(b"through the hub");
        let pin = SecretString::from(token.pin.clone());
        let request = SignRequest {
            hash: HashAlgorithm::Sha384,
            algorithm: SignatureAlgorithm::RsaPss,
            digest: &digest,
            pin: Some(&pin),
            parent_window: None,
        };
        let signature = hub.sign(key, &request).expect("signs");
        websign_core::verify(
            &der,
            HashAlgorithm::Sha384,
            SignatureAlgorithm::RsaPss,
            &digest,
            &signature.bytes,
        )
        .expect("verifies");
        assert_eq!(hub.chain(key).len(), 2);
        assert!(hub.pin_state(key).expect("PIN state").unlocked);

        hub.invalidate();
        assert!(
            hub.pin_state(key).expect("PIN state").unlocked,
            "relisting keeps the session"
        );
        hub.end_sessions();
        assert!(!hub.pin_state(key).expect("PIN state").unlocked);

        let alternate = KeyRef { path: 1, ..key };
        assert!(matches!(
            hub.sign(alternate, &request),
            Err(KeystoreError::NotFound)
        ));
    }
}
