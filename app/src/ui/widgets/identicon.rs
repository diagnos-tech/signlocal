//! The mirrored 5 × 5 identicon of a verification code (`docs/ux.md` §4.4):
//! a 40 px well with 6 px cells 1 px apart, in the code's palette color.
//!
//! The grid itself comes from `websign_protocol::code`, the same function
//! the SDK mirrors, so the site and the window always draw the same picture.
//! Decorative: the code text next to it is what screen readers read.

use egui::{CornerRadius, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget, pos2};
use websign_protocol::code::{GRID, VerificationCode};

use crate::ui::theme::{self, metrics, tokens};

const CELL: f32 = 6.0;
const GAP: f32 = 1.0;

/// Draws `code`'s identicon.
#[derive(Debug, Clone, Copy)]
pub struct Identicon<'a> {
    code: &'a VerificationCode,
}

impl<'a> Identicon<'a> {
    pub fn new(code: &'a VerificationCode) -> Self {
        Identicon { code }
    }
}

impl Widget for Identicon<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) =
            ui.allocate_exact_size(Vec2::splat(metrics::IDENTICON), Sense::hover());
        let c = theme::colors(ui.ctx());
        let painter = ui.painter();
        painter.rect(
            rect,
            CornerRadius::same(metrics::RADIUS_SM),
            c.bg_sunken,
            Stroke::new(1.0, c.border),
            StrokeKind::Inside,
        );
        let color = tokens::IDENTICON[usize::from(self.code.color_index) % tokens::IDENTICON.len()];
        let side = GRID as f32 * CELL + (GRID as f32 - 1.0) * GAP;
        let origin = rect.center() - Vec2::splat(side / 2.0);
        // A malformed grid (wrong length) draws what it has; it never panics.
        for (index, _) in self
            .code
            .cells
            .iter()
            .take(GRID * GRID)
            .enumerate()
            .filter(|(_, lit)| **lit)
        {
            let (row, column) = ((index / GRID) as f32, (index % GRID) as f32);
            let min = pos2(
                origin.x + column * (CELL + GAP),
                origin.y + row * (CELL + GAP),
            );
            painter.rect_filled(Rect::from_min_size(min, Vec2::splat(CELL)), 0, color);
        }
        response
    }
}
