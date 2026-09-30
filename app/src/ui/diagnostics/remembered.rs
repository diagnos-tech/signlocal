//! Remembered sites and programs (`docs/ux.md` §8.3 "Allowed sites"), read
//! from the consent store and shown the way the person saw them when
//! signing: an origin with its registrable domain emphasized, or a program
//! by its file name.

use websign_core::present::origin::format_origin;
use websign_host::store::ConsentRecord;

/// One remembered caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remembered {
    /// The consent store's key, to revoke it.
    pub key: String,
    pub label: CallerLabel,
    /// Unix seconds.
    pub remembered_at: i64,
    pub last_used_at: i64,
}

/// How a caller is named on screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallerLabel {
    /// `https://app.` dimmed, `diagnos.health` emphasized, `:8443` dimmed.
    Site {
        prefix: String,
        registrable: String,
        suffix: String,
    },
    /// A desktop program's file name or bundle identifier.
    Program { name: String },
}

impl CallerLabel {
    /// The whole label as plain text.
    pub fn plain(&self) -> String {
        match self {
            CallerLabel::Site {
                prefix,
                registrable,
                suffix,
            } => format!("{prefix}{registrable}{suffix}"),
            CallerLabel::Program { name } => name.clone(),
        }
    }
}

/// Sites, then programs, each newest use first.
pub fn split(records: Vec<ConsentRecord>) -> (Vec<Remembered>, Vec<Remembered>) {
    let mut all: Vec<Remembered> = records.into_iter().map(remembered).collect();
    all.sort_by_key(|entry| std::cmp::Reverse(entry.last_used_at));
    all.into_iter()
        .partition(|entry| matches!(entry.label, CallerLabel::Site { .. }))
}

fn remembered(record: ConsentRecord) -> Remembered {
    Remembered {
        label: label(&record.key),
        key: record.key,
        remembered_at: record.remembered_at,
        last_used_at: record.last_used_at,
    }
}

/// The label of a consent key: `https://…` origins, `app:authenticode:…`,
/// `app:apple:<team>:<identifier>` and `path:<executable>` programs. A path
/// is shown by its file name only: the folders may carry the user's name.
fn label(key: &str) -> CallerLabel {
    if let Some(rest) = key.strip_prefix("app:authenticode:") {
        let file = rest.rsplit(':').next().unwrap_or(rest);
        return CallerLabel::Program {
            name: file.to_owned(),
        };
    }
    if let Some(rest) = key.strip_prefix("app:apple:") {
        let identifier = rest.split_once(':').map_or(rest, |(_, id)| id);
        return CallerLabel::Program {
            name: identifier.to_owned(),
        };
    }
    if let Some(path) = key.strip_prefix("path:") {
        let file = path.rsplit(['/', '\\']).next().unwrap_or(path);
        return CallerLabel::Program {
            name: file.to_owned(),
        };
    }
    match format_origin(key) {
        Ok(origin) => CallerLabel::Site {
            prefix: origin.prefix,
            registrable: origin.registrable,
            suffix: origin
                .port
                .map(|port| format!(":{port}"))
                .unwrap_or_default(),
        },
        Err(_) => CallerLabel::Program {
            name: key.to_owned(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(key: &str, last_used_at: i64) -> ConsentRecord {
        ConsentRecord {
            key: key.to_owned(),
            remembered_at: 1,
            last_used_at,
            certificates: vec!["aa".to_owned()],
        }
    }

    #[test]
    fn sites_and_programs_are_split_and_named() {
        let (sites, programs) = split(vec![
            record("https://app.diagnos.health", 5),
            record("path:/home/ana/bin/laudos", 9),
            record("app:apple:ABCDE12345:health.diagnos.laudos", 3),
            record("https://laudos.clinica.med.br", 7),
        ]);
        assert_eq!(sites[0].label.plain(), "https://laudos.clinica.med.br");
        assert_eq!(
            sites[1].label,
            CallerLabel::Site {
                prefix: "https://app.".to_owned(),
                registrable: "diagnos.health".to_owned(),
                suffix: String::new(),
            }
        );
        assert_eq!(programs[0].label.plain(), "laudos");
        assert_eq!(programs[1].label.plain(), "health.diagnos.laudos");
    }

    #[test]
    fn authenticode_programs_show_their_file() {
        let (_, programs) = split(vec![record("app:authenticode:Diagnos Ltda:laudos.exe", 1)]);
        assert_eq!(programs[0].label.plain(), "laudos.exe");
    }
}
