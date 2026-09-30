//! `cargo xtask gen`: every derived file, from one description each.
//!
//! * `ts` — `websign-protocol` types (ts-rs) into `sdk/`, `extension/` and
//!   `clients/node/` under `src/generated/`, with an `index.ts` barrel;
//! * `locales` — `i18n/*.toml` `[popup]`, `[store]`, `[extension]` into
//!   `extension/public/_locales/<xx_YY>/messages.json`;
//! * `messages` — `i18n/*.toml` `[site.errors]` into `sdk/src/messages.gen.ts`;
//! * `project` — `project.toml` into `project.ts` of the extension and the
//!   Node client (IDs, `MIN_APP_VERSION`).
//!
//! The generators only build a [`Plan`]; `gen` writes it and
//! `check generated` compares it, so both always agree.

mod locales;
mod messages;
mod project;
mod text;
mod ts;

use std::path::Path;

use crate::plan::Plan;
use crate::root::repo_root;

/// Generators that `--only` accepts.
const GENERATORS: [&str; 4] = ["ts", "locales", "messages", "project"];

/// `cargo xtask gen`.
#[derive(Debug, clap::Args)]
pub struct GenArgs {
    /// Only this generator: `ts`, `locales`, `messages`, `project`.
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
