//! CLDR cardinal plural categories for integers (SPEC §5).

use websign_i18n::Locale;
use websign_i18n::plural::{PluralCategory as C, category};

const NO_MANY: [Locale; 2] = [Locale::En, Locale::De];
const WITH_MANY: [Locale; 5] = [
    Locale::PtBr,
    Locale::PtPt,
    Locale::Es,
    Locale::Fr,
    Locale::It,
];
const ZERO_IS_ONE: [Locale; 2] = [Locale::PtBr, Locale::Fr];

#[test]
fn spec_vectors() {
    assert_eq!(category(Locale::En, 0), C::Other);
    assert_eq!(category(Locale::En, 1), C::One);
    assert_eq!(category(Locale::En, 2), C::Other);
    assert_eq!(category(Locale::PtBr, 0), C::One);
    assert_eq!(category(Locale::PtBr, 1), C::One);
    assert_eq!(category(Locale::PtBr, 2), C::Other);
    assert_eq!(category(Locale::PtBr, 1_000_000), C::Many);
    assert_eq!(category(Locale::Fr, 1), C::One);
    assert_eq!(category(Locale::Fr, 2), C::Other);
    assert_eq!(category(Locale::De, 1_000_000), C::Other);
    assert_eq!(category(Locale::Es, 1_000_000), C::Many);
}

#[test]
fn one_is_exactly_one_for_en_de_it_es_pt_pt() {
    for l in [Locale::En, Locale::De, Locale::It, Locale::Es, Locale::PtPt] {
        assert_eq!(category(l, 0), C::Other, "{l:?} 0");
        assert_eq!(category(l, 1), C::One, "{l:?} 1");
        assert_eq!(category(l, 2), C::Other, "{l:?} 2");
    }
}

#[test]
fn zero_and_one_are_one_for_pt_br_and_fr() {
    for l in ZERO_IS_ONE {
        assert_eq!(category(l, 0), C::One, "{l:?} 0");
        assert_eq!(category(l, 1), C::One, "{l:?} 1");
        assert_eq!(category(l, 2), C::Other, "{l:?} 2");
        assert_eq!(category(l, 21), C::Other, "{l:?} 21");
    }
}

#[test]
fn many_is_a_nonzero_multiple_of_a_million() {
    for l in WITH_MANY {
        for n in [
            1_000_000,
            2_000_000,
            5_000_000,
            10_000_000,
            1_000_000_000_000,
        ] {
            assert_eq!(category(l, n), C::Many, "{l:?} {n}");
        }
        for n in [999_999, 1_000_001, 1_500_000, 100_000, 1_000] {
            assert_eq!(category(l, n), C::Other, "{l:?} {n}");
        }
    }
}

#[test]
fn zero_is_not_many() {
    for l in WITH_MANY {
        assert_ne!(category(l, 0), C::Many, "{l:?}");
    }
}

#[test]
fn en_and_de_never_have_many() {
    for l in NO_MANY {
        for n in [0, 1, 2, 1_000_000, 2_000_000, 1_000_000_000_000] {
            assert_ne!(category(l, n), C::Many, "{l:?} {n}");
        }
    }
}

#[test]
fn only_one_many_and_other_occur_for_integers() {
    for l in Locale::ALL {
        for n in (0..=200).chain([1_000, 1_000_000, 123_456_789]) {
            let c = category(l, n);
            assert!(matches!(c, C::One | C::Many | C::Other), "{l:?} {n}: {c:?}");
        }
    }
}

#[test]
fn negatives_use_the_absolute_value() {
    for l in Locale::ALL {
        for n in [0_i64, 1, 2, 5, 21, 1_000_000, 3_000_000, 999_999] {
            assert_eq!(category(l, -n), category(l, n), "{l:?} {n}");
        }
    }
    assert_eq!(category(Locale::En, -1), C::One);
    assert_eq!(category(Locale::Es, -1_000_000), C::Many);
}

#[test]
fn extreme_values_do_not_panic() {
    // |i64::MIN| = 9223372036854775808, which is 775808 mod 1_000_000.
    for l in Locale::ALL {
        assert_eq!(category(l, i64::MIN), C::Other, "{l:?}");
        assert_eq!(category(l, i64::MAX), C::Other, "{l:?}");
    }
}

#[test]
fn pt_pt_differs_from_pt_br_only_at_zero() {
    assert_eq!(category(Locale::PtPt, 0), C::Other);
    assert_eq!(category(Locale::PtBr, 0), C::One);
    for n in [1, 2, 1_000_000] {
        assert_eq!(category(Locale::PtPt, n), category(Locale::PtBr, n), "{n}");
    }
}
