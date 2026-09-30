# websign-client

Call the WebeSign app from a Rust program: the person picks a certificate in
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
use websign_client::{Client, ClientError, ErrorCode, HashName, PrepareContext, SignOptions};

fn main() -> Result<(), ClientError> {
    let mut client = Client::connect()?; // ClientError::AppMissing if not installed
    let options = SignOptions::new(HashName::Sha256);
    let signed = client.sign(options, |certificate, context| {
        // Runs once the person picked a certificate (again if they switch).
        // For CMS/PAdES, hash the signed attributes built from
        // `certificate.der` and `context.algorithm` here.
        Ok(digest_of_my_document(certificate.der.as_bytes(), context))
    });
    match signed {
        Ok(result) => println!("{:?}: {} bytes", result.algorithm, result.signature.as_bytes().len()),
        Err(error) if error.code() == ErrorCode::UserCancelled => println!("cancelled"),
        Err(error) => return Err(error),
    }
    Ok(()) // dropping `client` ends the app
}

/// Your code: the digest to sign, 32 bytes for SHA-256.
fn digest_of_my_document(_certificate_der: &[u8], _context: PrepareContext) -> Vec<u8> {
    vec![0; 32]
}
```

## Good to know

- `SignOptions::new(hash)` accepts the app's default algorithm;
  `.algorithm(a)` / `.algorithms([a, b])` restrict it (preferred first), and
  `.certificate(&chosen)` / `.fingerprint(fp)` preselect a certificate from
  an earlier `certificates(&[])`.
- `prepare` gets the certificate and a `PrepareContext { hash, algorithm }`
  and returns exactly 32/48/64 bytes for SHA-256/384/512. `Err(String)`, a
  wrong length, or an app that asks for another hash or an algorithm outside
  your set cancels the request in the app (`Aborted` / `InvalidRequest`).
- `ClientError::code()` gives the protocol's stable `ErrorCode`; branch on
  it, not on the message.
- Blocking API over `std::process`; `Client` is `Send`. `WEBSIGN_EXECUTABLE`
  or `ConnectOptions::executable` pins the binary, otherwise
  `find_executable()` searches `PATH` and the per-OS install locations.
- Certificates are the protocol crate's type: `der`/`chain` are decoded
  bytes (`as_bytes()`), `not_before`/`not_after` are Unix seconds.

## Development

- API guide: [`docs/architecture/desktop-api.md`](../../docs/architecture/desktop-api.md) §7.
- Contract: [`SPEC.md`](SPEC.md).
- License: **Apache-2.0** ([`LICENSE`](LICENSE)). Runtime dependencies: the
  Apache-2.0 crates `websign-protocol` (the wire contract) and
  `websign-project` (the product and executable names), plus `thiserror`.

```sh
cargo test -p websign-client
```

The tests run against `examples/fake_websign`, a fake `websign connect`
(one scenario per misbehavior); no installed app is needed.
