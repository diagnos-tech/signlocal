//! `{name}` placeholder rendering (SPEC §4).

use websign_i18n::Message;

fn render(template: &str, args: &[(&'static str, &str)]) -> String {
    let mut m = Message::new(template);
    for (k, v) in args {
        m = m.arg(k, v);
    }
    m.to_string()
}

#[test]
fn spec_vectors() {
    assert_eq!(
        render("Sign for {site} — SignLocal", &[("site", "a.b")]),
        "Sign for a.b — SignLocal"
    );
    assert_eq!(render("{a}{a}", &[("a", "x")]), "xx");
    assert_eq!(render("{missing}", &[]), "{missing}");
    assert_eq!(render("{{x}}", &[("x", "1")]), "{1}");
}

#[test]
fn template_without_placeholders_is_unchanged() {
    assert_eq!(render("plain — text…", &[]), "plain — text…");
    assert_eq!(render("", &[]), "");
}

#[test]
fn last_argument_for_a_name_wins() {
    assert_eq!(render("{a}", &[("a", "1"), ("a", "2")]), "2");
}

#[test]
fn arguments_without_placeholder_are_ignored() {
    assert_eq!(render("hello", &[("unused", "x")]), "hello");
}

#[test]
fn several_distinct_placeholders() {
    assert_eq!(
        render("{current} of {total}", &[("total", "3"), ("current", "1")]),
        "1 of 3"
    );
}

#[test]
fn unfilled_placeholder_stays_while_others_are_filled() {
    assert_eq!(render("{a}-{b}", &[("a", "x")]), "x-{b}");
}

#[test]
fn other_braces_are_literal() {
    let args = [("x", "1"), ("Name", "n"), ("a", "z")];
    for t in [
        "{ x }", "{Name}", "{", "}", "{}", "{a-b}", "{a1}", "{x", "x}", "}{", "{ }",
    ] {
        // `{x` and `x}` contain no complete placeholder.
        assert_eq!(render(t, &args), t, "{t:?}");
    }
}

#[test]
fn underscore_names_are_placeholders() {
    assert_eq!(render("{a_b}", &[("a_b", "ok")]), "ok");
    assert_eq!(render("{_}", &[("_", "ok")]), "ok");
}

#[test]
fn values_are_inserted_verbatim_without_escaping() {
    let v = "<b>&\"'\\{x}";
    assert_eq!(render("[{v}]", &[("v", v)]), format!("[{v}]"));
}

#[test]
fn multibyte_text_around_placeholders() {
    assert_eq!(render("„{n}“ — ✓ {n}…", &[("n", "é")]), "„é“ — ✓ é…");
}

#[test]
fn any_display_value_is_accepted() {
    let s = Message::new("{n}/{f}/{b}")
        .arg("n", 42)
        .arg("f", 1.5)
        .arg("b", true)
        .to_string();
    assert_eq!(s, "42/1.5/true");
}

#[test]
fn rendering_is_repeatable() {
    let m = Message::new("{a}").arg("a", "x");
    assert_eq!(m.to_string(), m.to_string());
    assert_eq!(m.clone().to_string(), "x");
}

#[test]
fn empty_value_removes_the_placeholder() {
    assert_eq!(render("a{x}b", &[("x", "")]), "ab");
}

#[test]
fn argument_values_are_never_re_expanded() {
    assert_eq!(render("{a} {b}", &[("a", "{b}"), ("b", "x")]), "{b} x");
    assert_eq!(render("{a}", &[("a", "{a}")]), "{a}");
}
