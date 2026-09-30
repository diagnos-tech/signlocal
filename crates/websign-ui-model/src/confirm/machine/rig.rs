//! A scripted clock and shortcuts for driving a [`ConfirmModel`] in tests.

use std::time::{Duration, Instant};

use websign_core::present::caller::CallerLabel;
use websign_protocol::VerificationCode;
use websign_protocol::types::HashName;

use super::*;
use crate::certs::{CertCandidate, PinMode};
use crate::confirm::port::{CallerView, Finish, Mode, OpenRequest, RequestKey};
use crate::fixtures::{candidate, context, fingerprint};

pub const KEY: RequestKey = RequestKey(1);

pub struct Rig {
    pub model: ConfirmModel,
    start: Instant,
    pub ms: u64,
}

pub fn request(mode: Mode, remembered: bool) -> OpenRequest {
    OpenRequest {
        key: KEY,
        mode,
        caller: CallerView::Desktop {
            label: CallerLabel {
                name: "App".into(),
                detail: "path".into(),
                verified: true,
            },
        },
        remembered,
        can_remember: true,
        position: (1, 1),
        timeout_secs: 300,
    }
}

pub const SIGN: Mode = Mode::Sign {
    hash: HashName::Sha256,
};

pub fn code() -> VerificationCode {
    VerificationCode {
        text: "7F3A 9C21 E0B4 55D8".into(),
        color_index: 1,
        cells: vec![true; 25],
    }
}

pub fn pin_app(locked: bool) -> PinMode {
    PinMode::App {
        length: Some((4, 8)),
        count_low: false,
        final_try: false,
        locked,
    }
}

impl Rig {
    pub fn new() -> Rig {
        Rig {
            model: ConfirmModel::new(),
            start: Instant::now(),
            ms: 0,
        }
    }

    pub fn now(&self) -> Instant {
        self.start + Duration::from_millis(self.ms)
    }

    pub fn wait(&mut self, ms: u64) {
        self.ms += ms;
    }

    pub fn apply(&mut self, command: UiCommand) {
        self.model.apply(command, self.now());
    }

    pub fn input(&mut self, input: UserInput) -> Vec<Intent> {
        self.model.input(input, self.now())
    }

    pub fn view(&self) -> ConfirmView {
        self.model.view(self.now())
    }

    /// Open, focus, list `candidates`, all at the current instant.
    pub fn open(&mut self, mode: Mode, remembered: bool, candidates: Vec<CertCandidate>) {
        self.apply(UiCommand::Open(request(mode, remembered)));
        self.input(UserInput::Focus(true));
        self.apply(UiCommand::Certificates {
            key: KEY,
            candidates,
            possible: Vec::new(),
            context: context(),
        });
    }

    pub fn open_default(&mut self, mode: Mode, remembered: bool) {
        self.open(
            mode,
            remembered,
            vec![candidate(1, "Ana"), candidate(2, "Bia")],
        );
    }

    /// A full armed click on the primary button.
    pub fn click(&mut self) -> Vec<Intent> {
        self.wait(700);
        self.input(UserInput::PrimaryPress);
        self.wait(10);
        self.input(UserInput::PrimaryRelease)
    }

    pub fn digest_ready(&mut self, seed: u8) {
        self.apply(UiCommand::DigestReady {
            key: KEY,
            fingerprint: fingerprint(seed),
            code: code(),
        });
    }

    pub fn finish(&mut self, finish: Finish) {
        self.apply(UiCommand::Finished { key: KEY, finish });
    }
}
