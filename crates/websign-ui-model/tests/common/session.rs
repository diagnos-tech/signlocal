//! A confirmation window driven with a fake clock.

use std::time::{Duration, Instant};

use websign_core::Fingerprint;
use websign_core::present::origin::FormattedOrigin;
use websign_protocol::VerificationCode;
use websign_protocol::types::{BrowserName, HashName};
use websign_ui_model::certs::{CertCandidate, ListContext};
use websign_ui_model::confirm::port::{
    CallerView, Failure, Finish, Mode, OpenRequest, RequestKey, UiCommand,
};
use websign_ui_model::confirm::{ConfirmModel, ConfirmState, ConfirmView, Intent, UserInput};

pub const KEY: RequestKey = RequestKey(1);

pub fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

pub fn origin() -> FormattedOrigin {
    FormattedOrigin {
        canonical: "https://app.diagnos.health".to_owned(),
        prefix: "https://app.".to_owned(),
        registrable: "diagnos.health".to_owned(),
        port: None,
        unicode: None,
        warning: None,
        can_remember: true,
    }
}

pub fn caller() -> CallerView {
    CallerView::Web {
        origin: origin(),
        top: None,
        browser: BrowserName::Chrome,
    }
}

/// A remembered caller's consent covers every fixture certificate
/// (`fp(n)` for n below 32); a new caller's covers none.
pub fn consent_of(remembered: bool) -> Vec<Fingerprint> {
    let seeds = if remembered { 0..32 } else { 0..0 };
    seeds.map(super::cert::fp).collect()
}

pub fn request(key: RequestKey, mode: Mode, remembered: bool) -> OpenRequest {
    OpenRequest {
        key,
        mode,
        caller: caller(),
        remembered,
        consented: consent_of(remembered),
        can_remember: true,
        position: (1, 1),
        timeout_secs: 300,
    }
}

pub fn sign_mode() -> Mode {
    Mode::Sign {
        hash: HashName::Sha256,
    }
}

pub fn code(text: &str) -> VerificationCode {
    VerificationCode {
        text: text.to_owned(),
        color_index: 3,
        cells: vec![false; 25],
    }
}

/// The model plus the clock. `now` only moves when a test says so.
#[derive(Debug)]
pub struct Window {
    pub model: ConfirmModel,
    pub now: Instant,
    pub start: Instant,
    pub key: RequestKey,
}

impl Window {
    pub fn new() -> Window {
        let now = Instant::now();
        Window {
            model: ConfirmModel::new(),
            now,
            start: now,
            key: KEY,
        }
    }

    pub fn wait(&mut self, millis: u64) -> &mut Self {
        self.now += ms(millis);
        self
    }

    /// Moves the clock to `millis` after the window was created.
    pub fn at(&mut self, millis: u64) -> &mut Self {
        self.now = self.start + ms(millis);
        self
    }

    pub fn state(&self) -> ConfirmState {
        self.model.state().clone()
    }

    pub fn view(&self) -> ConfirmView {
        self.model.view(self.now)
    }

    pub fn apply(&mut self, command: UiCommand) -> &mut Self {
        self.model.apply(command, self.now);
        self
    }

    pub fn input(&mut self, input: UserInput) -> Vec<Intent> {
        self.model.input(input, self.now)
    }

    pub fn tick(&mut self) -> Vec<Intent> {
        self.model.tick(self.now)
    }

    /// A press and release of the primary button at the current instant.
    pub fn click(&mut self) -> Vec<Intent> {
        let mut intents = self.input(UserInput::PrimaryPress);
        intents.extend(self.input(UserInput::PrimaryRelease));
        intents
    }

    /// Opens a request and gives the window focus, both at the current instant.
    pub fn open(&mut self, request: OpenRequest) -> &mut Self {
        self.key = request.key;
        self.apply(UiCommand::Open(request));
        self.input(UserInput::Focus(true));
        self
    }

    pub fn certificates(
        &mut self,
        candidates: Vec<CertCandidate>,
        context: ListContext,
    ) -> &mut Self {
        let key = self.key;
        self.apply(UiCommand::Certificates {
            key,
            candidates,
            possible: Vec::new(),
            context,
        })
    }

    pub fn digest(&mut self, fingerprint: Fingerprint, text: &str) -> &mut Self {
        let key = self.key;
        self.apply(UiCommand::DigestReady {
            key,
            fingerprint,
            code: code(text),
        })
    }

    pub fn fail(&mut self, failure: Failure) -> &mut Self {
        let key = self.key;
        self.apply(UiCommand::Failed { key, failure })
    }

    pub fn finish(&mut self, finish: Finish) -> &mut Self {
        let key = self.key;
        self.apply(UiCommand::Finished { key, finish })
    }

    pub fn signing(&mut self) -> &mut Self {
        let key = self.key;
        self.apply(UiCommand::Signing { key })
    }
}

/// Window of a remembered caller in Sign mode: list loaded, digest shown for
/// `candidates[0]`, everything armed (the clock is 1 s past the last event).
pub fn ready_remembered(candidates: Vec<CertCandidate>, context: ListContext) -> Window {
    let first = candidates[0].fingerprint;
    let mut window = Window::new();
    window.open(request(KEY, sign_mode(), true));
    window.wait(100).certificates(candidates, context);
    window.digest(first, "7F3A 9C21 E0B4 55D8");
    window.wait(1000);
    window
}

/// Window of a new caller in Sign mode after "Continue" and the digest,
/// armed. `candidates[0]` must be the initial selection.
pub fn ready_new_caller(candidates: Vec<CertCandidate>, context: ListContext) -> Window {
    let first = candidates[0].fingerprint;
    let mut window = Window::new();
    window.open(request(KEY, sign_mode(), false));
    window.wait(100).certificates(candidates, context);
    window.wait(1000);
    assert_eq!(window.click(), vec![Intent::Continue(first)]);
    window.wait(50).digest(first, "7F3A 9C21 E0B4 55D8");
    window.wait(1000);
    window
}
