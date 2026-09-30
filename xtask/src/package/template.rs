//! `{{key}}` substitution for the packaging templates. Deliberately tiny and
//! strict: an unknown key is an error, so renaming the product in
//! `project.toml` can never leave a stale placeholder in a shipped file.

/// Replaces every `{{key}}` in `template` with its value.
pub fn render(template: &str, values: &[(&str, &str)]) -> Result<String, String> {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let end = after
            .find("}}")
            .ok_or_else(|| "unterminated `{{` in a template".to_owned())?;
        let key = after[..end].trim();
        let value = values
            .iter()
            .find(|(name, _)| *name == key)
            .ok_or_else(|| format!("template uses unknown key `{key}`"))?
            .1;
        out.push_str(value);
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_replaced_everywhere() {
        let text = render("{{a}}-{{ b }}-{{a}}", &[("a", "1"), ("b", "2")]).unwrap();
        assert_eq!(text, "1-2-1");
    }

    #[test]
    fn unknown_and_unterminated_keys_fail() {
        assert!(render("{{x}}", &[]).is_err());
        assert!(render("{{x", &[("x", "1")]).is_err());
    }

    #[test]
    fn substituted_values_are_not_expanded_again() {
        assert_eq!(render("{{a}}", &[("a", "{{a}}")]).unwrap(), "{{a}}");
    }
}
