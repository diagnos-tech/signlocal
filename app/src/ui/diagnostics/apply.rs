//! Applying what the person did: stores change here, OS actions start here,
//! and a scan follows anything that changes what the scan would find.

use std::path::{Path, PathBuf};

use egui::{Context, ViewportCommand};
use websign_host::store::Settings;

use super::pick::{Outcome, Purpose};
use super::state::{Action, NOTICE_SECONDS, Notice, REVOKE_CONFIRM_SECONDS};
use super::window::DiagnosticsWindow;
use crate::platform::file_picker::Picked;

impl DiagnosticsWindow {
    pub(super) fn apply(&mut self, ctx: &Context, action: Action, clock: f64) {
        match action {
            Action::SelectTab(tab) => {
                self.state.tab = Some(tab);
                self.state.revoke_armed = None;
            }
            Action::Rescan => self.rescan(ctx),
            Action::CopyReport => {
                if let Some(report) = &self.report {
                    ctx.copy_text(report.clone());
                    self.notify(Notice::Copied, clock);
                }
            }
            Action::CopyText(text) => ctx.copy_text(text),
            Action::OpenUrl(url) => self.parts.os.open_url(&url),
            Action::Repair(browser) => {
                let notice = if self.parts.os.repair_registration() {
                    Notice::Repaired(browser)
                } else {
                    Notice::RepairFailed
                };
                self.notify(notice, clock);
                self.rescan(ctx);
            }
            Action::ArmRevoke(key) => {
                self.state.revoke_armed = Some((key, clock + REVOKE_CONFIRM_SECONDS));
            }
            Action::Revoke { key, shown } => self.revoke(&key, shown, clock),
            Action::PickDriver => self.pick(ctx, Purpose::Driver),
            Action::EditDriverInput(path) => self.state.driver_input = Some(path),
            Action::CancelDriverInput => self.state.driver_input = None,
            Action::AddDriver(path) => self.add_driver(ctx, path),
            Action::RemoveDriver(path) => {
                self.change_settings(ctx, |settings| {
                    settings.user_modules.retain(|kept| kept != &path);
                });
            }
            // Keychain Access imports a file it is given: the window asks
            // for it with a localized panel. The Windows wizard has its own.
            Action::ImportPfx if cfg!(target_os = "macos") => self.pick(ctx, Purpose::Pfx),
            Action::ImportPfx => self.import_pfx(ctx, None),
            Action::Details(fingerprint) => {
                let der = self
                    .facts
                    .as_ref()
                    .and_then(|facts| facts.certificates.der.get(&fingerprint).cloned());
                if let Some(der) = der {
                    self.parts.os.view_certificate(&der, self.owner);
                }
            }
            Action::ToggleFaq(index) => {
                if let Some(open) = self.state.faq_open.get_mut(index) {
                    *open = !*open;
                }
            }
            Action::ToggleHidden => self.state.hidden_open = !self.state.hidden_open,
            Action::DismissOnboarding => {
                self.change_settings_only(|settings| settings.onboarding_dismissed = true);
            }
            Action::Close => ctx.send_viewport_cmd(ViewportCommand::Close),
        }
    }

    /// What the file picker answered.
    pub(super) fn picked(&mut self, ctx: &Context, (purpose, picked): Outcome) {
        match (purpose, picked) {
            (_, Picked::Cancelled) => {}
            (Purpose::Driver, Picked::Chosen(path)) => self.add_driver(ctx, path),
            (Purpose::Driver, Picked::Unavailable) => {
                self.state.driver_input = Some(String::new());
            }
            (Purpose::Pfx, Picked::Chosen(path)) => self.import_pfx(ctx, Some(&path)),
            (Purpose::Pfx, Picked::Unavailable) => self.import_pfx(ctx, None),
        }
    }

    fn pick(&mut self, ctx: &Context, purpose: Purpose) {
        let wake = {
            let ctx = ctx.clone();
            move || ctx.request_repaint()
        };
        let os = self.parts.os.as_mut();
        self.picks
            .open(os, &self.parts.catalog, purpose, self.owner, wake);
    }

    fn import_pfx(&mut self, ctx: &Context, file: Option<&Path>) {
        if self.parts.os.import_pfx(file, self.owner) {
            self.rescan(ctx);
        }
    }

    /// Remembers the driver and scans, which loads it and shows how that
    /// went on its row (`docs/ux.md` §8.4).
    fn add_driver(&mut self, ctx: &Context, path: PathBuf) {
        self.state.driver_input = None;
        self.change_settings(ctx, |settings| {
            if !settings.user_modules.contains(&path) {
                settings.user_modules.push(path.clone());
            }
        });
    }

    fn revoke(&mut self, key: &str, shown: String, clock: f64) {
        self.state.revoke_armed = None;
        match self.parts.stores.consent().revoke(key) {
            Ok(()) => self.notify(Notice::Revoked(shown), clock),
            Err(error) => log::warn!("diagnostics: revoke failed: {error}"),
        }
        self.reload_stores();
    }

    fn notify(&mut self, notice: Notice, clock: f64) {
        self.state.notice = Some((notice, clock + NOTICE_SECONDS));
    }

    /// Changes the settings, then scans with them (drivers).
    fn change_settings(&mut self, ctx: &Context, mut change: impl FnMut(&mut Settings)) {
        self.change_settings_only(&mut change);
        self.rescan(ctx);
    }

    fn change_settings_only(&mut self, mut change: impl FnMut(&mut Settings)) {
        if let Err(error) = self.parts.stores.settings().update(&mut change) {
            log::warn!("diagnostics: settings not saved: {error}");
        }
        self.reload_stores();
    }
}
