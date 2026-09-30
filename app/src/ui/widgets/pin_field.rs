//! The PIN field (`docs/ux.md` §4.6): the shared field frame, bullets by
//! default, an `eye`/`eye-slash` button to show what was typed (a wrong PIN
//! locks the token and costs a trip to the CA).
//!
//! Not built on egui's `TextEdit`, which keeps copies of its text in its
//! undo history. This field reads keystrokes straight into the caller's
//! `Zeroizing` buffer and wipes egui's copies ([`pin_keys`]), has no
//! clipboard and no IME, and gives AccessKit a `PasswordInput` with no value
//! at all: not the PIN, not even its length. Enter reports `submitted`; the
//! window decides whether that signs.

use egui::accesskit::Role;
use egui::{Id, Rect, Response, Sense, Ui, Vec2, WidgetInfo, WidgetType, pos2, vec2};
use zeroize::Zeroizing;

use super::{field, focus, pin_buffer, pin_keys, text};
use crate::ui::icons;
use crate::ui::theme::{self, metrics, typography};

/// `padding: 0 4px 0 10px` and the 28 px icon button of the mockups.
const PADDING_LEFT: f32 = 10.0;
const PADDING_RIGHT: f32 = 4.0;
const EYE: f32 = 28.0;
/// Bullets are drawn a size up with wide spacing, so they can be counted.
const BULLET_SIZE: f32 = 16.0;
const BULLET_SPACING: f32 = 3.0;
const DISABLED_OPACITY: f32 = 0.42;

/// The field over `pin`. `shown` is the eye toggle's state, owned by the
/// window so it survives frames and resets with the request.
#[derive(Debug)]
pub struct PinField<'a> {
    pub pin: &'a mut Zeroizing<String>,
    pub shown: &'a mut bool,
    pub id: Id,
    /// Accessible name: "Token PIN" / "Card PIN".
    pub name: &'a str,
    pub show_label: &'a str,
    pub hide_label: &'a str,
    /// `ulMaxPinLen`: typing stops there.
    pub max_chars: usize,
    pub error: bool,
    pub enabled: bool,
}

/// What happened this frame.
#[derive(Debug)]
pub struct PinFieldResponse {
    pub field: Response,
    pub changed: bool,
    /// Enter was pressed in the field.
    pub submitted: bool,
}

impl PinField<'_> {
    pub fn show(self, ui: &mut Ui) -> PinFieldResponse {
        let (rect, _) = ui.allocate_exact_size(
            vec2(ui.available_width(), metrics::CONTROL_MD),
            Sense::hover(),
        );
        let opacity = if self.enabled { 1.0 } else { DISABLED_OPACITY };
        // A child that paints over the space already allocated, so the
        // whole field (frame, text, eye) dims together when disabled.
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
        child.multiply_opacity(opacity);
        self.paint(&mut child, rect)
    }

    fn paint(self, ui: &mut Ui, rect: Rect) -> PinFieldResponse {
        field::paint(ui, rect, self.error);
        let eye_rect = Rect::from_center_size(
            pos2(rect.max.x - PADDING_RIGHT - EYE / 2.0, rect.center().y),
            Vec2::splat(EYE),
        );
        let input_rect = Rect::from_min_max(rect.min, pos2(eye_rect.min.x, rect.max.y));
        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let field = ui.interact(input_rect, self.id, sense);
        if field.clicked() {
            field.request_focus();
        }
        let typed = if self.enabled && field.has_focus() {
            let (pin, max) = (&mut *self.pin, self.max_chars);
            ui.input_mut(|input| pin_keys::read(input, pin, max))
        } else {
            pin_keys::Typed::default()
        };

        let (name, enabled, error) = (self.name, self.enabled, self.error);
        field.widget_info(|| WidgetInfo::text_edit(enabled, "", "", ""));
        ui.ctx().accesskit_node_builder(field.id, |node| {
            node.set_role(Role::PasswordInput);
            node.set_label(name);
            node.clear_value();
            if error {
                node.set_invalid(egui::accesskit::Invalid::True);
            }
        });

        let text_min = pos2(rect.min.x + PADDING_LEFT, rect.min.y);
        let end = self.paint_value(ui, text_min, rect.center().y);
        if field.has_focus() {
            let c = theme::colors(ui.ctx());
            let caret = Rect::from_center_size(pos2(end + 1.0, rect.center().y), vec2(1.5, 18.0));
            ui.painter().rect_filled(caret, 0, c.accent_fg);
        }
        field::ring(ui, &field, rect);
        self.eye(ui, eye_rect);
        PinFieldResponse {
            field,
            changed: typed.changed,
            submitted: typed.submitted,
        }
    }

    /// Paints the bullets or, when shown, the PIN; returns where it ends.
    ///
    /// A shown PIN is laid out one character at a time, so no string holding
    /// the whole PIN reaches egui's layout cache.
    fn paint_value(&self, ui: &Ui, min: egui::Pos2, middle: f32) -> f32 {
        let fg = theme::colors(ui.ctx()).fg;
        let mut x = min.x;
        if !*self.shown {
            let bullets = pin_buffer::mask(self.pin);
            let run = typography::BODY
                .rich(bullets)
                .size(BULLET_SIZE)
                .extra_letter_spacing(BULLET_SPACING)
                .color(fg);
            let galley = text::whole(ui, run);
            x += galley.size().x;
            ui.painter()
                .galley(pos2(min.x, middle - galley.size().y / 2.0), galley, fg);
            return x;
        }
        let mut buffer = [0_u8; 4];
        for ch in self.pin.chars() {
            let glyph = text::whole(ui, typography::BODY.rich(&*ch.encode_utf8(&mut buffer)));
            let size = glyph.size();
            ui.painter()
                .galley(pos2(x, middle - size.y / 2.0), glyph, fg);
            x += size.x;
        }
        buffer.fill(0);
        x
    }

    fn eye(self, ui: &mut Ui, rect: Rect) {
        let c = theme::colors(ui.ctx());
        let (icon, label) = if *self.shown {
            (icons::HIDE, self.hide_label)
        } else {
            (icons::SHOW, self.show_label)
        };
        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let button = ui.interact(rect, self.id.with("eye"), sense);
        button.widget_info(|| WidgetInfo::labeled(WidgetType::Button, self.enabled, label));
        if button.clicked() {
            *self.shown = !*self.shown;
        }
        if button.hovered() {
            ui.painter()
                .rect_filled(rect, metrics::RADIUS_MD, c.bg_hover);
        }
        let color = if button.hovered() { c.fg } else { c.fg_muted };
        let glyph = text::whole(ui, icon.rich(metrics::ICON_SM, color));
        ui.painter()
            .galley(rect.center() - glyph.size() / 2.0, glyph, color);
        focus::ring(ui, &button, rect, metrics::RADIUS_MD);
        let _ = button.on_hover_text(label);
    }
}
