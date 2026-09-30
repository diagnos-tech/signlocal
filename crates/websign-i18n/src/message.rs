//! A message template being filled with arguments.

use std::fmt;

/// A template plus the arguments given so far. `Display` renders it.
///
/// Placeholders are `{name}` with `name` in `[a-z_]+`. A placeholder without
/// an argument renders as itself (visible in screenshots, caught by review);
/// an argument without a placeholder is ignored.
#[derive(Debug, Clone)]
pub struct Message<'a> {
    template: &'a str,
    args: Vec<(&'static str, String)>,
}

impl<'a> Message<'a> {
    /// A message with no arguments yet.
    pub fn new(template: &'a str) -> Message<'a> {
        Message {
            template,
            args: Vec::new(),
        }
    }

    /// Fills `{name}` with `value`.
    pub fn arg(mut self, name: &'static str, value: impl fmt::Display) -> Message<'a> {
        self.args.push((name, value.to_string()));
        self
    }
}

impl fmt::Display for Message<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut rest = self.template;
        while let Some(open) = rest.find('{') {
            f.write_str(&rest[..open])?;
            rest = &rest[open..];
            let Some((name, len)) = placeholder_at_start(rest) else {
                f.write_str("{")?;
                rest = &rest[1..];
                continue;
            };
            match self.args.iter().rev().find(|(arg, _)| *arg == name) {
                Some((_, value)) => f.write_str(value)?,
                None => f.write_str(&rest[..len])?,
            }
            rest = &rest[len..];
        }
        f.write_str(rest)
    }
}

/// If `text` starts with `{name}` (`name` in `[a-z_]+`), the name and the
/// byte length of the whole placeholder.
fn placeholder_at_start(text: &str) -> Option<(&str, usize)> {
    let inner = text.strip_prefix('{')?;
    let end = inner.find(|c: char| !(c.is_ascii_lowercase() || c == '_'))?;
    if end == 0 || !inner[end..].starts_with('}') {
        return None;
    }
    Some((&inner[..end], end + 2))
}

/// The distinct placeholder names of `template`, sorted.
pub(crate) fn placeholders(template: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        rest = &rest[open..];
        match placeholder_at_start(rest) {
            Some((name, len)) => {
                names.push(name.to_owned());
                rest = &rest[len..];
            }
            None => rest = &rest[1..],
        }
    }
    names.sort();
    names.dedup();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(template: &str, args: &[(&'static str, &str)]) -> String {
        let mut message = Message::new(template);
        for (name, value) in args {
            message = message.arg(name, value);
        }
        message.to_string()
    }

    #[test]
    fn vectors() {
        assert_eq!(
            render("Sign for {site} — WebeSign", &[("site", "a.b")]),
            "Sign for a.b — WebeSign"
        );
        assert_eq!(render("{a}{a}", &[("a", "x")]), "xx");
        assert_eq!(render("{missing}", &[]), "{missing}");
        assert_eq!(render("{{x}}", &[("x", "1")]), "{1}");
        assert_eq!(render("{ x } {Name} {", &[("x", "1")]), "{ x } {Name} {");
        assert_eq!(render("{a}", &[("a", "1"), ("a", "2")]), "2");
        assert_eq!(render("plain", &[("a", "1")]), "plain");
    }

    #[test]
    fn lists_placeholders() {
        assert_eq!(placeholders("{b} {a} {b} {C} {"), ["a", "b"]);
    }
}
