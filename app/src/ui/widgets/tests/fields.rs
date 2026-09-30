use egui::accesskit::Role;
use egui::{Key, Vec2};
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use zeroize::Zeroizing;

use super::support::{THEMES, harness, snapshot};
use crate::ui::icons;
use crate::ui::widgets::pin_field::PinField;
use crate::ui::widgets::text_field::TextField;

/// The PIN field over `pin`; returns whether Enter was pressed.
fn pin_field(
    ui: &mut egui::Ui,
    pin: &mut Zeroizing<String>,
    shown: &mut bool,
    error: bool,
) -> bool {
    PinField {
        pin,
        shown,
        id: ui.next_auto_id().with("pin"),
        name: "Token PIN",
        show_label: "Show PIN",
        hide_label: "Hide PIN",
        max_chars: 8,
        error,
        enabled: true,
    }
    .show(ui)
    .submitted
}

#[test]
fn typed_pin_reaches_the_buffer_but_accesskit_sees_no_value() {
    let mut pin = Zeroizing::new(String::new());
    let mut shown = false;
    let mut submitted = false;
    let mut h = harness(Vec2::new(360.0, 80.0), false, |ui| {
        submitted |= pin_field(ui, &mut pin, &mut shown, false);
    });
    let field = h.get_by_role_and_label(Role::PasswordInput, "Token PIN");
    field.click();
    h.run();
    h.get_by_role_and_label(Role::PasswordInput, "Token PIN")
        .type_text("1234");
    h.run();
    let value = h.get_by_role(Role::PasswordInput).accesskit_node().value();
    assert_eq!(value, None, "not the PIN, not even its length");
    h.key_press(Key::Backspace);
    h.run();
    h.key_press(Key::Enter);
    h.run();
    drop(h);
    assert_eq!(pin.as_str(), "123");
    assert!(submitted);
}

#[test]
fn a_paste_never_reaches_the_pin() {
    let mut pin = Zeroizing::new(String::new());
    let mut shown = false;
    let mut h = harness(Vec2::new(360.0, 80.0), false, |ui| {
        pin_field(ui, &mut pin, &mut shown, false);
    });
    h.get_by_role(Role::PasswordInput).click();
    h.run();
    h.event(egui::Event::Paste("1234".to_owned()));
    h.run();
    drop(h);
    assert!(pin.is_empty());
}

#[test]
fn the_eye_button_shows_and_hides() {
    let mut pin = Zeroizing::new("1234".to_owned());
    let mut shown = false;
    let mut h = harness(Vec2::new(360.0, 80.0), false, |ui| {
        pin_field(ui, &mut pin, &mut shown, false);
    });
    h.get_by_role_and_label(Role::Button, "Show PIN").click();
    h.run();
    assert!(
        h.query_by_role_and_label(Role::Button, "Hide PIN")
            .is_some()
    );
    let value = h.get_by_role(Role::PasswordInput).accesskit_node().value();
    assert_eq!(value, None, "shown on screen, never to AccessKit");
    drop(h);
    assert!(shown);
}

#[test]
fn the_filter_field_is_named() {
    let mut query = String::new();
    let h = harness(Vec2::new(360.0, 80.0), false, |ui| {
        TextField::new(&mut query, "Filter by name, ID number or issuer")
            .hint("Filter by name, ID number or issuer")
            .icon(icons::FILTER)
            .show(ui);
    });
    let node = h.get_by_role(Role::TextInput);
    assert_eq!(
        node.accesskit_node().label().as_deref(),
        Some("Filter by name, ID number or issuer")
    );
}

#[test]
fn snapshots() {
    for (dark, theme) in THEMES {
        let mut hidden = Zeroizing::new("1234".to_owned());
        let mut wrong = Zeroizing::new(String::new());
        let mut revealed = Zeroizing::new("1234".to_owned());
        let mut query = String::new();
        let mut h = harness(Vec2::new(420.0, 220.0), dark, |ui| {
            pin_field(ui, &mut hidden, &mut false, false);
            let before = ui.next_auto_id().with("pin");
            pin_field(ui, &mut wrong, &mut false, true);
            ui.memory_mut(|memory| memory.request_focus(before));
            pin_field(ui, &mut revealed, &mut true, false);
            TextField::new(&mut query, "Filter")
                .hint("Filter by name, ID number or issuer")
                .icon(icons::FILTER)
                .show(ui);
        });
        snapshot(&mut h, &format!("fields-{theme}"));
    }
}
