use super::*;
use crate::pcsc::{CardState, ReaderScan};
use crate::usb::UsbScan;

fn usb(vid: u16, pid: u16, classes: &[u8], product: &str) -> UsbDevice {
    UsbDevice {
        vendor_id: vid,
        product_id: pid,
        manufacturer: None,
        product: Some(product.to_owned()),
        classes: classes.to_vec(),
        smart_card: classes.contains(&0x0B),
    }
}

fn reader(name: &str, card: CardState, atr: Option<&str>) -> Reader {
    Reader {
        name: name.to_owned(),
        card,
        in_use: false,
        atr: atr.map(str::to_owned),
    }
}

fn snapshot(devices: Vec<UsbDevice>, readers: Vec<Reader>) -> Snapshot {
    Snapshot {
        usb: UsbScan {
            devices,
            hidden: 0,
            problem: None,
        },
        readers: ReaderScan {
            readers,
            problem: None,
        },
    }
}

fn db() -> DeviceDatabase {
    DeviceDatabase::embedded().expect("embedded database")
}

fn run(snapshot: &Snapshot, linked: &LinkedDevices) -> Vec<PossibleDevice> {
    possible_devices(snapshot, &db(), linked)
}

#[test]
fn a_known_token_is_confident_and_carries_its_hint() {
    let snap = snapshot(vec![usb(0x0529, 0x0620, &[0x0B], "eToken")], vec![]);
    let found = run(&snap, &LinkedDevices::default());
    assert_eq!(found.len(), 1);
    assert!(found[0].confident);
    assert_eq!(
        found[0].hint.as_ref().map(|h| h.id.as_str()),
        Some("safenet-etoken-5110")
    );
    assert_eq!(
        found[0].source,
        PossibleSource::Usb {
            vid_pid: "0529:0620".to_owned(),
            product: Some("eToken".to_owned())
        }
    );
}

#[test]
fn an_unknown_ccid_device_is_listed_without_confidence() {
    let snap = snapshot(vec![usb(0x1234, 0x5678, &[0x0B], "Mystery")], vec![]);
    let found = run(&snap, &LinkedDevices::default());
    assert_eq!(
        (found.len(), found[0].confident, found[0].hint.is_none()),
        (1, false, true)
    );
}

#[test]
fn a_known_non_ccid_device_counts_and_an_unknown_one_does_not() {
    let snap = snapshot(
        vec![
            usb(0x0529, 0x0620, &[0x03], "HID token"),
            usb(0x046d, 0xc52b, &[0x03], "Mouse"),
        ],
        vec![],
    );
    let found = run(&snap, &LinkedDevices::default());
    assert_eq!(found.len(), 1);
    assert!(found[0].confident);
}

#[test]
fn a_token_whose_model_already_brought_certificates_is_left_out() {
    let snap = snapshot(vec![usb(0x0529, 0x0620, &[0x0B], "eToken")], vec![]);
    let linked = LinkedDevices {
        token_models: vec!["safenet etoken 5110".to_owned()],
        ..LinkedDevices::default()
    };
    assert!(run(&snap, &linked).is_empty());
}

#[test]
fn a_bare_reader_is_never_confident() {
    let snap = snapshot(vec![usb(0x08e6, 0x3437, &[0x0B], "PC Twin")], vec![]);
    let found = run(&snap, &LinkedDevices::default());
    assert!(found[0].hint.is_some());
    assert!(!found[0].confident);
}

#[test]
fn only_readers_with_a_card_are_listed_and_the_atr_finds_the_hint() {
    let snap = snapshot(
        vec![],
        vec![
            reader("R1 00 00", CardState::Empty, None),
            reader(
                "R2 00 00",
                CardState::Present,
                Some("3B7D95000080318065B08311AABB83009000"),
            ),
            reader("R3 00 00", CardState::Present, Some("3B00")),
            reader("R4 00 00", CardState::Unresponsive, None),
        ],
    );
    let found = run(&snap, &LinkedDevices::default());
    assert_eq!(found.len(), 2);
    assert!(found[0].confident);
    assert_eq!(
        found[0].hint.as_ref().map(|h| h.id.as_str()),
        Some("pt-cartao-de-cidadao")
    );
    assert!(!found[1].confident && found[1].hint.is_none());
}

#[test]
fn readers_linked_to_a_certificate_are_left_out_even_with_a_serial_in_the_name() {
    let snap = snapshot(
        vec![],
        vec![reader("R 00 00", CardState::Present, Some("3B00"))],
    );
    let linked = LinkedDevices {
        readers: vec!["R (0123) 00 00".to_owned()],
        ..LinkedDevices::default()
    };
    assert!(run(&snap, &linked).is_empty());
}

#[test]
fn usb_comes_first_sorted_by_id_then_readers_sorted_by_name() {
    let snap = snapshot(
        vec![
            usb(0x1050, 0x0407, &[0x0B], "b"),
            usb(0x0529, 0x0620, &[0x0B], "a"),
        ],
        vec![
            reader("Z 00 00", CardState::Present, Some("3B00")),
            reader("A 00 00", CardState::Present, Some("3B00")),
        ],
    );
    let order: Vec<String> = run(&snap, &LinkedDevices::default())
        .iter()
        .map(|d| d.source.sort_key().to_owned())
        .collect();
    assert_eq!(order, ["0529:0620", "1050:0407", "A 00 00", "Z 00 00"]);
}

#[test]
fn a_token_whose_reader_name_carries_its_usb_product_is_left_out() {
    let snap = snapshot(vec![usb(0x0529, 0x0620, &[0x0B], "eToken 5110")], vec![]);
    let linked = LinkedDevices {
        readers: vec!["SafeNet eToken 5110 [eToken 5110] 00 00".to_owned()],
        ..LinkedDevices::default()
    };
    assert!(run(&snap, &linked).is_empty());
}

#[test]
fn a_token_model_matches_the_usb_product_or_part_of_the_hint_name() {
    let snap = snapshot(vec![usb(0x0529, 0x0620, &[0x0B], "Token JC")], vec![]);
    for model in ["token jc", "eToken 5110"] {
        let linked = LinkedDevices {
            token_models: vec![model.to_owned()],
            ..LinkedDevices::default()
        };
        assert!(run(&snap, &linked).is_empty(), "{model}");
    }
    let unrelated = LinkedDevices {
        token_models: vec!["PKCS#15 emulated".to_owned()],
        ..LinkedDevices::default()
    };
    assert_eq!(run(&snap, &unrelated).len(), 1);
}

#[test]
fn an_unknown_device_link_makes_every_hint_unconfident() {
    let snap = snapshot(
        vec![usb(0x0529, 0x0620, &[0x0B], "eToken")],
        vec![reader(
            "R2 00 00",
            CardState::Present,
            Some("3B7D95000080318065B08311AABB83009000"),
        )],
    );
    let linked = LinkedDevices {
        unknown_links: true,
        ..LinkedDevices::default()
    };
    let found = run(&snap, &linked);
    assert_eq!(found.len(), 2);
    assert!(
        found
            .iter()
            .all(|device| device.hint.is_some() && !device.confident)
    );
}
