# crates/websign-ui-model/src/certs

- `badge.rs` — The single neutral badge of a row (`docs/ux.md` §5.3).
- `candidate.rs` — What the host knows about one certificate before it becomes a row.
- `filter.rs` — The filter field shown above long lists (`docs/ux.md` §5.13).
- `location.rs` — "Where the certificate is", in lay words (`docs/ux.md` §5.7).
- `mod.rs` — The certificate list (`docs/ux.md` §5): which certificates appear, how each row reads, their order and the initial selection.
- `order.rs` — Building the list: hide, disable, order, select (`docs/ux.md` §5.8, §5.9, vectors §16.6).
- `row.rs` — A row of the list, and the list itself.
- `validity.rs` — The validity line (`docs/ux.md` §5.6, vectors §16.5).
