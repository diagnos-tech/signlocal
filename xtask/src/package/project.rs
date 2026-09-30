//! Identifiers for the packaging templates, read from `project.toml` and the
//! workspace version so nothing here hard-codes a name.

use std::path::Path;

use crate::fsutil::read_text;

/// The `project.toml` values packaging uses.
pub struct Project {
    pub name: String,
    pub slug: String,
    pub tagline: String,
    pub homepage: String,
    pub repository: String,
    pub native_host: String,
    pub url_scheme: String,
    pub bundle_id: String,
    /// The Safari appex's bundle ID (`[ids] safari_extension_bundle_id`).
    pub safari_bundle_id: String,
}

impl Project {
    /// The `{{key}}` values every template may use, plus `version`.
    pub fn values<'a>(&'a self, version: &'a str) -> Vec<(&'static str, &'a str)> {
        vec![
            ("name", &self.name),
            ("slug", &self.slug),
            ("tagline", &self.tagline),
            ("homepage", &self.homepage),
            ("repository", &self.repository),
            ("native_host", &self.native_host),
            ("url_scheme", &self.url_scheme),
            ("bundle_id", &self.bundle_id),
            ("safari_bundle_id", &self.safari_bundle_id),
            ("version", version),
        ]
    }
}

/// Reads `project.toml` and the workspace version.
pub fn load(root: &Path) -> Result<(Project, String), String> {
    let project = parse(&read_text(&root.join("project.toml"))?)?;
    let cargo: toml::Table = read_text(&root.join("Cargo.toml"))?
        .parse()
        .map_err(|e| format!("Cargo.toml is not valid TOML: {e}"))?;
    let version = cargo
        .get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("version"))
        .and_then(toml::Value::as_str)
        .ok_or("Cargo.toml has no [workspace.package] version")?;
    Ok((project, version.to_owned()))
}

fn parse(text: &str) -> Result<Project, String> {
    let table: toml::Table = text
        .parse()
        .map_err(|e| format!("project.toml is not valid TOML: {e}"))?;
    let get = |section: &str, key: &str| -> Result<String, String> {
        table
            .get(section)
            .and_then(|s| s.get(key))
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| format!("project.toml: missing [{section}] {key}"))
    };
    Ok(Project {
        name: get("product", "name")?,
        slug: get("product", "slug")?,
        tagline: get("product", "tagline")?,
        homepage: get("product", "homepage")?,
        repository: get("product", "repository")?,
        native_host: get("ids", "native_host")?,
        url_scheme: get("ids", "url_scheme")?,
        bundle_id: get("ids", "macos_bundle_id")?,
        safari_bundle_id: get("ids", "safari_extension_bundle_id")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_repository_project_file_has_every_identifier() {
        let root = crate::root::repo_root().unwrap();
        let (project, version) = load(&root).unwrap();
        assert_eq!(project.slug, "websign");
        assert!(version.split('.').count() == 3);
    }

    #[test]
    fn a_missing_key_names_itself() {
        let error = parse("[product]\nname = \"X\"\n").err().unwrap();
        assert!(error.contains("[product] slug"), "{error}");
    }
}
