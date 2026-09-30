# crates/websign-devices/src

- `hints.rs` — `devices.json`: which driver a token or card needs, per OS.
- `lib.rs` — Hardware that might hold a certificate: USB tokens and smart card readers, live reader/card events, and the hints of `devices.json`.
- `monitor.rs` — Live reader and card events, so the certificate list updates by itself.
- `pcsc.rs` — Smart card readers and the ATR of the card in each, through PC/SC.
- `possible.rs` — "Possible certificates": devices that look like a token or card but brought no certificate to the list (`docs/ux.md` §6.1).
- `usb.rs` — USB enumeration with `nusb`: IDs and descriptor strings only.
