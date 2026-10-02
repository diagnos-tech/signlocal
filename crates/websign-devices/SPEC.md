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
  versions). A service that does not know the PnP pseudo-reader
  (`UnknownReader` on a zero-timeout probe) is watched without it; the
  timeout re-list then catches reader changes.
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

`LinkedDevices { readers, token_models, unknown_links }` describes the
listed keys' devices (the host fills it from each hardware key's
`DeviceLink`).

- USB devices from the snapshot that are CCID class or known by VID:PID,
  minus those a listed key lives on: the device's product string appears
  (case-insensitively) in a `linked.readers` name — a CCID token shows up in
  PC/SC and PKCS#11 as a reader named after its USB product — or a
  `linked.token_models` entry equals the product string or appears in the
  hint's name (`CK_TOKEN_INFO.model` is `"eToken 5110"`, the commercial name
  `"SafeNet eToken 5110"`).
- Readers with a card present, minus those whose anonymized name is in
  `linked.readers`.
- `hint`: the database entry by VID:PID or ATR.
- `confident = true` when a hint exists and `linked.unknown_links` is false
  (no listed hardware key lacks a `DeviceLink`: otherwise any device might be
  the one it lives on); `false` entries are shown only in diagnostics.
- A USB device whose hint is `kind: "reader"` is never confident: a reader
  says nothing about the card in it. The card itself is judged by its ATR in
  the reader candidate, so the confirmation window never shows "possible
  certificate" for an empty or unrelated reader.
- Order: USB first (by VID:PID), then readers (by name).
