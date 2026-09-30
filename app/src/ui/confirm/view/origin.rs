//! Who asks (`docs/ux.md` §4.3, §4.3.1): the site with its registrable
//! domain emphasized, or the program's name with its signer, then what they
//! want and the warnings.
//!
//! The registrable domain is never cut: when the host needs more than two
//! lines, subdomains are cut from the left ("…secure.login.app.diagnos.health"),
//! because the classic phishing host hides its real domain at the end.

use std::sync::Arc;

use egui::text::{Galley, LayoutJob, TextFormat};
use egui::{Sense, Ui, WidgetInfo, WidgetType};
use websign_core::present::origin::{FormattedOrigin, OriginWarning};
use websign_i18n::{Catalog, k};
use websign_ui_model::confirm::port::CallerView;

use crate::ui::confirm::words;
use crate::ui::fonts::{self, Weight};
use crate::ui::icons::{self, Icon};
use crate::ui::theme::{self, typography};
use crate::ui::widgets::banner::Banner;
use crate::ui::widgets::text;
use crate::ui::widgets::tone::Tone;

/// Hosts longer than this use `text-title` instead of `text-headline`.
const LONG_HOST: usize = 32;
const MAX_LINES: usize = 2;
const ALERT_GAP: f32 = 10.0;

/// Draws the caller, the `asks` sentence and the warnings. `spoken` is the
/// header's sentence for screen readers, completed here with the warnings.
pub fn show(ui: &mut Ui, tr: &Catalog, caller: &CallerView, asks: &str, spoken: &str) {
    let c = theme::colors(ui.ctx());
    let mut warnings: Vec<(Icon, String)> = Vec::new();
    let galley = match caller {
        CallerView::Web { origin, top, .. } => {
            warnings.extend(origin_warning(tr, origin));
            if let Some(top) = top {
                let host = words::host(top);
                let text = tr.tr(k::CONFIRM_INSIDE_FRAME).arg("top_site", host);
                warnings.push((icons::ATTENTION, text.to_string()));
            }
            origin_galley(ui, origin)
        }
        CallerView::Desktop { label } => {
            if !label.verified {
                warnings.push((icons::ATTENTION, tr.tr(k::CALLER_UNVERIFIED).to_string()));
            }
            text::wrapped(
                ui,
                typography::HEADLINE.rich(&label.name).color(c.fg),
                ui.available_width(),
            )
        }
    };
    let full = std::iter::once(spoken.to_owned())
        .chain(warnings.iter().map(|(_, text)| text.clone()))
        .collect::<Vec<_>>()
        .join(", ");
    let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &full));
    ui.painter().galley(rect.min, galley, c.fg);

    if let CallerView::Desktop { label } = caller
        && label.verified
    {
        let signer = tr
            .tr(k::CALLER_SIGNED_BY)
            .arg("signer", &label.detail)
            .to_string();
        ui.label(typography::SMALL.rich(signer).color(c.fg_subtle));
    }
    ui.add_space(2.0);
    ui.label(typography::BODY.rich(asks).color(c.fg_muted));
    for (icon, warning) in &warnings {
        ui.add_space(ALERT_GAP);
        ui.add(Banner::new(Tone::Warning, warning).icon(*icon).compact());
    }
}

fn origin_warning(tr: &Catalog, origin: &FormattedOrigin) -> Option<(Icon, String)> {
    Some(match origin.warning? {
        OriginWarning::Idn => {
            let shown = origin.unicode.as_ref().map(|unicode| {
                tr.tr(k::ORIGIN_SHOWN_AS)
                    .arg("unicode", unicode)
                    .to_string()
            });
            let warn = tr.tr(k::ORIGIN_WARN_IDN).to_string();
            let text = match shown {
                Some(shown) => format!("{warn}\n{shown}"),
                None => warn,
            };
            (icons::ATTENTION, text)
        }
        OriginWarning::PublicIp => (icons::ATTENTION, tr.tr(k::ORIGIN_WARN_IP).to_string()),
        OriginWarning::LocalIp => (icons::ATTENTION, tr.tr(k::ORIGIN_WARN_IP_LOCAL).to_string()),
        OriginWarning::Localhost => (
            icons::SITE_LOCAL,
            tr.tr(k::ORIGIN_WARN_LOCALHOST).to_string(),
        ),
    })
}

/// The origin in at most two lines, cutting subdomains from the left.
fn origin_galley(ui: &Ui, origin: &FormattedOrigin) -> Arc<Galley> {
    let host_len = origin
        .canonical
        .split_once("://")
        .map_or(0, |(_, host)| host.chars().count());
    let token = if host_len > LONG_HOST {
        typography::TITLE
    } else {
        typography::HEADLINE
    };
    let (scheme, subdomains) = origin
        .prefix
        .split_once("://")
        .map_or(("", origin.prefix.as_str()), |(scheme, rest)| {
            (scheme, rest)
        });
    let width = ui.available_width();
    let mut cut = 0;
    loop {
        let shown = if cut == 0 {
            format!("{scheme}://{subdomains}")
        } else {
            let tail: String = subdomains.chars().skip(cut).collect();
            format!("{scheme}://…{tail}")
        };
        let job = job(ui, &token, &shown, origin, width);
        let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
        if galley.rows.len() <= MAX_LINES || cut >= subdomains.chars().count() {
            return galley;
        }
        cut += 1;
    }
}

fn job(
    ui: &Ui,
    token: &typography::TextToken,
    prefix: &str,
    origin: &FormattedOrigin,
    width: f32,
) -> LayoutJob {
    let c = theme::colors(ui.ctx());
    let dim = TextFormat {
        font_id: egui::FontId::new(token.size, fonts::family(token.face, Weight::Regular)),
        line_height: Some(token.line_height),
        color: c.fg_subtle,
        ..Default::default()
    };
    let strong = TextFormat {
        font_id: token.font_id(),
        line_height: Some(token.line_height),
        color: c.fg,
        ..Default::default()
    };
    let mut job = LayoutJob::default();
    job.wrap.max_width = width;
    job.wrap.break_anywhere = true;
    job.append(prefix, 0.0, dim.clone());
    job.append(&origin.registrable, 0.0, strong);
    if let Some(port) = origin.port {
        job.append(&format!(":{port}"), 0.0, dim);
    }
    job
}
