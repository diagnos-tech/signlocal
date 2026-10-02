//! The default build (what releases ship) has no e2e hook: no scripted
//! confirmation, no auto-confirm variables, no e2e marker. Built with
//! `--features e2e`, the same search must find the marker, which proves the
//! search looks at the binary cargo built for these tests and would catch
//! the hook if it leaked into a default build.

/// Strings only e2e code contains: the variables' prefix, the marker the
/// release job greps for, and the headless port's log line.
const NEEDLES: [&str; 3] = ["WEBSIGN_E2E_", "WEBSIGN_E2E_BUILD", "headless confirmation"];

fn contains(binary: &[u8], needle: &str) -> bool {
    binary.windows(needle.len()).any(|w| w == needle.as_bytes())
}

fn binary() -> Vec<u8> {
    std::fs::read(env!("CARGO_BIN_EXE_websign")).unwrap()
}

#[cfg(not(feature = "e2e"))]
#[test]
fn the_default_binary_contains_no_e2e_hook() {
    let binary = binary();
    for needle in NEEDLES {
        assert!(
            !contains(&binary, needle),
            "the default build contains {needle:?}"
        );
    }
}

#[cfg(feature = "e2e")]
#[test]
fn an_e2e_binary_carries_the_marker_the_check_looks_for() {
    let binary = binary();
    for needle in NEEDLES {
        assert!(contains(&binary, needle), "the e2e build lacks {needle:?}");
    }
}
