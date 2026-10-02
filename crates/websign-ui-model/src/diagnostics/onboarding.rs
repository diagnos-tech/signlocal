//! "Getting started" (`docs/ux.md` §8.2): what is still missing before the
//! first signature, one step per thing a person can fix, and when it is
//! all there ("You're ready to sign").

/// One step's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepState {
    Done,
    Attention,
    Pending,
}

/// The things a first signature needs, in the order they are fixed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// The browsers can start the app (its registration).
    App,
    /// The extension talks to the app in at least one browser.
    Extension,
    /// Linux: the card service (`pcscd`) runs.
    CardService,
    /// A connected token or card waits for its vendor's driver.
    Driver,
    /// At least one certificate can sign.
    Certificate,
    /// The test page signed once.
    TestSignature,
}

/// The whole checklist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Onboarding {
    pub app: StepState,
    pub extension: StepState,
    pub certificate: StepState,
    pub test_signature: StepState,
    /// `None` where there is no card service to run (Windows, macOS).
    pub card_service: Option<StepState>,
    /// `None` while no connected device waits for a driver.
    pub driver: Option<StepState>,
    /// Everything a signature needs is there; only the test may be left.
    pub ready: bool,
    /// Shown until the person is ready and has signed the test page, or
    /// hid it.
    pub visible: bool,
}

impl Onboarding {
    /// The steps that apply here, in order.
    pub fn steps(&self) -> Vec<(Step, StepState)> {
        [
            (Step::App, Some(self.app)),
            (Step::Extension, Some(self.extension)),
            (Step::CardService, self.card_service),
            (Step::Driver, self.driver),
            (Step::Certificate, Some(self.certificate)),
            (Step::TestSignature, Some(self.test_signature)),
        ]
        .into_iter()
        .filter_map(|(step, state)| state.map(|state| (step, state)))
        .collect()
    }
}

/// Inputs of the checklist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OnboardingFacts {
    pub any_extension_connected: bool,
    pub any_extension_problem: bool,
    pub usable_certificates: u32,
    pub test_signature_done: bool,
    pub dismissed: bool,
    /// No installed browser can start the app (registration missing in
    /// all of them).
    pub app_unreachable: bool,
    /// Linux: whether the card service runs; `None` elsewhere.
    pub card_service_running: Option<bool>,
    /// Connected tokens and cards that brought no certificate because
    /// their driver is missing.
    pub devices_without_driver: u32,
}

/// The checklist for `facts`.
pub fn onboarding(facts: OnboardingFacts) -> Onboarding {
    let extension = if facts.any_extension_connected {
        StepState::Done
    } else if facts.any_extension_problem {
        StepState::Attention
    } else {
        StepState::Pending
    };
    // A connected extension proves the browser started the app.
    let app = if facts.app_unreachable && !facts.any_extension_connected {
        StepState::Attention
    } else {
        StepState::Done
    };
    let done_or = |done: bool, otherwise: StepState| {
        if done { StepState::Done } else { otherwise }
    };
    let card_service = facts
        .card_service_running
        .map(|running| done_or(running, StepState::Attention));
    let driver = (facts.devices_without_driver > 0).then_some(StepState::Attention);
    let certificate = done_or(facts.usable_certificates > 0, StepState::Pending);
    let test_signature = done_or(facts.test_signature_done, StepState::Pending);
    // A missing driver for one device does not stop a certificate that
    // already works; a stopped card service stops every card.
    let ready = app == StepState::Done
        && extension == StepState::Done
        && card_service != Some(StepState::Attention)
        && certificate == StepState::Done;
    Onboarding {
        app,
        extension,
        certificate,
        test_signature,
        card_service,
        driver,
        ready,
        visible: !facts.dismissed && !(ready && facts.test_signature_done),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_install_shows_the_strip() {
        let strip = onboarding(OnboardingFacts::default());
        assert_eq!(strip.app, StepState::Done);
        assert_eq!(strip.extension, StepState::Pending);
        assert_eq!(strip.certificate, StepState::Pending);
        assert!(!strip.ready);
        assert!(strip.visible);
    }

    #[test]
    fn steps_list_only_what_applies_here() {
        let steps = |facts| {
            onboarding(facts)
                .steps()
                .into_iter()
                .map(|(step, _)| step)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            steps(OnboardingFacts::default()),
            [
                Step::App,
                Step::Extension,
                Step::Certificate,
                Step::TestSignature
            ]
        );
        let linux_with_token = OnboardingFacts {
            card_service_running: Some(true),
            devices_without_driver: 1,
            ..Default::default()
        };
        assert_eq!(
            steps(linux_with_token),
            [
                Step::App,
                Step::Extension,
                Step::CardService,
                Step::Driver,
                Step::Certificate,
                Step::TestSignature
            ]
        );
    }

    #[test]
    fn a_stopped_card_service_is_not_ready_but_a_missing_driver_can_be() {
        let working = OnboardingFacts {
            any_extension_connected: true,
            usable_certificates: 1,
            card_service_running: Some(true),
            devices_without_driver: 1,
            ..Default::default()
        };
        assert!(onboarding(working).ready);
        let stopped = OnboardingFacts {
            card_service_running: Some(false),
            ..working
        };
        assert_eq!(onboarding(stopped).card_service, Some(StepState::Attention));
        assert!(!onboarding(stopped).ready);
    }

    #[test]
    fn an_unreachable_app_needs_attention_until_an_extension_connects() {
        let unreachable = OnboardingFacts {
            app_unreachable: true,
            ..Default::default()
        };
        assert_eq!(onboarding(unreachable).app, StepState::Attention);
        let connected = OnboardingFacts {
            any_extension_connected: true,
            ..unreachable
        };
        assert_eq!(onboarding(connected).app, StepState::Done);
    }
}
