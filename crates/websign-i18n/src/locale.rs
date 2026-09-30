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
        let _ = tag;
        todo!("SPEC.md §2")
    }

    /// The locale of the running system (`sys-locale`), or English.
    pub fn from_system() -> Locale {
        todo!("SPEC.md §2")
    }

    /// Where a missing message is looked up, this locale first and English
    /// last: `pt-BR → pt-PT → en`, `pt-PT → pt-BR → en`, others `→ en`.
    pub fn fallback_chain(self) -> &'static [Locale] {
        todo!("SPEC.md §2")
    }
}
