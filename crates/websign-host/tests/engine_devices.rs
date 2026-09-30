//! SPEC §8.13: a reader or card event while the window is open refreshes the
//! list without losing the selection.

mod common;

use common::harness::Harness;
use common::{Cert, ORIGIN};
use websign_core::HashAlgorithm;
use websign_devices::monitor::DeviceEvent;
use websign_host::Control;
use websign_ui_model::confirm::UiEvent;

const SHA256: HashAlgorithm = HashAlgorithm::Sha256;

fn card_inserted() -> DeviceEvent {
    DeviceEvent::CardInserted {
        reader: "Reader 0".to_owned(),
        atr: None,
    }
}

#[test]
fn a_device_event_while_selecting_invalidates_relists_and_refreshes_the_window() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut h = Harness::native_ready();
    h.begin("s1", ORIGIN, SHA256);
    h.listed(&[&p]);

    assert_eq!(h.device(card_inserted()), Control::Continue);
    let out = h.take();
    assert_eq!(out.key_kinds(), ["invalidate", "list"], "in this order");
    assert_eq!(out.lists(), [true], "a real refresh, not the cache");
    assert!(out.frames.is_empty());

    let out = h.listed(&[&p, &r]);
    assert_eq!(out.certificates_count(), 1, "the window gets the new list");
    assert!(out.frames.is_empty());
}

#[test]
fn the_selection_survives_a_refresh() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut h = Harness::native_ready();
    let key = h.begin("s1", ORIGIN, SHA256).opens()[0].key;
    h.listed(&[&p, &r]);
    h.ui(UiEvent::Selected {
        key,
        fingerprint: r.fingerprint,
    });
    h.take();

    h.device(card_inserted());
    h.take();
    h.listed(&[&p, &r]);

    // Continue for the earlier selection still releases that certificate.
    let out = h.ui_out(UiEvent::Continue {
        key,
        fingerprint: r.fingerprint,
    });
    let needs = out.need_digests();
    assert_eq!(needs.len(), 1);
    assert_eq!(needs[0].1.certificate.fingerprint.as_str(), r.hex());
    assert_eq!(needs[0].1.seq, 1, "a refresh is not a new question");
}

#[test]
fn every_kind_of_device_event_refreshes_and_a_removal_forgets_pins() {
    let p = Cert::p256();
    let reader = || "R".to_owned();
    let events = [
        (DeviceEvent::ReaderAdded { reader: reader() }, false),
        (DeviceEvent::ServiceChanged { running: true }, false),
        (DeviceEvent::ReaderRemoved { reader: reader() }, true),
        (DeviceEvent::CardRemoved { reader: reader() }, true),
    ];
    for (event, removal) in events {
        let mut h = Harness::native_ready();
        h.begin("s1", ORIGIN, SHA256);
        h.listed(&[&p]);
        h.device(event);
        // D5: a PIN is cached until its token leaves; which token left is
        // unknown here, so every cached login ends.
        let expected: &[&str] = if removal {
            &["end", "invalidate", "list"]
        } else {
            &["invalidate", "list"]
        };
        assert_eq!(h.take().key_kinds(), expected);
    }
}

#[test]
fn a_device_event_with_nothing_on_screen_is_ignored() {
    let mut h = Harness::native_ready();
    assert_eq!(h.device(card_inserted()), Control::Continue);
    assert!(h.take().is_silent(), "the next request lists afresh anyway");
}

#[test]
fn a_refresh_that_finds_nothing_does_not_release_or_end_anything() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.begin("s1", ORIGIN, SHA256);
    h.listed(&[&p]);
    h.device(card_inserted());
    h.take();
    let out = h.listed(&[]);
    assert!(out.frames.is_empty());
}
