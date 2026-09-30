//! The people, certificates and callers of the mockups (`docs/ux/mockups.html`):
//! Dr. Ana Beatriz Souza signing at app.diagnos.health, her clinic, and
//! Marta Sofia Carvalho's Cartão de Cidadão. "Today" is 2026-09-29 12:00 UTC.

use jiff::civil::date;
use jiff::tz::TimeZone;
use websign_core::present::caller::CallerLabel;
use websign_core::present::origin::format_origin;
use websign_core::{
    CertInfo, DistinguishedName, Fingerprint, IcpBrasil, IcpLevel, KeyUsage, PublicKeyKind,
    Qualified, SignatureAlgorithm,
};
use websign_protocol::VerificationCode;
use websign_protocol::code::verification_code;
use websign_protocol::types::{BrowserName, HashName};
use websign_ui_model::certs::{CertCandidate, DeviceLabel, KeySource, ListContext, PinMode};
use websign_ui_model::confirm::port::{CallerView, Failure, Mode, OpenRequest, RequestKey};
use websign_ui_model::possible::PossibleCard;

pub const KEY: RequestKey = RequestKey(7);
pub const SIGN: Mode = Mode::Sign {
    hash: HashName::Sha256,
};
const NOW: i64 = 1_790_683_200;
const DAY: i64 = 86_400;

pub fn fingerprint(seed: u8) -> Fingerprint {
    Fingerprint::from_bytes([seed; 32])
}

pub fn context() -> ListContext {
    ListContext {
        today: date(2026, 9, 29),
        time_zone: TimeZone::UTC,
        now: NOW,
        accepted: Vec::new(),
        last_used_here: None,
        recent_anywhere: Vec::new(),
        requested: None,
    }
}

/// A signing certificate valid for `days` more days.
fn info(seed: u8, cn: &str, issuer: &str, days: i64) -> CertInfo {
    CertInfo {
        fingerprint: fingerprint(seed),
        subject: DistinguishedName {
            common_name: Some(cn.to_owned()),
            ..Default::default()
        },
        issuer: DistinguishedName {
            common_name: Some(issuer.to_owned()),
            organization: Some("ICP-Brasil".to_owned()),
            country: Some("BR".to_owned()),
            ..Default::default()
        },
        serial_hex: format!("{seed:02x}"),
        not_before: NOW - 400 * DAY,
        not_after: NOW + days * DAY,
        key: PublicKeyKind::Rsa { bits: 2048 },
        key_usage: Some(KeyUsage {
            digital_signature: true,
            non_repudiation: true,
            ..Default::default()
        }),
        extended_key_usage: Vec::new(),
        policies: Vec::new(),
        is_ca: false,
        icp_brasil: None,
        qualified: None,
    }
}

fn icp(holder: &str, level: IcpLevel, cpf: Option<&str>, cnpj: Option<&str>) -> Option<IcpBrasil> {
    Some(IcpBrasil {
        level: Some(level),
        holder_name: Some(holder.to_owned()),
        cpf: cpf.map(str::to_owned),
        cnpj: cnpj.map(str::to_owned),
    })
}

fn candidate(
    info: CertInfo,
    source: KeySource,
    device: Option<DeviceLabel>,
    pin: PinMode,
    hardware: bool,
) -> CertCandidate {
    CertCandidate {
        fingerprint: info.fingerprint,
        info: Ok(info),
        source,
        alternates: Vec::new(),
        device,
        pin,
        algorithms: vec![SignatureAlgorithm::RsaPkcs1v15],
        hardware: Some(hardware),
        has_private_key: true,
        removed: false,
    }
}

/// Ana's A3 on a card in a reader, through Windows, 23 days left.
pub fn ana_a3() -> CertCandidate {
    let mut info = info(
        1,
        "ANA BEATRIZ SOUZA:12345678909",
        "AC SOLUTI Multipla v5",
        23,
    );
    info.icp_brasil = icp("ANA BEATRIZ SOUZA", IcpLevel::A3, Some("12345678909"), None);
    let reader = DeviceLabel::CardInReader {
        reader: "Identiv uTrust 2700 R".to_owned(),
    };
    candidate(
        info,
        KeySource::Windows,
        Some(reader),
        PinMode::System,
        true,
    )
}

/// Ana's A3 card reached the way this OS reaches it, for the screenshots:
/// the certificate store on Windows (its PIN dialog), CryptoTokenKit on
/// macOS (the Keychain's), and OpenSC's PKCS#11 driver with our PIN field on
/// Linux, which has no system key store.
pub fn ana_card() -> CertCandidate {
    let mut card = ana_a3();
    if cfg!(target_os = "macos") {
        card.source = KeySource::MacosToken;
    } else if !cfg!(windows) {
        card.source = KeySource::Driver {
            path: "/usr/lib/x86_64-linux-gnu/opensc-pkcs11.so".to_owned(),
        };
        card.pin = app_pin(false, false, false);
    }
    card
}

