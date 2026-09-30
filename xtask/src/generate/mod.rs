//! `cargo xtask gen`: every derived file, from one description each.
//!
//! * `ts` — `websign-protocol` types (ts-rs) into `sdk/`, `extension/` and
//!   `clients/node/` under `src/generated/`, with an `index.ts` barrel;
//! * `locales` — `i18n/*.toml` `[popup]`, `[store]`, `[extension]` into
//!   `extension/public/_locales/<xx_YY>/messages.json`;
//! * `messages` — `i18n/*.toml` `[site.errors]` into `sdk/src/messages.gen.ts`;
//! * `project` — `project.toml` into `project.ts` of the extension and the
//!   Node client (IDs, `MIN_APP_VERSION`), and into the Rust constants of
//!   `crates/websign-project`;
//! * `limits` — `websign-protocol` version, message sources, lengths and
//!   timers into `extension/src/shared/limits.gen.ts`.
//!
//! The generators only build a [`Plan`]; `gen` writes it and
//! `check generated` compares it, so both always agree.

mod limits;
mod locales;
mod messages;
mod project;
mod project_rust;
mod text;
mod ts;

use std::path::Path;

use crate::plan::Plan;
use crate::root::repo_root;

/// Generators that `--only` accepts.
const GENERATORS: [&str; 5] = ["ts", "locales", "messages", "project", "limits"];

/// `cargo xtask gen`.
#[derive(Debug, clap::Args)]
pub struct GenArgs {
    /// Only this generator: `ts`, `locales`, `messages`, `project`, `limits`.
    #[arg(long)]
    pub only: Option<String>,
}

/// Runs the generators.
pub fn run(args: &GenArgs) -> Result<(), String> {
    let root = repo_root()?;
    let plan = plan(&root, args.only.as_deref())?;
    plan.apply(&root)?;
    println!("gen: {} files up to date", plan.files.len());
    Ok(())
}

/// The files the selected generators (all when `only` is `None`) produce.
pub fn plan(root: &Path, only: Option<&str>) -> Result<Plan, String> {
    if let Some(name) = only.filter(|name| !GENERATORS.contains(name)) {
        return Err(format!(
            "unknown generator `{name}`; choose one of: {}",
            GENERATORS.join(", ")
        ));
    }
    let wanted = |name: &str| only.is_none_or(|only| only == name);
    let mut plan = Plan::default();
    if wanted("ts") {
        plan.merge(ts::plan(root)?);
    }
    if wanted("locales") {
        plan.merge(locales::plan(root)?);
    }
    if wanted("messages") {
        plan.merge(messages::plan(root)?);
    }
    if wanted("project") {
        plan.merge(project::plan(root)?);
        plan.merge(project_rust::plan(root)?);
    }
    if wanted("limits") {
        plan.merge(limits::plan()?);
    }
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unknown_generator_is_refused_with_the_valid_names() {
        let error = plan(Path::new("."), Some("nope")).unwrap_err();
        assert!(
            error.contains("nope") && error.contains("locales"),
            "{error}"
        );
    }
}
