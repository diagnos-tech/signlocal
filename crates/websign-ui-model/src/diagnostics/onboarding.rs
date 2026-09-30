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
    let _ = facts;
    todo!("SPEC.md §3.2")
}
