# websign-devices — specification

`usb.rs` and `pcsc.rs` are **promoted from the Phase-0 kit**
(`probe/src/devices`), reviewed with their tests: USB scan via `nusb`
(smart-card class `0x0B` unless `all`), never reading serial numbers;
PC/SC scan with ATR as uppercase hex; `anonymous_reader_name` strips the
`[interface] (serial)` tail pcsc-lite appends. The rest is NEW.

## 1. `Snapshot::scan`

`usb::scan(false)` plus devices whose VID:PID is in `devices.json` (even
without the CCID class: some tokens are HID); `pcsc::scan()`.

## 2. `monitor::start`

- A thread loops on `SCardGetStatusChange` with every current reader plus
  `\\?PnP?\Notification`, timeout 5 s (infinite wait breaks on some pcsc-lite
  versions).
- Emits `ReaderAdded/ReaderRemoved` when the reader set changes,
  `CardInserted/CardRemoved` on `SCARD_STATE_PRESENT` edges (ATR included on
  insert).
- No PC/SC service: retry every 3 s; emit `ServiceChanged{running}` on
  transitions (once at start if not running).
- Dropping `MonitorHandle` cancels (`SCardCancel`) and joins the thread; a
  closed receiver also ends it.
- Reader names are anonymized before sending.

## 3. `hints`

- `DeviceDatabase::embedded()` parses `DEVICES_JSON`; errors → `HintsError`
  (CI validates the file against `devices.schema.json`, so this should never
  happen in a release).
- `by_usb("0529:0620")`: case-insensitive exact match in `match.usb`.
- `by_atr(atr)`: pattern bytes compared pairwise; `..` matches any byte;
  lengths must be equal; first matching entry in file order.

## 4. `possible_devices` (docs/ux.md §6.1)

- USB devices from the snapshot that are CCID class or known by VID:PID,
  minus those whose hint model name matches a `linked.token_models` entry.
- Readers with a card present, minus those whose name is in `linked.readers`.
- `hint`: the database entry by VID:PID or ATR.
- `confident = true` when a hint exists and no listed certificate reports
  an unknown device (`DeviceLink` absent for some hardware key); otherwise
  `false` (shown only in diagnostics).
- Order: USB first (by VID:PID), then readers (by name).
