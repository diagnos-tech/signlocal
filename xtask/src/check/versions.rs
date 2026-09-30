//! Versions move in lockstep (`packaging-and-release.md` §Release workflow).

use std::path::Path;

use crate::fsutil::read_text;

/// `package.json` files that carry the release version.
const PACKAGES: [&str; 3] = ["sdk", "clients/node", "extension"];

/// Problems with the version inputs; `tag` is a pushed `vX.Y.Z` tag if any.
pub fn check(root: &Path, tag: Option<&str>) -> Result<Vec<String>, String> {
    let cargo: toml::Table = read_text(&root.join("Cargo.toml"))?
        .parse()
        .map_err(|e| format!("Cargo.toml is not valid TOML: {e}"))?;
    let Some(version) = cargo
        .get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("version"))
        .and_then(toml::Value::as_str)
    else {
        return Ok(vec![
            "Cargo.toml: no [workspace.package] version".to_owned(),
        ]);
    };

    let mut problems = Vec::new();
    if parse_semver(version).is_none() {
        problems.push(format!("Cargo.toml: `{version}` is not MAJOR.MINOR.PATCH"));
    }
    for package in PACKAGES {
        let path = root.join(package).join("package.json");
        let json: serde_json::Value = serde_json::from_str(&read_text(&path)?)
            .map_err(|e| format!("{package}/package.json is not valid JSON: {e}"))?;
        match json.get("version").and_then(serde_json::Value::as_str) {
            Some(found) if found == version => {}
            Some(found) => problems.push(format!(
                "{package}/package.json: version {found} differs from Cargo.toml {version}"
            )),
            None => problems.push(format!("{package}/package.json: no version")),
        }
    }
    problems.extend(min_app_version(root, version)?);
    if let Some(tag) = tag.filter(|tag| tag.strip_prefix('v') != Some(version)) {
        problems.push(format!("tag {tag} does not match version {version}"));
    }
    Ok(problems)
}

/// The extension must not require an app newer than the one being released.
fn min_app_version(root: &Path, version: &str) -> Result<Option<String>, String> {
    let project: toml::Table = read_text(&root.join("project.toml"))?
        .parse()
        .map_err(|e| format!("project.toml is not valid TOML: {e}"))?;
    let Some(min) = project
        .get("extension")
        .and_then(|e| e.get("min_app_version"))
        .and_then(toml::Value::as_str)
    else {
        return Ok(Some(
            "project.toml: no [extension] min_app_version".to_owned(),
        ));
    };
    Ok(match (parse_semver(min), parse_semver(version)) {
        (Some(min_parts), Some(current)) if min_parts > current => Some(format!(
            "project.toml: min_app_version {min} is newer than {version}"
        )),
        (None, _) => Some(format!(
            "project.toml: min_app_version `{min}` is not MAJOR.MINOR.PATCH"
        )),
        _ => None,
    })
}

fn parse_semver(text: &str) -> Option<(u64, u64, u64)> {
    let mut parts = text.split('.').map(|part| part.parse::<u64>().ok());
    let version = (parts.next()??, parts.next()??, parts.next()??);
    parts.next().is_none().then_some(version)
}

#[cfg(test)]
mod tests;
