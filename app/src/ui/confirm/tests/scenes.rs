//! Every state of `docs/ux.md` §4.8 (and the header variants of §4.3),
//! reached the way the engine reaches it: a list of named scenes, each a
//! script of host commands and waits on a fresh window.

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use websign_protocol::types::{BrowserName, SignatureAlgorithmName};
use websign_ui_model::certs::PinMode;
use websign_ui_model::confirm::UiCommand;
use websign_ui_model::confirm::port::{Failure, Finish, Mode};

use super::fixtures::*;
use super::support::Rig;

pub type Scene = fn(&mut Rig);

/// `(name, script)` of every state the window can show.
pub const SCENES: [(&str, Scene); 19] = [
    ("loading", loading),
    ("loading-slow", loading_slow),
    ("empty", empty),
    ("continue-new-site", continue_new_site),
    ("preparing", preparing),
    ("arming", arming),
    ("ready", ready),
    ("pin-field", pin_field),
    ("pin-error", pin_error),
    ("pin-locked", pin_locked),
    ("signing", signing),
    ("error-driver", error_driver),
    ("error-unsupported", error_unsupported),
    ("success", success),
    ("site-cancelled", site_cancelled),
    ("timeout", timeout),
    ("choose", choose),
    ("desktop-unverified", desktop_unverified),
    ("queue-idn-expiring", queue_idn_expiring),
];

fn open_site(rig: &mut Rig, mode: Mode, remembered: bool) {
    let caller = web("https://app.diagnos.health", BrowserName::Chrome);
    rig.open(request(mode, caller, remembered));
}

fn main_list(rig: &mut Rig) {
    rig.list(
        vec![ana_card(), ana_a1(), clinic_a1(), old_a3()],
        Vec::new(),
    );
}

fn loading(rig: &mut Rig) {
    open_site(rig, SIGN, true);
    rig.wait(200);
}

fn loading_slow(rig: &mut Rig) {
    open_site(rig, SIGN, true);
    rig.apply(UiCommand::SlowListing {
        key: KEY,
        device: Some("SafeNet eToken 5110".to_owned()),
    });
    rig.wait(2100);
}

fn empty(rig: &mut Rig) {
    open_site(rig, SIGN, true);
    rig.list(Vec::new(), vec![token_without_driver()]);
    rig.wait(700);
}

fn continue_new_site(rig: &mut Rig) {
    open_site(rig, SIGN, false);
    rig.list(vec![ana_card(), ana_a1()], vec![token_without_driver()]);
    rig.wait(700);
}

fn preparing(rig: &mut Rig) {
    open_site(rig, SIGN, true);
    main_list(rig);
    rig.wait(200);
}

fn arming(rig: &mut Rig) {
    open_site(rig, SIGN, true);
    main_list(rig);
    rig.digest(1);
    rig.wait(100);
}

/// Armed, with a PIN typed where this OS asks for one in our window.
fn ready(rig: &mut Rig) {
    arming(rig);
    rig.wait(600);
    if let Some(field) = rig.harness.query_by_role(Role::PasswordInput) {
        field.type_text("123456");
        rig.settle();
    }
}

fn open_token(rig: &mut Rig, pin: PinMode) {
    let caller = web("https://app.diagnos.health", BrowserName::Firefox);
    rig.open(request(SIGN, caller, true));
    rig.list(vec![ana_token(pin), ana_a1()], Vec::new());
    rig.digest(5);
    rig.wait(700);
}

fn pin_field(rig: &mut Rig) {
    open_token(rig, app_pin(false, false, false));
}

fn pin_error(rig: &mut Rig) {
    open_token(rig, app_pin(false, true, false));
    rig.fail(Failure::PinIncorrect {
        count_low: false,
        final_try: true,
    });
}

fn pin_locked(rig: &mut Rig) {
    open_token(rig, app_pin(false, false, false));
    rig.fail(Failure::PinLocked {
        tool: Some("SafeNet Authentication Client".to_owned()),
        issuer: "AC Certisign".to_owned(),
    });
}

fn signing(rig: &mut Rig) {
    ready(rig);
    rig.apply(UiCommand::Signing { key: KEY });
}

fn error_driver(rig: &mut Rig) {
    ready(rig);
    rig.fail(card_failure());
    rig.wait(700);
}

fn error_unsupported(rig: &mut Rig) {
    ready(rig);
    rig.fail(Failure::UnsupportedAlgorithm {
        algorithm: SignatureAlgorithmName::RsaPss,
    });
}

fn success(rig: &mut Rig) {
    ready(rig);
    rig.finish(Finish::Signed);
}

fn site_cancelled(rig: &mut Rig) {
    ready(rig);
    rig.finish(Finish::SiteCancelled);
}

fn timeout(rig: &mut Rig) {
    ready(rig);
    rig.finish(Finish::Timeout);
}

fn choose(rig: &mut Rig) {
    let caller = web("https://app.diagnos.health", BrowserName::Chrome);
    rig.open(request(Mode::Choose, caller, false));
    rig.list(marta(), Vec::new());
    rig.wait(700);
}

fn desktop_unverified(rig: &mut Rig) {
    rig.open(request(SIGN, desktop("laudos.exe", false), false));
    main_list(rig);
    rig.wait(700);
}

fn queue_idn_expiring(rig: &mut Rig) {
    let caller = web("https://xn--dignos-4nf.health", BrowserName::Edge);
    let mut open = request(SIGN, caller, false);
    open.position = (1, 3);
    rig.open(open);
    main_list(rig);
    rig.wait(700);
    rig.wait(300 * 1000 - 28 * 1000 - 1400);
}
