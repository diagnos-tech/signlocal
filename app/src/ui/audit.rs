//! A walk over a window's whole AccessKit tree (`docs/ux.md` §14), for the
//! tests of every state: whatever a screen reader lands on must say what it
//! is and where it is, in words.
//!
//! A single widget test checks one name; this catches the widget someone
//! adds later without `widget_info` (no bounds: the reader cannot show or
//! scroll to it), without a name, or with an icon glyph in its name (read
//! aloud as "private use character").

use egui::accesskit::Role;
use egui_kittest::Node;
use egui_kittest::kittest::NodeT as _;

/// Roles a person reaches with Tab or a screen reader's quick keys.
const REACHED: [Role; 14] = [
    Role::Button,
    Role::Link,
    Role::RadioButton,
    Role::CheckBox,
    Role::PasswordInput,
    Role::TextInput,
    Role::Tab,
    Role::TabList,
    Role::RadioGroup,
    Role::ListItem,
    Role::Alert,
    Role::Status,
    Role::Document,
    Role::Label,
];

/// Every problem found among `nodes`, one line each; empty when none.
pub fn problems<'t>(nodes: impl Iterator<Item = Node<'t>>) -> Vec<String> {
    let mut found = Vec::new();
    for node in nodes {
        let node = node.accesskit_node();
        let role = node.role();
        let label = node.label().unwrap_or_default();
        let value = node.value().unwrap_or_default();
        if [&label, &value]
            .iter()
            .any(|text| text.chars().any(icon_glyph))
        {
            found.push(format!(
                "{role:?} {label:?} {value:?}: an icon glyph in its name"
            ));
        }
        if !REACHED.contains(&role) {
            continue;
        }
        // A label reads its text as its value; the others need a name.
        let named = if role == Role::Label {
            !label.trim().is_empty() || !value.trim().is_empty()
        } else {
            !label.trim().is_empty()
        };
        if !named {
            found.push(format!("{role:?} without a name"));
        }
        if node.bounding_box().is_none() {
            found.push(format!("{role:?} {label:?}: no bounds"));
        }
    }
    found
}

/// Phosphor draws its icons from the Unicode private use area.
fn icon_glyph(c: char) -> bool {
    ('\u{E000}'..='\u{F8FF}').contains(&c)
}
