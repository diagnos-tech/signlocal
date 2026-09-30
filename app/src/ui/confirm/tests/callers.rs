//! Who asks, in the words the person reads: an interpreter is "a script run
//! by {program}" and cannot be remembered (§4.3.1), and a site that never
//! sent its document is blamed, not the person (§4.11).

use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use websign_protocol::types::BrowserName;
use websign_ui_model::confirm::port::Finish;

use super::fixtures::*;
use super::support::Rig;

#[test]
fn an_interpreter_is_a_script_run_by_it_and_cannot_be_remembered() {
    let mut rig = Rig::new(false);
    rig.open(request(SIGN, script("node"), false));
    rig.list(vec![ana_a3()], Vec::new());
    rig.harness.get_by_label_contains("A script run by node");
    let remember = rig
        .harness
        .get_by_role_and_label(Role::CheckBox, "Remember this program on this computer");
    assert!(remember.accesskit_node().is_disabled());
}

#[test]
fn a_digest_timeout_says_the_site_did_not_respond() {
    let mut rig = Rig::new(false);
    let caller = web("https://app.diagnos.health", BrowserName::Chrome);
    rig.open(request(SIGN, caller, false));
    rig.list(vec![ana_a3()], Vec::new());
    rig.finish(Finish::DigestTimeout);
    rig.harness.get_by_label_contains("The site didn't respond");
    rig.harness
        .get_by_label_contains("app.diagnos.health didn't prepare the document");
}
