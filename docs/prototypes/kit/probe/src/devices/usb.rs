//! USB enumeration with `nusb`: IDs and descriptor strings only.
//!
//! Descriptor strings are what the OS already cached; nothing is opened, so
//! no permission is needed (on Linux they come from sysfs). The serial number
//! is deliberately never read.

use serde::Serialize;

/// USB interface class of smart card readers and CCID tokens.
pub const CLASS_SMART_CARD: u8 = 0x0B;

/// One USB device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UsbDevice {
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    /// Device class (when not 0, "per interface") and every interface class, sorted.
    pub classes: Vec<u8>,
    /// Exposes a CCID (smart card) interface.
    pub smart_card: bool,
}

impl UsbDevice {
    fn new(
        (vendor_id, product_id): (u16, u16),
        manufacturer: Option<&str>,
        product: Option<&str>,
        device_class: u8,
        interface_classes: impl IntoIterator<Item = u8>,
    ) -> Self {
        let mut classes: Vec<u8> = interface_classes.into_iter().collect();
        if device_class != 0 {
            classes.push(device_class);
        }
        classes.sort_unstable();
        classes.dedup();
        let text = |value: Option<&str>| {
            value
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        };
        UsbDevice {
            vendor_id,
            product_id,
            manufacturer: text(manufacturer),
            product: text(product),
            smart_card: classes.contains(&CLASS_SMART_CARD),
            classes,
        }
    }

    /// `0529:0620`, lowercase, as `lsusb` prints it.
    pub fn id(&self) -> String {
        format!("{:04x}:{:04x}", self.vendor_id, self.product_id)
    }
}

/// The devices to show, how many were left out, and why nothing was listed if
/// enumeration failed.
#[derive(Debug, Serialize)]
pub struct UsbScan {
    pub devices: Vec<UsbDevice>,
    /// Devices without a smart card interface that were not listed.
    pub hidden: usize,
    pub problem: Option<String>,
}

/// Lists the USB devices; only smart card readers and tokens unless `all`.
pub fn scan(all: bool) -> UsbScan {
    match enumerate() {
        Ok(devices) => select(devices, all),
        Err(problem) => UsbScan {
            devices: Vec::new(),
            hidden: 0,
            problem: Some(problem),
        },
    }
}

fn select(mut devices: Vec<UsbDevice>, all: bool) -> UsbScan {
    devices.sort_by_key(|device| (!device.smart_card, device.vendor_id, device.product_id));
    let total = devices.len();
    if !all {
        devices.retain(|device| device.smart_card);
    }
    UsbScan {
        hidden: total - devices.len(),
        devices,
        problem: None,
    }
}

fn enumerate() -> Result<Vec<UsbDevice>, String> {
    use nusb::MaybeFuture as _;
    let devices = nusb::list_devices()
        .wait()
        .map_err(|error| format!("USB enumeration failed: {error}"))?;
    Ok(devices
        .map(|info| {
            UsbDevice::new(
                (info.vendor_id(), info.product_id()),
                info.manufacturer_string(),
                info.product_string(),
                info.class(),
                info.interfaces().map(|interface| interface.class()),
            )
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device(ids: (u16, u16), interface_classes: &[u8]) -> UsbDevice {
        UsbDevice::new(
            ids,
            Some("Maker"),
            Some("Model"),
            0,
            interface_classes.iter().copied(),
        )
    }

    #[test]
    fn classes_are_merged_sorted_and_deduplicated() {
        let device = UsbDevice::new((1, 2), None, None, 0xEF, [0x0B, 0x03, 0x0B]);
        assert_eq!(device.classes, [0x03, 0x0B, 0xEF]);
        assert!(device.smart_card);
        assert!(!UsbDevice::new((1, 2), None, None, 0, [0x03]).smart_card);
    }

    #[test]
    fn a_class_zero_device_class_means_per_interface_and_is_not_listed() {
        assert_eq!(
            UsbDevice::new((1, 2), None, None, 0, [0x0B]).classes,
            [0x0B]
        );
    }

    #[test]
    fn blank_strings_become_none_and_ids_print_like_lsusb() {
        let device = UsbDevice::new(
            (0x0529, 0x0620),
            Some("  "),
            Some(" eToken 5110 "),
            0,
            [0x0B],
        );
        assert_eq!(device.manufacturer, None);
        assert_eq!(device.product.as_deref(), Some("eToken 5110"));
        assert_eq!(device.id(), "0529:0620");
    }

    #[test]
    fn by_default_only_smart_card_devices_are_listed_and_the_rest_counted() {
        let devices = vec![
            device((0x046d, 0xc52b), &[0x03]),
            device((0x0529, 0x0620), &[0x0B]),
            device((0x1d6b, 0x0002), &[0x09]),
        ];
        let scan = select(devices.clone(), false);
        assert_eq!(scan.devices.len(), 1);
        assert_eq!(scan.devices[0].id(), "0529:0620");
        assert_eq!(scan.hidden, 2);

        let all = select(devices, true);
        assert_eq!((all.devices.len(), all.hidden), (3, 0));
        assert!(all.devices[0].smart_card, "smart card devices come first");
    }
}
