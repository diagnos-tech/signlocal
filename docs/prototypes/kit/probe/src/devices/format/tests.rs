use super::super::pcsc::ReaderScan;
use super::super::usb::UsbScan;
use super::*;

fn token() -> UsbDevice {
    UsbDevice {
        vendor_id: 0x0529,
        product_id: 0x0620,
        manufacturer: Some("SafeNet".into()),
        product: Some("eToken 5110".into()),
        classes: vec![0x0B],
        smart_card: true,
    }
}

fn reader(card: CardState, atr: Option<&str>) -> Reader {
    Reader {
        name: "Reader 00 00".into(),
        card,
        in_use: false,
        atr: atr.map(str::to_owned),
    }
}

fn snapshot() -> Snapshot {
    Snapshot {
        usb: UsbScan {
            devices: vec![token()],
            hidden: 3,
            problem: None,
        },
        readers: ReaderScan {
            readers: vec![reader(CardState::Present, Some("3B8F8001"))],
            problem: None,
        },
    }
}

#[test]
fn hex_is_uppercase_without_separators() {
    assert_eq!(hex_upper(&[0x3B, 0x0A, 0xFF]), "3B0AFF");
    assert_eq!(hex_upper(&[]), "");
}

#[test]
fn text_lists_devices_readers_atr_and_the_hidden_count() {
    let text = text(&snapshot());
    assert!(
        text.contains("0529:0620  CCID           SafeNet eToken 5110"),
        "{text}"
    );
    assert!(text.contains("3 other USB device(s) not shown"), "{text}");
    assert!(text.contains("Reader 00 00: card present"), "{text}");
    assert!(text.contains("ATR 3B8F8001"), "{text}");
}

#[test]
fn markdown_starts_with_the_section_heading_and_has_both_tables() {
    let markdown = markdown(&snapshot());
    assert!(markdown.starts_with("### Devices\n"));
    assert!(
        markdown.contains("| `0529:0620` | CCID | SafeNet eToken 5110 |"),
        "{markdown}"
    );
    assert!(
        markdown.contains("| Reader 00 00 | card present | `3B8F8001` |"),
        "{markdown}"
    );
    assert!(markdown.ends_with('\n'));
}

#[test]
fn problems_replace_the_tables_instead_of_failing() {
    let snapshot = Snapshot {
        usb: UsbScan {
            devices: vec![],
            hidden: 0,
            problem: Some("USB enumeration failed: denied".into()),
        },
        readers: ReaderScan {
            readers: vec![],
            problem: Some("the PC/SC service is not running (start pcscd)".into()),
        },
    };
    let text = text(&snapshot);
    assert!(
        text.contains("USB enumeration failed: denied") && text.contains("start pcscd"),
        "{text}"
    );
    assert!(markdown(&snapshot).contains("_the PC/SC service is not running (start pcscd)_"));
}

#[test]
fn empty_machines_say_none_found_and_unnamed_devices_are_still_listed() {
    let mut snapshot = Snapshot {
        usb: UsbScan {
            devices: vec![],
            hidden: 0,
            problem: None,
        },
        readers: ReaderScan {
            readers: vec![],
            problem: None,
        },
    };
    assert_eq!(text(&snapshot).matches("none found").count(), 2);
    snapshot.usb.devices.push(UsbDevice {
        manufacturer: None,
        product: None,
        classes: vec![0x0B, 0xFF],
        ..token()
    });
    assert!(
        text(&snapshot).contains("CCID+vendor"),
        "{}",
        text(&snapshot)
    );
    assert!(text(&snapshot).contains("(unnamed)"));
}

#[test]
fn json_never_carries_a_serial_number() {
    let json = serde_json::to_string(&snapshot()).unwrap();
    assert!(!json.to_lowercase().contains("serial"), "{json}");
    assert!(json.contains("\"vendor_id\":1321"));
}
