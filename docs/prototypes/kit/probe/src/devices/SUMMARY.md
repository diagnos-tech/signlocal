# probe/src/devices

USB devices and smart card readers, described but never opened.

- `format/` — plain-text and Markdown rendering of the device snapshot
- `mod.rs` — the device snapshot and its data types
- `pcsc.rs` — smart card readers and the ATR of the card in each, through PC/SC, without connecting to the card
- `usb.rs` — USB enumeration with `nusb`: IDs and descriptor strings only
