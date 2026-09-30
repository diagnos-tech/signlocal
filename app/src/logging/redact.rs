//! The last line of defense for log privacy (`docs/architecture/security.md`
//! §Logs, threat T10).
//!
//! Every `log::` call must already carry only steps, codes, counts and
//! sizes. This filter runs over every formatted line anyway, because a
//! mistake here is permanent: logs are attached to support requests. It
//! removes what has a recognizable shape — certificates, document numbers,
//! digests, serials, e-mails, sites, IP addresses, labelled secrets and
//! names, the home folder, the login, other accounts' profile folders and
//! network share servers — and cannot recognize a bare name in free text,
//! which is why the rule for callers stays.

mod network;
mod paths;
mod runs;
mod spans;
mod structured;

use spans::Span;

/// The redaction context of this process.
#[derive(Debug, Clone, Default)]
pub struct Redactor {
    home: Option<String>,
    login: Option<String>,
}

impl Redactor {
    /// Reads the home folder and login name from the environment.
    pub fn from_env() -> Redactor {
        let var = |name: &str| std::env::var(name).ok().filter(|v| !v.is_empty());
        let home = if cfg!(windows) {
            var("USERPROFILE")
        } else {
            var("HOME")
        };
        let login = if cfg!(windows) {
            var("USERNAME")
        } else {
            var("USER").or_else(|| var("LOGNAME"))
        };
        Redactor { home, login }
    }

    /// A redactor for a given home and login (tests).
    pub fn new(home: Option<&str>, login: Option<&str>) -> Redactor {
        Redactor {
            home: home.map(str::to_owned),
            login: login.map(str::to_owned),
        }
    }

    /// `message` with everything personal replaced by a `[kind]` marker.
    ///
    /// The order matters: shapes that contain others go first (a PEM block
    /// holds Base64; a URL or an e-mail may hold digits; a DN holds names).
    pub fn apply(&self, message: &str) -> String {
        let passes: [fn(&str) -> Vec<Span>; 11] = [
            structured::pem,
            structured::urls,
            structured::emails,
            paths::profiles,
            network::addresses,
            structured::dn_attributes,
            structured::secret_values,
            structured::name_values,
            runs::base64,
            runs::hex,
            runs::digits,
        ];
        let text = paths::redact(message, self.home.as_deref(), self.login.as_deref());
        passes
            .iter()
            .fold(text, |text, pass| spans::replace(&text, &pass(&text)))
    }
}