/// How the card's key store fails on this OS: the Windows smart-card
/// provider (with the driver as the other path), CryptoTokenKit, or the
/// PKCS#11 driver itself (no other path).
pub fn card_failure() -> Failure {
    let (driver, native, alternate) = if cfg!(windows) {
        (
            "Microsoft Smart Card Key Storage Provider",
            "NTE_BAD_KEYSET (0x80090016)",
            true,
        )
    } else if cfg!(target_os = "macos") {
        ("CryptoTokenKit", "errSecInternalComponent (-2070)", true)
    } else {
        ("OpenSC", "CKR_DEVICE_ERROR (0x00000030)", false)
    };
    Failure::DriverFailure {
        driver: driver.to_owned(),
        native: native.to_owned(),
        alternate,
    }
}

/// Ana's A1 on this computer (Windows store), until 14 Mar 2027.
pub fn ana_a1() -> CertCandidate {
    let mut info = info(
        2,
        "ANA BEATRIZ SOUZA:12345678909",
        "AC Certisign RFB G5",
        166,
    );
    info.icp_brasil = icp("ANA BEATRIZ SOUZA", IcpLevel::A1, Some("12345678909"), None);
    candidate(info, KeySource::Windows, None, PinMode::System, false)
}

/// The clinic's A1, 5 days left.
pub fn clinic_a1() -> CertCandidate {
    let mut info = info(
        3,
        "CLINICA SOUZA IMAGEM LTDA:12345678000190",
        "AC Certisign RFB G5",
        5,
    );
    info.icp_brasil = icp(
        "CLINICA SOUZA IMAGEM LTDA",
        IcpLevel::A1,
        None,
        Some("12345678000190"),
    );
    candidate(info, KeySource::Windows, None, PinMode::System, false)
}

/// Ana's previous A3, expired on 10 May 2026.
pub fn old_a3() -> CertCandidate {
    let mut info = info(
        4,
        "ANA BEATRIZ SOUZA:12345678909",
        "AC SOLUTI Multipla v5",
        -142,
    );
    info.icp_brasil = icp("ANA BEATRIZ SOUZA", IcpLevel::A3, Some("12345678909"), None);
    candidate(info, KeySource::Windows, None, PinMode::System, true)
}

/// Ana's A3 on a SafeNet token through its PKCS#11 driver: our PIN field.
pub fn ana_token(pin: PinMode) -> CertCandidate {
    let mut info = info(
        5,
        "ANA BEATRIZ SOUZA:12345678909",
        "AC Certisign RFB G5",
        1119,
    );
    info.icp_brasil = icp("ANA BEATRIZ SOUZA", IcpLevel::A3, Some("12345678909"), None);
    let token = DeviceLabel::Token {
        name: "SafeNet eToken 5110".to_owned(),
    };
    let driver = KeySource::Driver {
        path: "C:\\Windows\\System32\\eTPKCS11.dll".to_owned(),
    };
    candidate(info, driver, Some(token), pin, true)
}

pub fn app_pin(count_low: bool, final_try: bool, locked: bool) -> PinMode {
    PinMode::App {
        length: Some((4, 16)),
        count_low,
        final_try,
        locked,
    }
}

/// Marta's Cartão de Cidadão signing certificate and a qualified token.
pub fn marta() -> Vec<CertCandidate> {
    let mut cc = info(
        6,
        "Marta Sofia Carvalho",
        "EC de Assinatura Digital Qualificada do Cartão de Cidadão 0017",
        1362,
    );
    cc.subject.serial_number = Some("IDCPT-12345123".to_owned());
    let card = DeviceLabel::CardInReader {
        reader: "Identiv uTrust 2700 R".to_owned(),
    };
    let mut qualified = info(
        7,
        "Marta Sofia Carvalho",
        "Multicert CA Qualificada 005",
        12,
    );
    qualified.qualified = Some(Qualified {
        compliance: true,
        sscd: true,
        types: Vec::new(),
    });
    vec![
        candidate(cc, KeySource::Windows, Some(card), PinMode::System, true),
        candidate(
            qualified,
            KeySource::Windows,
            Some(DeviceLabel::Unknown),
            PinMode::System,
            true,
        ),
    ]
}

pub fn token_without_driver() -> PossibleCard {
    PossibleCard {
        name: Some("SafeNet eToken 5110".to_owned()),
        reader: None,
        card: false,
        driver: Some("SafeNet Authentication Client".to_owned()),
        download: Some("https://example.com/sac".to_owned()),
        needs_complement: false,
    }
}

pub fn web(origin: &str, browser: BrowserName) -> CallerView {
    CallerView::Web {
        origin: format_origin(origin).expect("a valid test origin"),
        top: None,
        browser,
    }
}

pub fn desktop(name: &str, verified: bool) -> CallerView {
    CallerView::Desktop {
        label: CallerLabel {
            name: name.to_owned(),
            detail: "Diagnos Health Ltda".to_owned(),
            verified,
        },
    }
}

pub fn request(mode: Mode, caller: CallerView, remembered: bool) -> OpenRequest {
    let can_remember = match &caller {
        CallerView::Web { origin, .. } => origin.can_remember,
        CallerView::Desktop { .. } => true,
    };
    OpenRequest {
        key: KEY,
        mode,
        caller,
        remembered,
        can_remember,
        position: (1, 1),
        timeout_secs: 300,
    }
}

/// The code of the mockups: 7F3A 9C21 E0B4 55D8.
pub fn code() -> VerificationCode {
    verification_code(&[0x7F, 0x3A, 0x9C, 0x21, 0xE0, 0xB4, 0x55, 0xD8]).expect("8 bytes")
}
