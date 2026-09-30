//! The monitor against a real PC/SC service. Without `pcscd` (or with no
//! reader attached, which is the usual CI case) there is nothing to observe
//! beyond start-up and shutdown, and that is what is asserted: the monitor
//! never fails, reports a stopped service at most once, and stops promptly
//! when dropped. Plugging a reader or a card in while this runs produces
//! `ReaderAdded` / `CardInserted` events, which are only printed.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use websign_devices::monitor::{self, DeviceEvent};

#[test]
fn the_monitor_starts_reports_service_state_once_and_stops_promptly() {
    let (sender, receiver) = mpsc::channel();
    let handle = monitor::start(sender);
    std::thread::sleep(Duration::from_millis(600));

    let started = Instant::now();
    drop(handle);
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "drop must interrupt the wait"
    );

    let events: Vec<DeviceEvent> = receiver.try_iter().collect();
    let service_events = events
        .iter()
        .filter(|event| matches!(event, DeviceEvent::ServiceChanged { .. }))
        .count();
    assert!(
        service_events <= 1,
        "unexpected service flapping: {events:?}"
    );
    for event in &events {
        eprintln!("{event:?}");
    }
}

#[test]
fn a_hung_up_receiver_does_not_hang_shutdown() {
    let (sender, receiver) = mpsc::channel();
    drop(receiver);
    let started = Instant::now();
    drop(monitor::start(sender));
    assert!(started.elapsed() < Duration::from_secs(8));
}
