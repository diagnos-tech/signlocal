//! Signs the SHA-256 of a file with the person's certificate.
//!
//! `cargo run -- <file>` uses the fake app (runs anywhere, CI included);
//! `cargo run -- <file> --app` uses the installed SignLocal app.

use std::process::ExitCode;
use std::{env, fs};

use sha2::{Digest, Sha256};
use websign_client::testing::FakeApp;
use websign_client::{Client, ClientError, ErrorCode, HashName, SignOptions};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let (path, real_app) = match args.as_slice() {
        [path] => (path, false),
        [path, flag] if flag == "--app" => (path, true),
        _ => {
            eprintln!("usage: websign-rust-cli-example <file> [--app]");
            return ExitCode::from(2);
        }
    };
    let document = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("cannot read {path}: {error}");
            return ExitCode::from(1);
        }
    };
    match sign(&document, real_app) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => report(&error),
    }
}

fn sign(document: &[u8], real_app: bool) -> Result<(), ClientError> {
    // Keep the fake alive as long as the client; the real app needs neither.
    let fake = FakeApp::new();
    let mut client = if real_app {
        Client::connect()?
    } else {
        fake.connect()?
    };
    let signed = client.sign(
        SignOptions::new(HashName::Sha256),
        |_certificate, _context| {
            // The app calls this once the person picked a certificate.
            Ok(Sha256::digest(document).to_vec())
        },
    )?;
    println!(
        "signed by {} with {:?}",
        signed.certificate.display_name, signed.algorithm
    );
    let hex: String = signed
        .signature
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    println!("{hex}");
    Ok(())
}

/// Code, message, hint and docs link; a cancelled signature exits with 3.
fn report(error: &ClientError) -> ExitCode {
    eprintln!("{:?}: {error}", error.code());
    eprintln!("{}", error.hint());
    eprintln!("{}", error.docs_url());
    if error.code() == ErrorCode::UserCancelled {
        ExitCode::from(3)
    } else {
        ExitCode::from(1)
    }
}
