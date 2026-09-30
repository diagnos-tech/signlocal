//! Busy indicator: a turning arc in the accent color.
//!
//! With reduced motion the arc stands still (the text next to it, e.g.
//! "Signing…", still says what is happening) and no repaint is requested,
//! so a waiting window costs no CPU.

use egui::{
    Color32, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, Widget, WidgetInfo, WidgetType,
};

use crate::ui::theme::{self, metrics, motion};

/// One turn per this many seconds.
const PERIOD: f32 = 1.0;
/// Share of the circle the arc covers.
const SWEEP: f32 = 0.7;

/// A standalone spinner with an accessible name ("Looking for certificates…").
#[derive(Debug, Clone)]
pub struct Spinner<'a> {
    label: &'a str,
    size: f32,
}

impl<'a> Spinner<'a> {
    pub fn new(label: &'a str) -> Self {
        Spinner {
            label,
            size: metrics::ICON_MD,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl Widget for Spinner<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(self.size), Sense::hover());
        response
            .widget_info(|| WidgetInfo::labeled(WidgetType::ProgressIndicator, true, self.label));
        paint(ui, rect, theme::colors(ui.ctx()).accent_fg);
        response
    }
}

/// Paints the arc inside `rect`; used by busy buttons too.
pub fn paint(ui: &Ui, rect: Rect, color: Color32) {
    let turn = if motion::reduced(ui.ctx()) {
        0.0
    } else {
        ui.ctx().request_repaint();
        (ui.input(|input| input.time) as f32 / PERIOD).fract()
    };
    let radius = rect.width().min(rect.height()) / 2.0 - 1.5;
    let start = turn * std::f32::consts::TAU;
    let points: Vec<_> = (0..=24)
        .map(|step| {
            let angle = start + SWEEP * std::f32::consts::TAU * step as f32 / 24.0;
            rect.center() + radius * Vec2::angled(angle)
        })
        .collect();
    ui.painter()
        .add(Shape::line(points, Stroke::new(2.0, color)));
}
