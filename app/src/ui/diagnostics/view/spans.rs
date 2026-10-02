//! A row title made of styled runs: a name in `text-body-strong`, a dimmed
//! origin prefix, and a mono detail (`USB 0529:0620`, a version) 8 px after
//! it, as the mockups' `.t` and `.t .mono`.

use egui::text::{LayoutJob, TextFormat};

use crate::ui::theme::typography::{self, TextToken};
use crate::ui::theme::{Colors, metrics};

/// How a run looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Strong,
    /// The scheme and subdomains of an origin.
    Dim,
    /// A detail after the name, 8 px away.
    Mono,
}

/// One run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub style: Style,
}

impl Span {
    pub fn strong(text: impl Into<String>) -> Span {
        Span {
            text: text.into(),
            style: Style::Strong,
        }
    }

    pub fn dim(text: impl Into<String>) -> Span {
        Span {
            text: text.into(),
            style: Style::Dim,
        }
    }

    pub fn mono(text: impl Into<String>) -> Span {
        Span {
            text: text.into(),
            style: Style::Mono,
        }
    }
}

/// The runs as one layout job (not wrapped yet).
pub fn job(spans: &[Span], c: &Colors) -> LayoutJob {
    let mut job = LayoutJob::default();
    for span in spans {
        let (token, color, lead) = match span.style {
            Style::Strong => (typography::BODY_STRONG, c.fg, 0.0),
            Style::Dim => (typography::BODY, c.fg_subtle, 0.0),
            Style::Mono => (typography::MONO, c.fg_subtle, metrics::SPACE_2),
        };
        job.append(&span.text, lead, format(token, color, span.style));
    }
    job
}

fn format(token: TextToken, color: egui::Color32, style: Style) -> TextFormat {
    TextFormat {
        font_id: token.font_id(),
        color,
        // The mono detail sits on the name's line: same line box, smaller
        // glyphs, aligned to the baseline like CSS `align-items: baseline`.
        line_height: Some(typography::BODY_STRONG.line_height),
        valign: if style == Style::Mono {
            egui::Align::BOTTOM
        } else {
            egui::Align::Center
        },
        ..TextFormat::default()
    }
}
