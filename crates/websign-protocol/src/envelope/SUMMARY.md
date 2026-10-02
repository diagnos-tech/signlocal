# crates/websign-protocol/src/envelope

- `bounds.rs` — limits the types cannot express, checked after the body parses (step 7)
- `describe.rs` — names the field that broke parsing, by path, without echoing values
- `json.rs` — reads a frame into JSON, refusing repeated object keys
- `parse.rs` — the ordered envelope checks (steps 1 to 6)
- `tests.rs` — envelope parsing and round-trip tests
