//! SPEC §2.3 and §2.4: which error a closed window reports, and how a
//! failure maps to its protocol code.

use websign_protocol::ErrorCode;
use websign_protocol::types::SignatureAlgorithmName;
use websign_ui_model::confirm::ConfirmState;
use websign_ui_model::confirm::cancel::cancel_code;
use websign_ui_model::confirm::port::Failure;
use websign_ui_model::confirm::view::banner_code;

#[test]
fn an_empty_list_reports_no_certificates() {
    assert_eq!(cancel_code(&ConfirmState::Empty), ErrorCode::NoCertificates);
}

#[test]
fn a_locked_pin_reports_pin_locked() {
    assert_eq!(cancel_code(&ConfirmState::PinLocked), ErrorCode::PinLocked);
}

#[test]
fn an_unavailable_certificate_reports_certificate_unavailable() {
    let state = ConfirmState::Error {
        code: ErrorCode::CertificateUnavailable,
    };
    assert_eq!(cancel_code(&state), ErrorCode::CertificateUnavailable);
}

#[test]
fn every_other_state_reports_user_cancelled() {
    let states = [
        ConfirmState::Idle,
        ConfirmState::LoadingCerts,
        ConfirmState::Choosing,
        ConfirmState::Ready,
        ConfirmState::PinError,
        ConfirmState::Signing,
        ConfirmState::Success,
        ConfirmState::SiteCancelled,
        ConfirmState::Timeout,
    ];
    for state in states {
        assert_eq!(cancel_code(&state), ErrorCode::UserCancelled, "{state:?}");
    }
}

#[test]
fn other_error_banners_report_user_cancelled() {
    for code in ErrorCode::ALL {
        if code == ErrorCode::CertificateUnavailable {
            continue;
        }
        let state = ConfirmState::Error { code };
        assert_eq!(cancel_code(&state), ErrorCode::UserCancelled, "{code:?}");
    }
}

#[test]
fn every_failure_maps_to_its_protocol_code() {
    let table = [
        (
            Failure::PinIncorrect {
                count_low: false,
                final_try: false,
            },
            ErrorCode::PinIncorrect,
        ),
        (
            Failure::PinIncorrect {
                count_low: true,
                final_try: true,
            },
            ErrorCode::PinIncorrect,
        ),
        (
            Failure::PinLocked {
                tool: Some("SafeNet Authentication Client".to_owned()),
                issuer: "AC SOLUTI".to_owned(),
            },
            ErrorCode::PinLocked,
        ),
        (
            Failure::PinLocked {
                tool: None,
                issuer: String::new(),
            },
            ErrorCode::PinLocked,
        ),
        (Failure::TokenRemoved, ErrorCode::TokenRemoved),
        (
            Failure::DriverFailure {
                driver: "eTPKCS11.dll".to_owned(),
                native: "CKR_DEVICE_ERROR (0x00000030)".to_owned(),
                alternate: true,
            },
            ErrorCode::DriverFailure,
        ),
        (
            Failure::UnsupportedAlgorithm {
                algorithm: SignatureAlgorithmName::RsaPss,
            },
            ErrorCode::UnsupportedAlgorithm,
        ),
        (
            Failure::CertificateUnavailable,
            ErrorCode::CertificateUnavailable,
        ),
        (
            Failure::Internal {
                detail: "bug".to_owned(),
            },
            ErrorCode::Internal,
        ),
    ];
    for (failure, code) in table {
        assert_eq!(banner_code(&failure), code, "{failure:?}");
    }
}
