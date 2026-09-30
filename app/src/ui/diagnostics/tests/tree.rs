//! The whole AccessKit tree of every tab (`docs/ux.md` §14), on the
//! mockups' scenario and on a first run: each node a screen reader reaches
//! is named in words and placed on screen.

use egui_kittest::kittest::Queryable as _;
use websign_protocol::messages::DiagnosticsTab;

use super::fixture;
use super::support::{Setup, open};
use crate::ui::audit;
use crate::ui::diagnostics::state::TABS;

#[test]
fn every_tab_reads_as_words() {
    for (scenario, facts) in [
        ("mockups", fixture::facts()),
        ("first run", fixture::first_run()),
    ] {
        for tab in TABS {
            let mut window = open(Setup {
                facts: Some(facts.clone()),
                tab: Some(tab),
                height: 1600.0,
                ..Setup::default()
            });
            if tab == DiagnosticsTab::Certificates {
                window.window().state.hidden_open = true;
                window.harness.run();
            }
            let problems = audit::problems(window.harness.query_all_by(|_| true));
            assert!(problems.is_empty(), "{scenario}, {tab:?}: {problems:#?}");
        }
    }
}
