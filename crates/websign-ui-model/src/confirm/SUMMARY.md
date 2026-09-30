# crates/websign-ui-model/src/confirm

- `arming.rs` — The anti-accident arming of the primary button (`docs/ux.md` §4.7).
- `cancel.rs` — Which error a caller receives when the person closes the window (`docs/ux.md` §15): the code of the last blocking condition on screen (`NoCertificates`, `PinLocked`, `CertificateUnavailable`), else `UserCancelled`.
- `machine.rs` — The confirmation window's state machine (`docs/ux.md` §4.8).
- `mod.rs` — The confirmation window: its contract with the host ([`port`]) and its state machine ([`machine`], `docs/ux.md` §4.8), with the anti-accident rules ([`arming`], §4.7) and the cancel-code rule ([`cancel`], §15).
- `port.rs` — The contract between the host engine and the confirmation window.
- `view.rs` — A description of the confirmation window at one instant: everything the egui renderer needs, nothing it must decide.
