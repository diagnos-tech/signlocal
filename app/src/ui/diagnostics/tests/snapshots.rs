//! Every tab in light and dark, on the mockups' scenario, at the top and
//! scrolled to the end (the last row must be reachable, not cut).

use websign_protocol::messages::DiagnosticsTab;

use super::support::{Setup, THEMES, open, snapshot};

fn tab_snapshot(tab: DiagnosticsTab, name: &str, prepare: impl Fn(&mut super::support::Window)) {
    for (dark, theme) in THEMES {
        let mut window = open(Setup {
            dark,
            tab: Some(tab),
            ..Setup::default()
        });
        prepare(&mut window);
        window.harness.run();
        snapshot(&mut window.harness, &format!("{name}-{theme}"));
        scroll_to_end(&mut window);
        snapshot(&mut window.harness, &format!("{name}-end-{theme}"));
    }
}

/// Wheels the content far down, past its end.
fn scroll_to_end(window: &mut super::support::Window) {
    let over_content = egui::pos2(480.0, 300.0);
    let input = window.harness.input_mut();
    input.events.push(egui::Event::PointerMoved(over_content));
    input.events.push(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(0.0, -10_000.0),
        phase: egui::TouchPhase::Move,
        modifiers: egui::Modifiers::NONE,
    });
    // Wheel scrolling is animated: let it settle.
    window.harness.run_steps(60);
    // No cursor in the picture.
    window
        .harness
        .input_mut()
        .events
        .push(egui::Event::PointerGone);
    window.harness.run();
}

#[test]
fn browsers() {
    tab_snapshot(DiagnosticsTab::Browsers, "browsers", |_| {});
}

/// The other tabs with "Getting started" hidden, to show more of them.
fn hide_strip(window: &mut super::support::Window) {
    window.window().settings.onboarding_dismissed = true;
}

#[test]
fn devices() {
    tab_snapshot(DiagnosticsTab::Devices, "devices", hide_strip);
}

#[test]
fn certificates() {
    tab_snapshot(DiagnosticsTab::Certificates, "certificates", |window| {
        hide_strip(window);
        window.window().state.hidden_open = true;
    });
}

#[test]
fn help() {
    tab_snapshot(DiagnosticsTab::Help, "help", hide_strip);
}

/// A first run: what is missing, with the fixes; and the ready state is
/// on the Browsers baselines (the mockups' scenario can sign).
#[test]
fn getting_started() {
    for (dark, theme) in THEMES {
        let mut window = open(Setup {
            dark,
            facts: Some(super::fixture::first_run()),
            tab: Some(DiagnosticsTab::Browsers),
            ..Setup::default()
        });
        window.harness.run();
        snapshot(&mut window.harness, &format!("getting-started-{theme}"));
    }
}

#[test]
fn first_scan_running() {
    for (dark, theme) in THEMES {
        let mut window = open(Setup {
            dark,
            facts: None,
            ..Setup::default()
        });
        snapshot(&mut window.harness, &format!("loading-{theme}"));
        assert_eq!(*window.scans.borrow(), 1, "the first frame starts a scan");
    }
}
