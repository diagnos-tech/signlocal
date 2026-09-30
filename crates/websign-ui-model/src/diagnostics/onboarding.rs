//! "Getting started" (`docs/ux.md` §8.2).

/// One step's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepState {
    Done,
    Attention,
    Pending,
}

/// The four steps, in order: app, extension, certificate, test signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Onboarding {
    pub app: StepState,
    pub extension: StepState,
    pub certificate: StepState,
    pub test_signature: StepState,
    /// Shown until every step is done or the person hid it.
    pub visible: bool,
}

/// Inputs of the strip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OnboardingFacts {
    pub any_extension_connected: bool,
    pub any_extension_problem: bool,
    pub usable_certificates: u32,
    pub test_signature_done: bool,
    pub dismissed: bool,
}

/// The strip for `facts`.
pub fn onboarding(facts: OnboardingFacts) -> Onboarding {
    let extension = if facts.any_extension_connected {
        StepState::Done
    } else if facts.any_extension_problem {
        StepState::Attention
    } else {
        StepState::Pending
    };
    let done_or_pending = |done: bool| {
        if done {
            StepState::Done
        } else {
            StepState::Pending
        }
    };
    let mut strip = Onboarding {
        app: StepState::Done,
        extension,
        certificate: done_or_pending(facts.usable_certificates > 0),
        test_signature: done_or_pending(facts.test_signature_done),
        visible: false,
    };
    let all_done = [
        strip.app,
        strip.extension,
        strip.certificate,
        strip.test_signature,
    ]
    .iter()
    .all(|step| *step == StepState::Done);
    strip.visible = !facts.dismissed && !all_done;
    strip
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
        assert!(strip.visible);
    }

    #[test]
    fn extension_problem_needs_attention() {
        let strip = onboarding(OnboardingFacts {
            any_extension_problem: true,
            ..Default::default()
        });
        assert_eq!(strip.extension, StepState::Attention);
        let connected = onboarding(OnboardingFacts {
            any_extension_problem: true,
            any_extension_connected: true,
            ..Default::default()
        });
        assert_eq!(connected.extension, StepState::Done);
    }

    #[test]
    fn hidden_when_dismissed_or_complete() {
        let complete = OnboardingFacts {
            any_extension_connected: true,
            any_extension_problem: false,
            usable_certificates: 1,
            test_signature_done: true,
            dismissed: false,
        };
        assert!(!onboarding(complete).visible);
        let dismissed = OnboardingFacts {
            dismissed: true,
            ..Default::default()
        };
        assert!(!onboarding(dismissed).visible);
    }
}
