//! Turns successive views of the readers into events. Pure, so the edge
//! rules are testable without a PC/SC service.

use std::collections::BTreeMap;

use super::DeviceEvent;
use crate::anonymous_reader_name;

/// One reader as PC/SC reported it (raw name, never sent out).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Observed {
    pub name: String,
    pub card_present: bool,
    pub atr: Option<String>,
}

/// Remembers the last view and reports what changed since.
#[derive(Debug)]
pub(super) struct Tracker {
    known: BTreeMap<String, Observed>,
    primed: bool,
}

impl Tracker {
    /// The first view is the baseline: readers already plugged in when the
    /// monitor starts are not "added".
    pub fn new() -> Tracker {
        Tracker {
            known: BTreeMap::new(),
            primed: false,
        }
    }

    /// After the service came back: everything present is news, because
    /// nothing could be watched meanwhile.
    pub fn after_outage() -> Tracker {
        Tracker {
            known: BTreeMap::new(),
            primed: true,
        }
    }

    pub fn observe(&mut self, current: Vec<Observed>) -> Vec<DeviceEvent> {
        let next: BTreeMap<String, Observed> = current
            .into_iter()
            .map(|reader| (reader.name.clone(), reader))
            .collect();
        let mut events = Vec::new();
        if self.primed {
            for (name, old) in &self.known {
                if !next.contains_key(name) {
                    if old.card_present {
                        events.push(card_removed(name));
                    }
                    events.push(DeviceEvent::ReaderRemoved {
                        reader: anonymous_reader_name(name),
                    });
                }
            }
            for (name, reader) in &next {
                match self.known.get(name) {
                    None => {
                        events.push(DeviceEvent::ReaderAdded {
                            reader: anonymous_reader_name(name),
                        });
                        if reader.card_present {
                            events.push(card_inserted(reader));
                        }
                    }
                    Some(old) => compare_card(old, reader, &mut events),
                }
            }
        }
        self.known = next;
        self.primed = true;
        events
    }
}

/// A different card without an empty reader in between (fast swap) counts
/// as removal plus insertion.
fn compare_card(old: &Observed, new: &Observed, events: &mut Vec<DeviceEvent>) {
    let swapped = old.card_present && new.card_present && old.atr != new.atr;
    if old.card_present && (!new.card_present || swapped) {
        events.push(card_removed(&new.name));
    }
    if new.card_present && (!old.card_present || swapped) {
        events.push(card_inserted(new));
    }
}

fn card_removed(name: &str) -> DeviceEvent {
    DeviceEvent::CardRemoved {
        reader: anonymous_reader_name(name),
    }
}

fn card_inserted(reader: &Observed) -> DeviceEvent {
    DeviceEvent::CardInserted {
        reader: anonymous_reader_name(&reader.name),
        atr: reader.atr.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reader(name: &str, atr: Option<&str>) -> Observed {
        Observed {
            name: name.to_owned(),
            card_present: atr.is_some(),
            atr: atr.map(str::to_owned),
        }
    }

    fn reader_name(name: &str) -> String {
        anonymous_reader_name(name)
    }

    #[test]
    fn the_first_view_is_a_silent_baseline() {
        let mut tracker = Tracker::new();
        assert!(
            tracker
                .observe(vec![reader("A (123) 00 00", Some("3B00"))])
                .is_empty()
        );
        assert!(
            tracker
                .observe(vec![reader("A (123) 00 00", Some("3B00"))])
                .is_empty()
        );
    }

    #[test]
    fn inserting_and_removing_a_card_are_edges_and_names_are_anonymous() {
        let mut tracker = Tracker::new();
        tracker.observe(vec![reader("A (123) 00 00", None)]);
        let inserted = tracker.observe(vec![reader("A (123) 00 00", Some("3B8F"))]);
        assert_eq!(
            inserted,
            [DeviceEvent::CardInserted {
                reader: "A 00 00".to_owned(),
                atr: Some("3B8F".to_owned())
            }]
        );
        let removed = tracker.observe(vec![reader("A (123) 00 00", None)]);
        assert_eq!(
            removed,
            [DeviceEvent::CardRemoved {
                reader: "A 00 00".to_owned()
            }]
        );
    }

    #[test]
    fn readers_appearing_and_disappearing_report_their_cards_too() {
        let mut tracker = Tracker::new();
        tracker.observe(vec![]);
        let added = tracker.observe(vec![reader("B 00 00", Some("3B"))]);
        assert_eq!(added.len(), 2);
        assert_eq!(
            added[0],
            DeviceEvent::ReaderAdded {
                reader: reader_name("B 00 00")
            }
        );
        let gone = tracker.observe(vec![]);
        assert_eq!(
            gone,
            [
                DeviceEvent::CardRemoved {
                    reader: "B 00 00".to_owned()
                },
                DeviceEvent::ReaderRemoved {
                    reader: "B 00 00".to_owned()
                },
            ]
        );
    }

    #[test]
    fn a_swapped_card_is_removal_then_insertion() {
        let mut tracker = Tracker::new();
        tracker.observe(vec![reader("A", Some("3B01"))]);
        let events = tracker.observe(vec![reader("A", Some("3B02"))]);
        assert!(matches!(events[0], DeviceEvent::CardRemoved { .. }));
        assert!(matches!(events[1], DeviceEvent::CardInserted { .. }));
        assert!(tracker.observe(vec![reader("A", Some("3B02"))]).is_empty());
    }

    #[test]
    fn after_an_outage_present_readers_are_reported() {
        let mut tracker = Tracker::after_outage();
        let events = tracker.observe(vec![reader("A", None)]);
        assert_eq!(
            events,
            [DeviceEvent::ReaderAdded {
                reader: "A".to_owned()
            }]
        );
    }
}
