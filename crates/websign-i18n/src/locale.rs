//! The supported locales and how one is picked.

/// A shipped locale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locale {
    En,
    PtBr,
    PtPt,
    Es,
    Fr,
    It,
    De,
}

impl Locale {
    /// Every locale, reference first.
    pub const ALL: [Locale; 7] = [
        Locale::En,
        Locale::PtBr,
        Locale::PtPt,
        Locale::Es,
        Locale::Fr,
        Locale::It,
        Locale::De,
    ];

    /// BCP 47 tag, also the file name: `"pt-BR"`.
    pub const fn tag(self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::PtBr => "pt-BR",
            Locale::PtPt => "pt-PT",
            Locale::Es => "es",
            Locale::Fr => "fr",
            Locale::It => "it",
            Locale::De => "de",
        }
    }

    /// The closest shipped locale for an OS or browser locale string
    /// (`"pt_BR.UTF-8"`, `"pt-br"`, `"es-MX"`, `"fr_CA"`, `"de-AT"`); `None`
    /// when the language is not shipped. Bare `"pt"` → `PtBr` (most users).
    pub fn match_tag(tag: &str) -> Option<Locale> {
        let end = tag.find(['.', '@']).unwrap_or(tag.len());
        let tag = tag[..end].replace('_', "-").to_ascii_lowercase();
        let mut parts = tag.split('-');
        let language = parts.next().unwrap_or_default();
        match language {
            "en" => Some(Locale::En),
            "pt" => match parts.next() {
                None | Some("br") => Some(Locale::PtBr),
                Some(_) => Some(Locale::PtPt),
            },
            "es" => Some(Locale::Es),
            "fr" => Some(Locale::Fr),
            "it" => Some(Locale::It),
            "de" => Some(Locale::De),
            _ => None,
        }
    }

    /// The locale of the running system (`sys-locale`), or English.
    pub fn from_system() -> Locale {
        sys_locale::get_locales()
            .find_map(|tag| Locale::match_tag(&tag))
            .unwrap_or(Locale::En)
    }

    /// Where a missing message is looked up, this locale first and English
    /// last: `pt-BR → pt-PT → en`, `pt-PT → pt-BR → en`, others `→ en`.
    pub fn fallback_chain(self) -> &'static [Locale] {
        match self {
            Locale::En => &[Locale::En],
            Locale::PtBr => &[Locale::PtBr, Locale::PtPt, Locale::En],
            Locale::PtPt => &[Locale::PtPt, Locale::PtBr, Locale::En],
            Locale::Es => &[Locale::Es, Locale::En],
            Locale::Fr => &[Locale::Fr, Locale::En],
            Locale::It => &[Locale::It, Locale::En],
            Locale::De => &[Locale::De, Locale::En],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_tags() {
        let cases = [
            ("en_GB.UTF-8", Some(Locale::En)),
            ("pt", Some(Locale::PtBr)),
            ("pt_br", Some(Locale::PtBr)),
            ("pt_PT@euro", Some(Locale::PtPt)),
            ("pt-AO", Some(Locale::PtPt)),
            ("es-MX", Some(Locale::Es)),
            ("fr_BE", Some(Locale::Fr)),
            ("it-CH", Some(Locale::It)),
            ("de_CH", Some(Locale::De)),
            ("zh-CN", None),
            ("C", None),
            ("POSIX", None),
            ("", None),
        ];
        for (tag, expected) in cases {
            assert_eq!(Locale::match_tag(tag), expected, "{tag}");
        }
    }

    #[test]
    fn chains_start_with_self_and_end_in_english() {
        for locale in Locale::ALL {
            let chain = locale.fallback_chain();
            assert_eq!(chain[0], locale);
            assert_eq!(chain.last(), Some(&Locale::En));
        }
    }
}
