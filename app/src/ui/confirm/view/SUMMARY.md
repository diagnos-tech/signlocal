# app/src/ui/confirm/view

- `mod.rs` — Drawing one `ConfirmView` (§4.2): fixed header and 64 px footer, a scrolling body, and the `Action`s the person produced.
- `header.rs` — Eyebrow with the permission chip, and the header as one sentence for screen readers (§4.3, §14).
- `origin.rs` — The site with its registrable domain emphasized (subdomains cut from the left), or the program and its signer; the warnings (§4.3, §4.3.1).
- `body.rs` — What the body shows in each state of §4.8.
- `code.rs` — The verification-code slot: code, skeleton, the Continue hint (D11) or what Choose mode shares (§4.4, §4.10).
- `list.rs` — "Sign with", the filter for long lists, the rows and "Can't sign (n)" in one scroll box that shrinks so the body fits (§4.5, §5).
- `rows.rs` — One row: select by click or Space, arrows over rows that can sign, Enter only moves focus (§4.9).
- `details.rs` — "Details" of the selected certificate (§5.12).
- `possible.rs` — Devices without certificates: the card with download and rescan, and its compact row (§6.2).
- `empty.rs` — Loading (skeleton after 150 ms, slow-driver hint after 2 s) and the empty state (§4.8).
- `pin.rs` — The PIN area: our field, keypad note, unlocked chip, locked notice (§4.6).
- `fit.rs` — Keeps the body's top, the error notice, the PIN block and "Remember" on screen: the list shrinks and scrolls inside; the body scrolls only past that, on every change (§4.5, §4.6, §4.10).
- `remember.rs` — "Remember this site / program" (§4.10).
- `checkbox.rs` — The checkbox widget "Remember" uses.
- `error.rs` — The error notice with its actions and "Technical details" (§15).
- `outcome.rs` — Signed, certificate sent, site cancelled, expired (§4.8).
- `footer.rs` — The next-step hint and Cancel / primary in the platform's order; press and release reported apart (§4.7).
