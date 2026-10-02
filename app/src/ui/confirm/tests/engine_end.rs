//! The window closes once the engine ends, also while it is hidden: eframe
//! runs only the logic of a hidden window (`Context::run_logic`), and that
//! pass alone must close it, or the process outlives its connection.

use std::sync::mpsc::channel;
use std::time::Duration;

use egui::{Context, RawInput, ViewportCommand, ViewportEvent, ViewportId, ViewportInfo};
use websign_i18n::{Catalog, Locale};
use websign_protocol::types::BrowserName;
use websign_ui_model::confirm::UiCommand;
use websign_ui_model::confirm::port::Finish;

use super::fixtures::{KEY, SIGN, request, web};
use crate::ui::confirm::clock::{Clock, ManualTime};
use crate::ui::confirm::run::App;
use crate::ui::confirm::window::ConfirmWindow;
use crate::ui::theme;

/// One logic-only pass of the root viewport, as eframe runs it while the
/// OS window is hidden; returns the commands for the OS window.
fn hidden_pass(ctx: &Context, app: &mut App<'_>, close_requested: bool) -> Vec<ViewportCommand> {
    let mut input = RawInput::default();
    let info = ViewportInfo {
        occluded: Some(true),
        events: if close_requested {
            vec![ViewportEvent::Close]
        } else {
            Vec::new()
        },
        ..ViewportInfo::default()
    };
    input.viewports.insert(ViewportId::ROOT, info);
    let mut output = ctx.run_logic(&input, |ctx| app.step(ctx));
    output
        .viewport_commands
        .remove(&ViewportId::ROOT)
        .unwrap_or_default()
}

/// A window showing a request, and the hand of its clock.
fn opened(ctx: &Context) -> (ManualTime, ConfirmWindow) {
    let installed = theme::install_for_tests(ctx, false);
    let (clock, time) = Clock::manual();
    let (events, _) = channel();
    let mut window = ConfirmWindow::new(installed, Catalog::new(Locale::En), events, clock);
    let caller = web("https://app.diagnos.health", BrowserName::Chrome);
    window.apply(UiCommand::Open(request(SIGN, caller, true)));
    (time, window)
}

fn site_cancelled() -> UiCommand {
    UiCommand::Finished {
        key: KEY,
        finish: Finish::SiteCancelled,
    }
}

#[test]
fn a_hidden_window_closes_when_the_engine_ends() {
    let ctx = Context::default();
    let (time, window) = opened(&ctx);
    let (engine, inbox) = channel();
    let mut app = App::new(window, &inbox);
    assert!(hidden_pass(&ctx, &mut app, false).contains(&ViewportCommand::Visible(true)));

    engine
        .send(site_cancelled())
        .expect("the window's end is open");
    assert!(!hidden_pass(&ctx, &mut app, false).contains(&ViewportCommand::Visible(false)));
    time.advance(Duration::from_millis(1600));
    let hide = hidden_pass(&ctx, &mut app, false);
    assert!(hide.contains(&ViewportCommand::Visible(false)), "{hide:?}");

    drop(engine);
    let first = hidden_pass(&ctx, &mut app, false);
    assert!(first.contains(&ViewportCommand::Close), "{first:?}");
    let second = hidden_pass(&ctx, &mut app, true);
    assert!(
        !second.contains(&ViewportCommand::CancelClose),
        "the close the window asked for goes through: {second:?}"
    );
}

#[test]
fn a_notice_on_screen_finishes_its_hold_before_closing() {
    let ctx = Context::default();
    let (time, window) = opened(&ctx);
    let (engine, inbox) = channel();
    let mut app = App::new(window, &inbox);
    engine
        .send(site_cancelled())
        .expect("the window's end is open");
    drop(engine);

    let held = hidden_pass(&ctx, &mut app, false);
    assert!(!held.contains(&ViewportCommand::Close), "{held:?}");
    time.advance(Duration::from_millis(1600));
    let done = hidden_pass(&ctx, &mut app, false);
    assert!(done.contains(&ViewportCommand::Close), "{done:?}");
}

#[test]
fn the_close_button_cancels_until_the_engine_ends() {
    let ctx = Context::default();
    let (_, window) = opened(&ctx);
    let (engine, inbox) = channel::<UiCommand>();
    let mut app = App::new(window, &inbox);
    let open = hidden_pass(&ctx, &mut app, true);
    assert!(open.contains(&ViewportCommand::CancelClose), "{open:?}");
    drop(engine);
    let closing = hidden_pass(&ctx, &mut app, true);
    assert!(
        !closing.contains(&ViewportCommand::CancelClose),
        "{closing:?}"
    );
}
