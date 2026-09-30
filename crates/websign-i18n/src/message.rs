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
        let _ = (f, self.template, &self.args);
        todo!("SPEC.md §4")
    }
}
