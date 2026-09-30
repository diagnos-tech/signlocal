//! Buttons (`docs/ux.md` §4.7, §11.3): primary, secondary and ghost, in the
//! three control heights, with the Sign button's arming.
//!
//! **Arming.** An unarmed button looks disabled and ignores the pointer and
//! the keyboard; when it becomes armed it fades to enabled over
//! `motion-base`. The 600 ms clock and the "press after arming" rule live in
//! `websign-ui-model` (`Arming`); this widget only shows the state it is
//! given and never reports a click while unarmed.
//!
//! **Enter.** A focused button clicks on Space or a pointer click, never on
//! Enter ([`super::keys`]): [`Button::show`] reports Enter apart, and the
//! window forwards it only where `docs/ux.md` §4.9 lets Enter act (Sign).

use egui::{
    CornerRadius, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget, WidgetInfo, WidgetType,
};

mod look;

pub use look::{Kind, Size};

use super::{focus, keys, spinner, text};
use crate::ui::icons::Icon;
use crate::ui::theme::{self, metrics, motion, typography};

/// A button. Build it, then `ui.add(button)`.
#[derive(Debug, Clone)]
pub struct Button<'a> {
    label: &'a str,
    kind: Kind,
    size: Size,
    icon: Option<Icon>,
    enabled: bool,
    armed: bool,
    busy: bool,
}

impl<'a> Button<'a> {
    pub fn new(kind: Kind, label: &'a str) -> Self {
        Button {
            label,
            kind,
            size: Size::Medium,
            icon: None,
            enabled: true,
            armed: true,
            busy: false,
        }
    }

    pub fn primary(label: &'a str) -> Self {
        Button::new(Kind::Primary, label).size(Size::Large)
    }

    pub fn secondary(label: &'a str) -> Self {
        Button::new(Kind::Secondary, label)
    }

    pub fn ghost(label: &'a str) -> Self {
        Button::new(Kind::Ghost, label)
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// A leading icon (§12: 16 px next to text).
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Whether the arming delay has passed (default `true`).
    pub fn armed(mut self, armed: bool) -> Self {
        self.armed = armed;
        self
    }

    /// Shows a spinner before the label ("Signing…") and ignores input.
    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }
}

/// What happened to a button this frame.
#[derive(Debug)]
pub struct ButtonResponse {
    /// `clicked()` means a pointer click, Space or an assistive-technology
    /// action on an armed, enabled, idle button; never Enter.
    pub response: Response,
    /// Enter was pressed while the button had focus and could act.
    pub enter: bool,
}

impl Widget for Button<'_> {
    /// The response alone: Enter on this button does nothing.
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).response
    }
}

impl Button<'_> {
    pub fn show(self, ui: &mut Ui) -> ButtonResponse {
        let c = theme::colors(ui.ctx());
        let label = text::whole(ui, typography::BUTTON.rich(self.label));
        let lead = if self.busy {
            Some(metrics::ICON_SM)
        } else {
            self.icon.map(|_| metrics::ICON_SM)
        };
        let lead_width = lead.map_or(0.0, |size| size + metrics::SPACE_2);
        let min_width = match self.kind {
            Kind::Primary => metrics::PRIMARY_MIN_WIDTH,
            _ => metrics::CONTROL_MD,
        };
        let width = (label.size().x + lead_width + 2.0 * look::padding(self.kind, self.size))
            .max(min_width);
        let interactive = self.enabled && self.armed && !self.busy;
        let sense = if interactive {
            Sense::click()
        } else if self.busy {
            // Keeps keyboard focus where it was while "Signing…" runs.
            Sense::focusable_noninteractive()
        } else {
            Sense::hover()
        };
        let size = Vec2::new(width, look::height(self.size));
        let (id, space) = ui.allocate_space(size);
        let rect = ui.layout().align_size_within_rect(size, space);
        let enter = interactive && keys::take_enter(ui, id);
        let response = ui.interact(rect, id, sense);
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, interactive, self.label));

        let shown = motion::animate(
            ui.ctx(),
            response.id.with("armed"),
            self.enabled && self.armed,
            motion::MOTION_BASE,
        );
        let opacity = look::DISABLED_OPACITY + (1.0 - look::DISABLED_OPACITY) * shown;
        let (fill, border, ink) = look::colors(self.kind, &c, &response);
        let radius = CornerRadius::same(metrics::RADIUS_MD);
        let painter = ui.painter();
        painter.rect(
            rect,
            radius,
            fill.gamma_multiply(opacity),
            Stroke::new(1.0, border.gamma_multiply(opacity)),
            StrokeKind::Inside,
        );
        let ink = ink.gamma_multiply(opacity);
        let content = Vec2::new(label.size().x + lead_width, label.size().y);
        let mut cursor = rect.center() - content / 2.0;
        if self.busy {
            let spot = egui::Rect::from_min_size(
                egui::pos2(cursor.x, rect.center().y - metrics::ICON_SM / 2.0),
                Vec2::splat(metrics::ICON_SM),
            );
            spinner::paint(ui, spot, ink);
        } else if let Some(icon) = self.icon {
            let glyph = text::whole(ui, icon.rich(metrics::ICON_SM, ink));
            let y = rect.center().y - glyph.size().y / 2.0;
            painter.galley(egui::pos2(cursor.x, y), glyph, ink);
        }
        cursor.x += lead_width;
        painter.galley(cursor, label, ink);
        focus::ring(ui, &response, rect, metrics::RADIUS_MD);
        ButtonResponse { response, enter }
    }
}
