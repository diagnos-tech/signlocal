//! The registration steps shared by `install`, `uninstall`, `register` and
//! the silent repair, as report steps.

use websign_registration::{Action, Outcome, Request, Scope, url_scheme};

use crate::cli::report::{Kind, Step};

/// Native messaging manifests for `request`; a run that cannot be planned
/// (`--system` off Linux, no home folder) is one failed step.
pub fn manifests(request: &Request) -> Vec<Step> {
    match websign_registration::run(request) {
        Ok(report) => report
            .results
            .into_iter()
            .map(|(target, outcome)| Step {
                kind: Kind::Manifest,
                location: target.location.describe(),
                target: target.label,
                outcome,
            })
            .collect(),
        Err(error) => vec![Step {
            kind: Kind::Plan,
            target: "native messaging".to_owned(),
            location: String::new(),
            outcome: Outcome::Failed(error.to_string()),
        }],
    }
}

/// A request for every browser (or `browsers`) of this user or the system.
pub fn request(action: Action, system: bool, dry_run: bool) -> Request {
    Request {
        action,
        scope: if system { Scope::System } else { Scope::User },
        browsers: Vec::new(),
        user_data_dir: None,
        extension_ids: Vec::new(),
        manifest_dir: None,
        host: None,
        dry_run,
    }
}

/// The `websign:` URL handler (per user).
pub fn url_scheme(action: Action, dry_run: bool) -> Step {
    let outcome = match action {
        Action::Install => match websign_registration::host_binary() {
            Ok(exe) => url_scheme::register(&exe, dry_run),
            Err(error) => Outcome::Failed(error.to_string()),
        },
        Action::Uninstall => url_scheme::unregister(dry_run),
    };
    Step {
        kind: Kind::UrlScheme,
        target: format!("{}: URLs", websign_project::URL_SCHEME),
        location: String::new(),
        outcome,
    }
}

/// Windows extension pre-registration (nothing elsewhere, or before the
/// extension is in a store).
pub fn preregistration(action: Action, dry_run: bool) -> Vec<Step> {
    websign_registration::preregister::apply(action == Action::Uninstall, dry_run)
        .into_iter()
        .map(|(what, outcome)| Step {
            kind: Kind::Extension,
            target: "extension pre-registration".to_owned(),
            location: what,
            outcome,
        })
        .collect()
}
