//! The install scripts are plain files users download on their own, so they
//! cannot read `project.toml`: their identifiers are written out, and this
//! check keeps them equal to the project's.

use std::path::Path;

use crate::fsutil::read_text;
use crate::package::project::{self, Project};

pub fn check(root: &Path) -> Result<Vec<String>, String> {
    let (project, _) = project::load(root)?;
    let mut problems = Vec::new();
    for (file, lines) in expected_lines(&project)? {
        let text = read_text(&root.join(file))?;
        problems.extend(
            lines
                .iter()
                .filter(|line| !text.lines().any(|actual| actual.trim() == line.as_str()))
                .map(|line| format!("{file}: expected the line `{line}` (from project.toml)")),
        );
    }
    Ok(problems)
}

/// For each script, the exact lines that carry a project identifier.
fn expected_lines(project: &Project) -> Result<[(&'static str, Vec<String>); 2], String> {
    let repo = github_repo(&project.repository)?;
    Ok([
        (
            "scripts/install/install.sh",
            vec![
                format!("SLUG={}", project.slug),
                format!("APP_NAME={}", project.name),
                format!("BUNDLE_ID={}", project.bundle_id),
                format!("REPO=${{WEBSIGN_REPO:-{repo}}}"),
            ],
        ),
        (
            "scripts/install/install.ps1",
            vec![
                format!("$Script:Slug = '{}'", project.slug),
                format!("$Script:AppName = '{}'", project.name),
                format!(
                    "$Script:Repo = if ($env:WEBSIGN_REPO) {{ $env:WEBSIGN_REPO }} else {{ '{repo}' }}"
                ),
            ],
        ),
    ])
}

/// `owner/name` of a `https://github.com/owner/name` repository URL.
fn github_repo(url: &str) -> Result<&str, String> {
    url.strip_prefix("https://github.com/")
        .map(|rest| rest.trim_end_matches('/'))
        .filter(|repo| repo.split('/').count() == 2)
        .ok_or_else(|| format!("project.toml: repository `{url}` is not a GitHub repository URL"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempDir;

    #[test]
    fn the_repository_scripts_match_project_toml() {
        let root = crate::root::repo_root().unwrap();
        assert_eq!(check(&root).unwrap(), Vec::<String>::new());
    }

    #[test]
    fn a_stale_identifier_is_reported_with_its_file() {
        let root = crate::root::repo_root().unwrap();
        let copy = TempDir::new();
        for file in ["project.toml", "Cargo.toml", "scripts/install/install.ps1"] {
            copy.write(file, &read_text(&root.join(file)).unwrap());
        }
        let script = read_text(&root.join("scripts/install/install.sh")).unwrap();
        copy.write(
            "scripts/install/install.sh",
            &script.replace("SLUG=websign", "SLUG=oldname"),
        );
        let problems = check(copy.path()).unwrap();
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(problems[0].contains("install.sh") && problems[0].contains("SLUG=websign"));
    }

    #[test]
    fn only_github_repository_urls_are_accepted() {
        assert_eq!(github_repo("https://github.com/a/b/").unwrap(), "a/b");
        assert!(github_repo("https://gitlab.com/a/b").is_err());
        assert!(github_repo("https://github.com/a").is_err());
    }
}
