//! SPEC §3.1 and §3.2 (ux §8.1, §8.2): traffic lights and the first-steps strip.

use websign_ui_model::diagnostics::onboarding::{OnboardingFacts, StepState, onboarding};
use websign_ui_model::diagnostics::status::{
    BrowsersFacts, CertificatesFacts, DevicesFacts, Light, browsers_light, certificates_light,
    devices_light, overall,
};

fn browsers(connected: u32, with_problems: u32, missing: bool) -> Light {
    browsers_light(&BrowsersFacts {
        connected,
        with_problems,
        registration_missing_everywhere: missing,
    })
}

#[test]
fn browsers_are_red_when_none_is_connected() {
    assert_eq!(browsers(0, 0, false), Light::Red);
    assert_eq!(browsers(0, 2, false), Light::Red);
    assert_eq!(BrowsersFacts::default().connected, 0);
    assert_eq!(browsers_light(&BrowsersFacts::default()), Light::Red);
}

#[test]
fn browsers_are_red_when_the_registration_is_missing_everywhere() {
    assert_eq!(browsers(2, 0, true), Light::Red);
    assert_eq!(browsers(0, 0, true), Light::Red);
}

#[test]
fn browsers_are_yellow_when_one_is_connected_but_another_has_problems() {
    assert_eq!(browsers(1, 1, false), Light::Yellow);
    assert_eq!(browsers(3, 2, false), Light::Yellow);
}

#[test]
fn browsers_are_green_when_all_installed_ones_work() {
    assert_eq!(browsers(1, 0, false), Light::Green);
    assert_eq!(browsers(4, 0, false), Light::Green);
}

fn devices(facts: DevicesFacts) -> Light {
    devices_light(&facts)
}

#[test]
fn devices_are_green_when_nothing_is_wrong_or_nothing_is_detected() {
    assert_eq!(devices(DevicesFacts::default()), Light::Green);
}

#[test]
fn devices_are_yellow_for_each_soft_problem() {
    let soft = [
        DevicesFacts {
            devices_without_certificates: 1,
            ..DevicesFacts::default()
        },
        DevicesFacts {
            drivers_failed: 1,
            ..DevicesFacts::default()
        },
        DevicesFacts {
            complement_outdated: true,
            ..DevicesFacts::default()
        },
    ];
    for facts in soft {
        assert_eq!(devices(facts.clone()), Light::Yellow, "{facts:?}");
    }
}

#[test]
fn devices_are_red_when_pcscd_is_stopped_whatever_else_is_true() {
    let stopped = DevicesFacts {
        pcscd_stopped: true,
        ..DevicesFacts::default()
    };
    assert_eq!(devices(stopped.clone()), Light::Red);
    let with_others = DevicesFacts {
        drivers_failed: 2,
        devices_without_certificates: 1,
        complement_outdated: true,
        ..stopped
    };
    assert_eq!(devices(with_others), Light::Red);
}

fn certificates(usable: u32, expiring: u32) -> Light {
    certificates_light(&CertificatesFacts {
        usable,
        expiring_within_30_days: expiring,
    })
}

#[test]
fn certificates_are_red_without_a_usable_one() {
    assert_eq!(certificates(0, 0), Light::Red);
    assert_eq!(certificates(0, 3), Light::Red);
}

#[test]
fn certificates_are_yellow_when_a_usable_one_expires_within_30_days() {
    assert_eq!(certificates(3, 1), Light::Yellow);
    assert_eq!(certificates(1, 1), Light::Yellow);
}

#[test]
fn certificates_are_green_otherwise() {
    assert_eq!(certificates(1, 0), Light::Green);
    assert_eq!(certificates(9, 0), Light::Green);
}

#[test]
fn lights_rank_red_over_yellow_over_green_over_gray() {
    assert!(Light::Red > Light::Yellow);
    assert!(Light::Yellow > Light::Green);
    assert!(Light::Green > Light::Gray);
}

#[test]
fn the_overall_light_is_the_worst_one() {
    use Light::*;
    let table: [(&[Light], Light); 9] = [
        (&[], Gray),
        (&[Gray], Gray),
        (&[Green], Green),
        (&[Green, Gray], Green),
        (&[Gray, Green, Gray], Green),
        (&[Green, Yellow, Gray], Yellow),
        (&[Green, Green, Yellow], Yellow),
        (&[Red, Green], Red),
        (&[Yellow, Red, Green, Gray], Red),
    ];
    for (lights, expected) in table {
        assert_eq!(overall(lights), expected, "{lights:?}");
    }
}

fn facts() -> OnboardingFacts {
    OnboardingFacts::default()
}

#[test]
fn the_app_step_is_always_done() {
    assert_eq!(onboarding(facts()).app, StepState::Done);
}

#[test]
fn the_extension_step_is_done_attention_or_pending() {
    let connected = OnboardingFacts {
        any_extension_connected: true,
        ..facts()
    };
    assert_eq!(onboarding(connected).extension, StepState::Done);

    let problem = OnboardingFacts {
        any_extension_problem: true,
        ..facts()
    };
    assert_eq!(onboarding(problem).extension, StepState::Attention);

    assert_eq!(onboarding(facts()).extension, StepState::Pending);
}

#[test]
fn a_connected_extension_wins_over_a_problem_elsewhere() {
    // SPEC §3.2 lists "Done if connected" first.
    let both = OnboardingFacts {
        any_extension_connected: true,
        any_extension_problem: true,
        ..facts()
    };
    assert_eq!(onboarding(both).extension, StepState::Done);
}

#[test]
fn the_certificate_and_test_steps_follow_their_facts() {
    let strip = onboarding(facts());
    assert_eq!(strip.certificate, StepState::Pending);
    assert_eq!(strip.test_signature, StepState::Pending);

    let strip = onboarding(OnboardingFacts {
        usable_certificates: 2,
        test_signature_done: true,
        ..facts()
    });
    assert_eq!(strip.certificate, StepState::Done);
    assert_eq!(strip.test_signature, StepState::Done);
}

#[test]
fn the_strip_is_visible_until_everything_is_done_or_it_is_dismissed() {
    assert!(onboarding(facts()).visible);

    let almost = OnboardingFacts {
        any_extension_connected: true,
        usable_certificates: 1,
        ..facts()
    };
    assert!(onboarding(almost).visible, "the test signature is pending");

    let all_done = OnboardingFacts {
        test_signature_done: true,
        ..almost
    };
    assert!(!onboarding(all_done).visible);

    let dismissed = OnboardingFacts {
        dismissed: true,
        ..facts()
    };
    assert!(!onboarding(dismissed).visible);
}

#[test]
fn an_extension_problem_keeps_the_strip_visible_even_with_the_rest_done() {
    let facts = OnboardingFacts {
        any_extension_problem: true,
        usable_certificates: 1,
        test_signature_done: true,
        ..facts()
    };
    assert!(onboarding(facts).visible);
}
