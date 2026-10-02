# crates/websign-ui-model/src/certs

- `badge.rs` — The single neutral badge of a row (`docs/ux.md` §5.3).
- `build_row.rs` — From a candidate to the row the window draws.
- `candidate.rs` — What the host knows about one certificate before it becomes a row.
- `filter.rs` — The filter field shown above long lists (`docs/ux.md` §5.13).
- `hide.rs` — Which certificates never reach the list (`docs/ux.md` §5.8).
- `location.rs` — "Where the certificate is", in lay words (`docs/ux.md` §5.7).
- `merge.rs` — Merging a fresh listing into an open window's list.
- `mod.rs` — The certificate list (`docs/ux.md` §5): which certificates appear, how each row reads, their order and the initial selection.
- `order.rs` — Building the list: hide, disable, order, select (`docs/ux.md` §5.8, §5.9, vectors §16.6).
- `order/` — Unit tests of list building, ordering and appending.
- `row.rs` — A row of the list, and the list itself.
- `row_tests.rs` — Unit tests of badge, location and filter rules.
- `status.rs` — Whether a listed certificate can sign right now.
- `text.rs` — Case- and accent-insensitive text for matching names and issuers.
- `validity.rs` — The validity line (`docs/ux.md` §5.6, vectors §16.5).
