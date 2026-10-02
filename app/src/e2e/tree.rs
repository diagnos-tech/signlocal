//! The window's widgets as its accessibility tree lists them: what a screen
//! reader sees is what the driver acts on, so it never needs coordinates
//! or the window's internals.

use egui::accesskit::{Action, ActionRequest, NodeId, Role, Toggled, TreeId, TreeUpdate};
use egui::{Event, RawInput};

/// One widget of the last frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Widget {
    pub id: NodeId,
    pub role: Role,
    pub label: String,
    pub enabled: bool,
    pub checked: bool,
}

/// The widgets of the last frame (egui sends the whole tree every frame).
#[derive(Debug, Default)]
pub struct Widgets {
    all: Vec<Widget>,
}

impl Widgets {
    /// Replaces the list with the widgets of `update`, if the frame had one.
    pub fn update(&mut self, update: Option<&TreeUpdate>) {
        let Some(update) = update else {
            return;
        };
        self.all = update
            .nodes
            .iter()
            .map(|(id, node)| Widget {
                id: *id,
                role: node.role(),
                label: node.label().unwrap_or_default().to_owned(),
                enabled: !node.is_disabled(),
                checked: node.toggled() == Some(Toggled::True),
            })
            .collect();
    }

    /// The first widget with `role` whose label is one of `labels`.
    pub fn find(&self, role: Role, labels: &[String]) -> Option<&Widget> {
        self.all
            .iter()
            .find(|w| w.role == role && labels.contains(&w.label))
    }

    /// The first widget with `role`.
    pub fn first(&self, role: Role) -> Option<&Widget> {
        self.all.iter().find(|w| w.role == role)
    }

    /// A fingerprint of what is on screen, to tell when it stopped changing.
    pub fn digest(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for widget in &self.all {
            (
                widget.role as u32,
                &widget.label,
                widget.enabled,
                widget.checked,
            )
                .hash(&mut hasher);
        }
        hasher.finish()
    }
}

/// Queues an accessibility `action` on `target` for the next frame, as a
/// screen reader would send it.
pub fn request(input: &mut RawInput, target: NodeId, action: Action) {
    input
        .events
        .push(Event::AccessKitActionRequest(ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node: target,
            data: None,
        }));
}
