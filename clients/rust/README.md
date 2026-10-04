# websign-client

Call the SignLocal app from a Rust program: the person picks a certificate in
the app's window, your closure supplies the digest, the app signs it with the
key (PIN included). Starts `websign connect` on demand; the window names
your program. Same flow and options as the web SDK (`@websign/sdk`) and the
Node client (`@websign/desktop`).

## Quickstart

```toml
[dependencies]
websign-client = "0.1"
```

```rust,no_run
use websign_client::{Client, ClientError, ErrorCode, HashName, SignOptions};

fn main() -> Result<(), ClientError> {
    let mut client = Client::connect()?; // ClientError::AppMissing if not installed
    let signed = client.sign(SignOptions::new(HashName::Sha256), |_certificate, _context| {
        // Runs once the person picked a certificate (again if they switch).
        // For CMS/PAdES, hash the signed attributes built from
        // `_certificate.der` and `_context.algorithm` here.
        Ok(vec![0; 32]) // your document's digest, 32 bytes for SHA-256
    });
    match signed {
        Ok(result) => println!("{:?}: {} bytes", result.algorithm, result.signature.as_bytes().len()),
        Err(error) if error.code() == ErrorCode::UserCancelled => println!("cancelled"),
        Err(error) => return Err(error),
    }
    Ok(()) // dropping `client` ends the app
}
```

## Test without the app

The `testing` feature adds `FakeApp`, an in-process stand-in that speaks the
real protocol, so your code runs unchanged:

```toml
[dev-dependencies]
websign-client = { version = "0.1", features = ["testing"] }
```

```rust,ignore
use websign_client::testing::FakeApp;
use websign_client::{ErrorCode, HashName, SignOptions};

#[test]
fn a_blocked_pin_is_reported() -> Result<(), websign_client::ClientError> {
    let app = FakeApp::builder().fail_at_confirm(ErrorCode::PinLocked).build();
    let mut client = app.connect()?;
    let error = client
        .sign(SignOptions::new(HashName::Sha256), |_, _| Ok(vec![0; 32]))
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::PinLocked);
    println!("{}", error.hint());
    Ok(())
}
```

It answers `sign` with the digest itself as the signature: it never verifies,
so do not use it to test signature validation. `app.requests()` lists what
your code sent. Builder: `certificate(..)`, `remembered(..)`,
`signature(..)`, `fail_at_choose(code)`, `fail_at_confirm(code)`.

## Errors

`ClientError` implements `std::error::Error`. Branch on `error.code()` (the
protocol's stable `ErrorCode`), print `error.hint()` for what to do and
`error.docs_url()` for the explanation; `source()` gives the underlying
I/O or protocol failure of `AppMissing` and `Connection`.

## Good to know

- `SignOptions::new(hash)` accepts the app's default algorithm;
  `.algorithm(a)` / `.algorithms([a, b])` restrict it (preferred first), and
  `.certificate(&chosen)` / `.fingerprint(fp)` preselect a certificate from
  an earlier `certificates(&[])`.
- `prepare` gets the certificate and a `PrepareContext { hash, algorithm }`
  and returns exactly 32/48/64 bytes for SHA-256/384/512. `Err(String)`, a
  wrong length, or an app that asks for another hash or an algorithm outside
  your set cancels the request in the app (`Aborted` / `InvalidRequest`).
- Blocking API over `std::process`; `Client` is `Send`. `WEBSIGN_EXECUTABLE`
  or `ConnectOptions::executable` pins the binary, otherwise
  `find_executable()` searches `PATH` and the per-OS install locations.
- Certificates are the protocol crate's type: `der`/`chain` are decoded
  bytes (`as_bytes()`), `not_before`/`not_after` are Unix seconds.

## Development

- API guide: [`docs/architecture/desktop-api.md`](../../docs/architecture/desktop-api.md) §7.
- Contract: [`SPEC.md`](SPEC.md). Example: [`examples/desktop/rust-cli`](../../examples/desktop/rust-cli).
- License: **Apache-2.0** ([`LICENSE`](LICENSE)). Runtime dependencies: the
  Apache-2.0 crates `websign-protocol` (the wire contract) and
  `websign-project` (the product and executable names), plus `thiserror`.

```sh
cargo test -p websign-client
```

The tests run against `examples/fake_websign`, a fake `websign connect`
(one scenario per misbehavior), and against the public `FakeApp`
(`cargo test -p websign-client --all-features`); no installed app is needed.
