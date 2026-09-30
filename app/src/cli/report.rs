//! The report of `install`, `uninstall` and `register`: one step per place
//! written or removed, as text or as stable JSON.
//!
//! JSON shape (keys never renamed; new keys may be added):
//! `{"command":"install","scope":"user","dryRun":false,"ok":true,
//!   "steps":[{"kind":"manifest","target":"Google Chrome",
//!   "location":"/home/…/dev.websign.host.json","outcome":"written"}]}`
//! with `"reason"` on `skipped` and `failed` steps. Outcomes: `written`,
//! `removed`, `notPresent`, `skipped`, `dryRun`, `failed`.

use std::process::ExitCode;

use serde_json::{Value, json};
use websign_registration::Outcome;

use super::output::{FAILURE, SUCCESS, print_json};

/// What a step touched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A native messaging manifest (file or registry key).
    Manifest,
    /// The `websign:` URL handler.
    UrlScheme,
    /// Windows extension pre-registration.
    Extension,
    /// The Linux app-menu entry.
    MenuEntry,
    /// Settings, remembered sites and logs (`--purge`).
    Data,
    /// The run could not be planned at all.
    Plan,
}

impl Kind {
    const fn as_str(self) -> &'static str {
        match self {
            Kind::Manifest => "manifest",
            Kind::UrlScheme => "urlScheme",
            Kind::Extension => "extension",
            Kind::MenuEntry => "menuEntry",
            Kind::Data => "data",
            Kind::Plan => "plan",
        }
    }
}

/// One place and what happened there.
#[derive(Debug)]
pub struct Step {
    pub kind: Kind,
    pub target: String,
    pub location: String,
    pub outcome: Outcome,
}

/// A whole run.
#[derive(Debug)]
pub struct Report {
    pub command: &'static str,
    pub system: bool,
    pub dry_run: bool,
    pub steps: Vec<Step>,
}

impl Report {
    /// Whether every step succeeded (skips are not failures).
    pub fn ok(&self) -> bool {
        !self
            .steps
            .iter()
            .any(|s| matches!(s.outcome, Outcome::Failed(_)))
    }

    /// The JSON document.
    pub fn to_json(&self) -> Value {
        let steps: Vec<Value> = self.steps.iter().map(step_json).collect();
        json!({
            "command": self.command,
            "scope": if self.system { "system" } else { "user" },
            "dryRun": self.dry_run,
            "ok": self.ok(),
            "steps": steps,
        })
    }

    /// One line per step, for people.
    pub fn to_text(&self) -> String {
        self.steps
            .iter()
            .map(|step| {
                let (outcome, reason) = outcome_parts(&step.outcome);
                let reason = reason.map(|r| format!(": {r}")).unwrap_or_default();
                format!(
                    "{:<11} {} — {}{reason}\n",
                    outcome, step.target, step.location
                )
            })
            .collect()
    }

    /// Prints the report and returns 0, or 1 when a step failed.
    pub fn print(&self, json: bool) -> ExitCode {
        if json {
            print_json(&self.to_json().to_string());
        } else {
            print!("{}", self.to_text());
        }
        ExitCode::from(if self.ok() { SUCCESS } else { FAILURE })
    }
}

fn step_json(step: &Step) -> Value {
    let (outcome, reason) = outcome_parts(&step.outcome);
    let mut value = json!({
        "kind": step.kind.as_str(),
        "target": step.target,
        "location": step.location,
        "outcome": outcome,
    });
    if let (Some(reason), Some(map)) = (reason, value.as_object_mut()) {
        map.insert("reason".to_owned(), Value::from(reason));
    }
    value
}

fn outcome_parts(outcome: &Outcome) -> (&'static str, Option<&str>) {
    match outcome {
        Outcome::Written => ("written", None),
        Outcome::Removed => ("removed", None),
        Outcome::NotPresent => ("notPresent", None),
        Outcome::Skipped(reason) => ("skipped", Some(reason)),
        Outcome::DryRun => ("dryRun", None),
        Outcome::Failed(reason) => ("failed", Some(reason)),
    }
}
