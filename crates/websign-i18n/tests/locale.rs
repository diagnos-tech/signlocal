//! Locale matching and fallback chains (SPEC §2).

use websign_i18n::Locale;

#[test]
fn matches_every_spec_row() {
    let rows: &[(&str, Locale)] = &[
        ("en", Locale::En),
        ("en-US", Locale::En),
        ("en_GB.UTF-8", Locale::En),
        ("pt-BR", Locale::PtBr),
        ("pt_br", Locale::PtBr),
        ("pt", Locale::PtBr),
        ("pt-PT", Locale::PtPt),
        ("pt_PT@euro", Locale::PtPt),
        ("pt-AO", Locale::PtPt),
        ("pt-MZ", Locale::PtPt),
        ("es", Locale::Es),
        ("es-MX", Locale::Es),
        ("es_ES", Locale::Es),
        ("fr", Locale::Fr),
        ("fr-CA", Locale::Fr),
        ("fr_BE", Locale::Fr),
        ("it", Locale::It),
        ("it-CH", Locale::It),
        ("de", Locale::De),
        ("de-AT", Locale::De),
        ("de_CH", Locale::De),
    ];
    for (tag, want) in rows {
        assert_eq!(Locale::match_tag(tag), Some(*want), "tag {tag:?}");
    }
}

#[test]
fn unsupported_or_empty_tags_do_not_match() {
    for tag in ["ja", "zh-CN", "", "C", "POSIX"] {
        assert_eq!(Locale::match_tag(tag), None, "tag {tag:?}");
    }
}

#[test]
fn matching_ignores_case_separator_encoding_and_modifier() {
    assert_eq!(Locale::match_tag("PT-br"), Some(Locale::PtBr));
    assert_eq!(Locale::match_tag("PT_PT.UTF-8"), Some(Locale::PtPt));
    assert_eq!(Locale::match_tag("DE_de.utf8@euro"), Some(Locale::De));
    assert_eq!(Locale::match_tag("fr-FR.UTF-8"), Some(Locale::Fr));
}

#[test]
fn matching_never_panics_on_odd_input() {
    for tag in [
        "-", "_", ".", "@", "-pt", "pt-", "é", "\0", "en\u{0}", "🙂", "   ",
    ] {
        let _ = Locale::match_tag(tag);
    }
}

#[test]
fn every_tag_round_trips_through_match_tag() {
    for locale in Locale::ALL {
        assert_eq!(Locale::match_tag(locale.tag()), Some(locale));
    }
}

#[test]
fn all_lists_seven_distinct_locales_reference_first() {
    assert_eq!(Locale::ALL.len(), 7);
    assert_eq!(Locale::ALL[0], Locale::En);
    for (i, a) in Locale::ALL.iter().enumerate() {
        for b in &Locale::ALL[i + 1..] {
            assert_ne!(a, b);
        }
    }
}

#[test]
fn tags_are_bcp47_file_names() {
    let tags: Vec<_> = Locale::ALL.iter().map(|l| l.tag()).collect();
    assert_eq!(tags, ["en", "pt-BR", "pt-PT", "es", "fr", "it", "de"]);
}

#[test]
fn fallback_chains_follow_the_spec() {
    use Locale::*;
    assert_eq!(En.fallback_chain(), &[En]);
    assert_eq!(PtBr.fallback_chain(), &[PtBr, PtPt, En]);
    assert_eq!(PtPt.fallback_chain(), &[PtPt, PtBr, En]);
    for l in [Es, Fr, It, De] {
        assert_eq!(l.fallback_chain(), &[l, En]);
    }
}

#[test]
fn every_chain_starts_with_self_and_ends_with_english() {
    for l in Locale::ALL {
        let chain = l.fallback_chain();
        assert_eq!(chain.first(), Some(&l));
        assert_eq!(chain.last(), Some(&Locale::En));
    }
}

#[test]
fn from_system_returns_a_shipped_locale() {
    assert!(Locale::ALL.contains(&Locale::from_system()));
}
