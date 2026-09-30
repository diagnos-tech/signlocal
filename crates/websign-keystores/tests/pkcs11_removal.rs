//! A SoftHSM2 token deleted while the module is loaded and the token is
//! unlocked: what a pulled card looks like to the adapter.

mod support;

#[test]
fn softhsm_token_removal() {
    support::softhsm::run_children("after_removal::");
}

mod after_removal {
    use std::process::Command;

    use secrecy::SecretString;
    use websign_core::{HashAlgorithm, SignatureAlgorithm};
    use websign_keystores::{FoundKey, Keystore, KeystoreError, SignRequest};

    use crate::support::softhsm::SoftHsm;

    fn sign(
        keystore: &mut dyn Keystore,
        key: &FoundKey,
        pin: Option<&str>,
    ) -> Result<(), KeystoreError> {
        let digest = HashAlgorithm::Sha256.digest(b"removal");
        let pin = pin.map(|pin| SecretString::from(pin.to_owned()));
        let request = SignRequest {
            hash: HashAlgorithm::Sha256,
            algorithm: SignatureAlgorithm::Ecdsa,
            digest: &digest,
            pin: pin.as_ref(),
            parent_window: None,
        };
        keystore.sign(key, &request).map(drop)
    }

    #[test]
    #[ignore = "needs the SoftHSM2 token of softhsm_token_removal"]
    fn the_key_becomes_unavailable_and_the_session_ends() {
        let Some(token) = SoftHsm::in_child() else {
            return;
        };
        let mut keystore = token.keystore();
        let der = token.key("ec-p256");
        let key = keystore
            .list()
            .expect("list")
            .into_iter()
            .find(|key| key.cert_der == der)
            .expect("listed");
        sign(keystore.as_mut(), &key, Some(&token.pin)).expect("signs before removal");

        let status = Command::new("softhsm2-util")
            .args(["--delete-token", "--token", &token.label])
            .status()
            .expect("softhsm2-util runs");
        assert!(status.success());

        let error = sign(keystore.as_mut(), &key, None).expect_err("the token is gone");
        assert!(
            matches!(error, KeystoreError::NotFound | KeystoreError::TokenRemoved),
            "{error}"
        );
        assert!(!keystore.pin_state(&key).is_some_and(|state| state.unlocked));
        let listed = keystore.list().unwrap_or_default();
        assert!(listed.iter().all(|listed| listed.cert_der != der));
        keystore.end_sessions();
    }
}
