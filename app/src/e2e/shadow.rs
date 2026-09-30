//! A copy of the window's model fed the same host commands, so the driver
//! can name the state on screen without reaching into the window.
//!
//! The window's own model also sees the person's input, but every state
//! change a screenshot is named after follows a host command (the digest
//! arrived, signing started, the request finished) or a step of the driver,
//! which the driver repeats here ([`mirror`]), so the copy agrees with what
//! is drawn. One window per process, so one copy per process.

use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use websign_host::ports::ConfirmUi;
use websign_ui_model::confirm::port::Mode;
use websign_ui_model::confirm::view::PrimaryButton;
use websign_ui_model::confirm::{ConfirmModel, ConfirmState, UiCommand, UserInput};

struct Shadow {
    model: Option<ConfirmModel>,
    choose: bool,
}

static SHADOW: Mutex<Shadow> = Mutex::new(Shadow {
    model: None,
    choose: false,
});

fn shadow() -> MutexGuard<'static, Shadow> {
    SHADOW.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The window's port, copying every command into the shadow model first.
pub struct Watched {
    inner: Box<dyn ConfirmUi>,
}

impl std::fmt::Debug for Watched {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Watched")
    }
}

impl Watched {
    pub fn new(inner: Box<dyn ConfirmUi>) -> Watched {
        Watched { inner }
    }
}

impl ConfirmUi for Watched {
    fn command(&mut self, command: UiCommand) {
        match &command {
            UiCommand::Finished { finish, .. } => {
                log::debug!("e2e: window command Finished {finish:?}")
            }
            _ => log::debug!("e2e: window command {}", kind(&command)),
        }
        {
            let mut shadow = shadow();
            let open = match &command {
                UiCommand::Open(request) => Some(request.mode == Mode::Choose),
                _ => None,
            };
            let now = Instant::now();
            let model = shadow.model.get_or_insert_with(ConfirmModel::new);
            model.apply(command.clone(), now);
            if let Some(choose) = open {
                // The driven window counts as focused (see `crate::e2e`).
                let _ = model.input(UserInput::Focus(true), now);
                shadow.choose = choose;
            }
        }
        self.inner.command(command);
    }

    fn parent_window(&self) -> Option<isize> {
        self.inner.parent_window()
    }

    fn view_certificate(&mut self, der: Vec<u8>) {
        self.inner.view_certificate(der);
    }
}

/// Repeats in the copy what the driver just did in the window.
pub fn mirror(inputs: &[UserInput]) {
    let mut shadow = shadow();
    if let Some(model) = shadow.model.as_mut() {
        for input in inputs {
            let _ = model.input(input.clone(), Instant::now());
        }
    }
}

/// The state on screen as a file-name part (`ready`, `continue-new-site`,
/// `error-driver-failure`…), `None` while nothing is shown.
pub fn state_name() -> Option<String> {
    let mut shadow = shadow();
    let choose = shadow.choose;
    let model = shadow.model.as_mut()?;
    let now = Instant::now();
    // The window keeps a notice up while its pictures are taken; so does
    // the copy, or the capture would drop them halfway.
    if !super::capture::in_progress() {
        let _ = model.tick(now);
    }
    let name = match model.state() {
        ConfirmState::Idle => return None,
        ConfirmState::LoadingCerts => "loading".to_owned(),
        ConfirmState::Empty => "empty".to_owned(),
        ConfirmState::Choosing if choose => "choose".to_owned(),
        ConfirmState::Choosing => match model.view(now).footer.primary {
            PrimaryButton::Continue => "continue-new-site".to_owned(),
            _ => "preparing".to_owned(),
        },
        ConfirmState::Ready => "ready".to_owned(),
        ConfirmState::PinError => "pin-error".to_owned(),
        ConfirmState::PinLocked => "pin-locked".to_owned(),
        ConfirmState::Signing => "signing".to_owned(),
        ConfirmState::Success => "success".to_owned(),
        ConfirmState::Error { code } => format!("error-{}", kebab(&format!("{code:?}"))),
        ConfirmState::SiteCancelled => "site-cancelled".to_owned(),
        ConfirmState::Timeout => "timeout".to_owned(),
    };
    Some(name)
}

/// The command's name only: its fields carry sites and certificates.
fn kind(command: &UiCommand) -> &'static str {
    match command {
        UiCommand::Open(_) => "Open",
        UiCommand::Certificates { .. } => "Certificates",
        UiCommand::SlowListing { .. } => "SlowListing",
        UiCommand::DigestPending { .. } => "DigestPending",
        UiCommand::DigestReady { .. } => "DigestReady",
        UiCommand::Signing { .. } => "Signing",
        UiCommand::Failed { .. } => "Failed",
        UiCommand::Finished { .. } => "Finished",
        UiCommand::Queue { .. } => "Queue",
        UiCommand::Hide => "Hide",
    }
}

/// `DriverFailure` → `driver-failure`.
fn kebab(camel: &str) -> String {
    let mut out = String::new();
    for (i, c) in camel.chars().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            out.push('-');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::kebab;

    #[test]
    fn error_codes_become_file_name_parts() {
        assert_eq!(kebab("DriverFailure"), "driver-failure");
        assert_eq!(kebab("Internal"), "internal");
    }
}
