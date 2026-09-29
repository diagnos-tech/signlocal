//! The `key: value` lines of one p11-kit `.module` file.

/// The keys of a `.module` file this program cares about.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ModuleFile {
    pub module: Option<String>,
    pub enable_in: Vec<String>,
    pub disable_in: Vec<String>,
    /// The module only serves the system trust store (`p11-kit-trust`).
    pub trust_policy: bool,
    /// The module runs in another process (`remote:`); there is no path to load.
    pub remote: bool,
}

impl ModuleFile {
    /// Parses the `key: value` lines; comments start with `#`.
    pub fn parse(text: &str) -> ModuleFile {
        let mut file = ModuleFile::default();
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            let value = value.trim();
            match key.trim().to_ascii_lowercase().as_str() {
                "module" if !value.is_empty() => file.module = Some(value.to_owned()),
                "enable-in" => file.enable_in = program_list(value),
                "disable-in" => file.disable_in = program_list(value),
                "trust-policy" => file.trust_policy = value.eq_ignore_ascii_case("yes"),
                "remote" => file.remote = !value.is_empty(),
                _ => {}
            }
        }
        file
    }

    /// p11-kit's own rule: `enable-in` is an allow list, `disable-in` a deny
    /// list, and both compare the program's file name. `programs` are the
    /// names this program may be known by.
    pub fn applies_to(&self, programs: &[String]) -> bool {
        let listed = |list: &[String]| list.iter().any(|name| programs.contains(name));
        (self.enable_in.is_empty() || listed(&self.enable_in)) && !listed(&self.disable_in)
    }

    /// Whether the module is worth loading to look for signing certificates.
    /// The trust module only holds CA certificates and never a private key.
    pub fn is_signing_candidate(&self) -> bool {
        !self.trust_policy && !self.remote
    }
}

fn program_list(value: &str) -> Vec<String> {
    value
        .split([',', ' ', '\t'])
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn programs(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn parses_the_keys_that_matter_and_ignores_the_rest() {
        let file = ModuleFile::parse(
            "# comment\nmodule: opensc-pkcs11.so\npriority: 1\nDisable-In: p11-kit-proxy, firefox\nx-init-reserved:\n\nnot a pair\n",
        );
        assert_eq!(file.module.as_deref(), Some("opensc-pkcs11.so"));
        assert_eq!(file.disable_in, ["p11-kit-proxy", "firefox"]);
        assert!(file.enable_in.is_empty());
        assert!(file.is_signing_candidate());
    }

    #[test]
    fn disable_in_excludes_only_the_listed_programs() {
        let file = ModuleFile::parse("module: a.so\ndisable-in: p11-kit-proxy");
        assert!(file.applies_to(&programs(&["websign"])));
        assert!(!file.applies_to(&programs(&["websign", "p11-kit-proxy"])));
    }

    #[test]
    fn enable_in_is_an_allow_list() {
        let file = ModuleFile::parse("module: a.so\nenable-in: firefox thunderbird");
        assert!(!file.applies_to(&programs(&["websign"])));
        assert!(file.applies_to(&programs(&["websign", "thunderbird"])));
    }

    #[test]
    fn trust_and_remote_modules_are_not_signing_candidates() {
        assert!(
            !ModuleFile::parse("module: p11-kit-trust.so\ntrust-policy: yes")
                .is_signing_candidate()
        );
        assert!(
            !ModuleFile::parse("remote: /usr/bin/ssh host p11-kit remote x.so")
                .is_signing_candidate()
        );
    }
}
