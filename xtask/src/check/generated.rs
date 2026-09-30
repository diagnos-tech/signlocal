//! Committed generated files equal what `cargo xtask gen` produces now.

use std::path::Path;

use crate::generate;

pub fn check(root: &Path) -> Result<Vec<String>, String> {
    let mut problems = generate::plan(root, None)?.differences(root)?;
    if !problems.is_empty() {
        problems.push("run `cargo xtask gen` and commit the result".to_owned());
    }
    Ok(problems)
}
