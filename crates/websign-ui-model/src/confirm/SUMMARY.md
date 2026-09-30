# crates/websign-ui-model/src/confirm

- `arming.rs` — The anti-accident arming of the primary button (`docs/ux.md` §4.7).
- `build.rs` — Describing the window at one instant (`docs/ux.md` §4.2–§4.11).
- `cancel.rs` — Which error a caller receives when the person closes the window (`docs/ux.md` §15): the code of the last blocking condition on screen (`NoCertificates`, `PinLocked`, `CertificateUnavailable`), else `UserCancelled`.
- `commands.rs` — Host commands: what the window does when the engine speaks.
- `helpers.rs` — Small questions the transitions ask of the model.
- `inputs.rs` — What the person does: clicks, keys, selection, closing.
- `machine.rs` — The confirmation window's state machine (`docs/ux.md` §4.8).
- `machine/` — Unit tests of the state machine with a scripted clock.
- `mod.rs` — The confirmation window: its contract with the host ([`port`]) and its state machine ([`machine`], `docs/ux.md` §4.8), with the anti-accident rules ([`arming`], §4.7) and the cancel-code rule ([`cancel`], §15).
- `outcome.rs` — How a request ends: failures the person can act on, and results.
- `pin.rs` — The PIN area of the window (`docs/ux.md` §4.6).
- `port.rs` — The contract between the host engine and the confirmation window.
- `slot.rs` — What the code card is waiting for.
- `timers.rs` — Time-driven changes: result holds, arming, countdown, skeleton delay.
- `view.rs` — A description of the confirmation window at one instant: everything the egui renderer needs, nothing it must decide.
