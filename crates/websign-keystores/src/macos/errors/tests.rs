use super::*;

fn no_description() -> String {
    panic!("description is only needed for errors reported as they are")
}

fn no_attempts() -> Option<i64> {
    None
}

fn map(domain: &str, code: isize) -> KeystoreError {
    let details = Details {
        describe: no_description,
        remaining_attempts: no_attempts,
    };
    classify("x", domain, code, details)
}

#[test]
fn cancellation_in_both_domains() {
    for (domain, code) in [(OSSTATUS_DOMAIN, -128), (CTK_DOMAIN, -4)] {
        assert!(
            matches!(map(domain, code), KeystoreError::Cancelled),
            "{domain} {code}"
        );
    }
}

#[test]
fn wrong_pin_in_both_domains() {
    for (domain, code) in [(OSSTATUS_DOMAIN, -25293), (CTK_DOMAIN, -5)] {
        assert!(
            matches!(map(domain, code), KeystoreError::WrongPin),
            "{domain} {code}"
        );
    }
}

#[test]
fn authentication_failure_with_no_attempts_left_is_a_blocked_pin() {
    let details = Details {
        describe: no_description,
        remaining_attempts: || Some(0),
    };
    let error = classify("x", CTK_DOMAIN, -5, details);
    assert!(matches!(error, KeystoreError::PinLocked), "{error:?}");
}

#[test]
fn token_codes_the_app_reacts_to() {
    assert!(matches!(map(CTK_DOMAIN, -7), KeystoreError::TokenRemoved));
    assert!(matches!(map(CTK_DOMAIN, -6), KeystoreError::NotFound));
    assert!(matches!(map(CTK_DOMAIN, -9), KeystoreError::PinRequired));
    assert!(matches!(map(CTK_DOMAIN, -1), KeystoreError::Unsupported(_)));
}

#[test]
fn osstatus_domain_uses_the_status_mapping() {
    assert!(matches!(
        map(OSSTATUS_DOMAIN, -25300),
        KeystoreError::NotFound
    ));
}

#[test]
fn anything_else_keeps_api_code_domain_and_symbol() {
    let details = Details {
        describe: || "communication error".to_owned(),
        remaining_attempts: no_attempts,
    };
    let error = classify("SecKeyCreateSignature", CTK_DOMAIN, -2, details);
    let KeystoreError::Native { api, code, message } = error else {
        panic!("expected Native, got {error:?}");
    };
    assert_eq!(api, "SecKeyCreateSignature");
    assert_eq!(code, -2);
    assert_eq!(
        message,
        "CryptoTokenKit TKErrorCodeCommunicationError: communication error"
    );
}
