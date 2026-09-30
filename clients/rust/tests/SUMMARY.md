# clients/rust/tests

- `common/` — links the fake app under a scenario name and connects to it.
- `connect.rs` — what `connect` reports: missing app, early exit, refusal (at our version or not), version mismatch, silence.
- `flows.rs` — status, certificates, diagnostics, decoded certificate bytes, signing, certificate switch, cancellation, `Send`/`Sync`.
- `misbehaving.rs` — bad, oversized and truncated frames, wrong ids, a digest request for another hash, exits mid-sign, kill on drop.
- `testing.rs` — the public `FakeApp` (feature `testing`): sign, filters, failures, length rule.
