//! SPEC §3.2 (ux §8.2): the "Getting started" strip.

use websign_ui_model::diagnostics::onboarding::{OnboardingFacts, StepState, onboarding};

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
    // A working extension is what the step is about; a problem in another
    // browser shows on the Browsers light instead.
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

#[test]
fn ready_means_everything_a_signature_needs_is_there() {
    let ready = OnboardingFacts {
        any_extension_connected: true,
        usable_certificates: 1,
        ..facts()
    };
    let strip = onboarding(ready);
    assert!(strip.ready);
    assert!(strip.visible, "until the test signature");

    for missing in [
        OnboardingFacts {
            usable_certificates: 0,
            ..ready
        },
        OnboardingFacts {
            any_extension_connected: false,
            ..ready
        },
        OnboardingFacts {
            card_service_running: Some(false),
            ..ready
        },
    ] {
        assert!(!onboarding(missing).ready, "{missing:?}");
    }
}
