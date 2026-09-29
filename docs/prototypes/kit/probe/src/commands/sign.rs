//! `websign-probe sign`: signs random digests and verifies every signature.

use std::process::ExitCode;

use anyhow::{Context, bail};
use probe_core::{HashAlgorithm, PublicKeyKind, SignatureAlgorithm};
use secrecy::SecretString;

use super::inventory::{Entry, Inventory};
use super::view;
use crate::keystores::{Options, PinPrompt, SignRequest};

/// Arguments of `websign-probe sign`.
#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(flatten)]
    pub options: Options,
    /// Fingerprint prefix (hex, at least 4 digits) of a certificate to use (repeatable).
    #[arg(long = "cert", value_name = "FINGERPRINT")]
    pub certs: Vec<String>,
    /// Use every certificate that can sign.
    #[arg(long, conflicts_with = "certs")]
    pub all: bool,
    /// Also sign through every alternate path (e.g. both CNG and PKCS#11).
    #[arg(long)]
    pub every_path: bool,
    /// Hash algorithms to test.
    #[arg(long = "hash", value_enum, default_value = "sha256")]
    pub hashes: Vec<HashChoice>,
    /// Try RSASSA-PSS too on RSA keys (ECDSA keys always use ECDSA).
    #[arg(long)]
    pub pss: bool,
    /// Read the PKCS#11 PIN from this environment variable instead of asking.
    #[arg(long, value_name = "VAR")]
    pub pin_env: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum HashChoice {
    Sha256,
    Sha384,
    Sha512,
    All,
}

pub fn run(args: &Args) -> anyhow::Result<ExitCode> {
    let mut inventory = Inventory::collect(&args.options);
    for failure in &inventory.opened.failures {
        eprintln!("warning: {}: {}", failure.source, failure.error);
    }
    let selected = select(&inventory, args)?;
    let mut pin: Option<SecretString> = None;
    let mut failures = 0;

    for index in selected {
        let entry = &inventory.entries[index];
        let Ok(info) = &entry.info else { continue };
        println!("{}", view::describe(entry, true));
        if matches!(
            entry.key.pin,
            PinPrompt::App {
                protected_path: false
            }
        ) && pin.is_none()
        {
            pin = Some(read_pin(args.pin_env.as_deref())?);
        }
        for (hash, algorithm) in cases(&info.key, args) {
            let digest = random_digest(hash)?;
            let request = SignRequest {
                hash,
                algorithm,
                digest: &digest,
                pin: pin.as_ref(),
                parent_window: None,
            };
            let key = entry.key.clone();
            let store = &mut inventory.opened.keystores[entry.store];
            let outcome = store
                .sign(&key, &request)
                .map_err(anyhow::Error::from)
                .and_then(|signature| {
                    probe_core::verify(&key.cert_der, hash, algorithm, &digest, &signature.bytes)
                        .map(|()| signature)
                        .context("signature does not verify")
                });
            match outcome {
                Ok(signature) => println!(
                    "   OK    {hash} {algorithm} via {} in {} ms",
                    signature.api,
                    signature.elapsed.as_millis()
                ),
                Err(error) => {
                    failures += 1;
                    println!("   FAIL  {hash} {algorithm}: {error:#}");
                }
            }
        }
    }
    Ok(if failures == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

/// Indices into `inventory.entries` to sign with.
fn select(inventory: &Inventory, args: &Args) -> anyhow::Result<Vec<usize>> {
    let position = |target: &Entry| {
        inventory
            .entries
            .iter()
            .position(|entry| std::ptr::eq(entry, target))
    };
    let mut chosen = Vec::new();
    for group in inventory.deduped() {
        let Ok(info) = &group.primary.info else {
            continue;
        };
        let hex = info.fingerprint.to_hex();
        let wanted = if args.all {
            info.can_sign()
        } else {
            args.certs
                .iter()
                .any(|prefix| prefix.len() >= 4 && hex.starts_with(&prefix.to_lowercase()))
        };
        if !wanted {
            continue;
        }
        chosen.extend(position(group.primary));
        if args.every_path {
            chosen.extend(group.alternates.iter().filter_map(|entry| position(entry)));
        }
    }
    if chosen.is_empty() {
        bail!("no certificate selected: pass --all or --cert <fingerprint prefix> (see `list`)");
    }
    Ok(chosen)
}

/// Every (hash, algorithm) pair to try on a key.
fn cases(key: &PublicKeyKind, args: &Args) -> Vec<(HashAlgorithm, SignatureAlgorithm)> {
    let hashes: Vec<HashAlgorithm> = if args.hashes.contains(&HashChoice::All) {
        HashAlgorithm::ALL.to_vec()
    } else {
        args.hashes
            .iter()
            .map(|choice| match choice {
                HashChoice::Sha256 | HashChoice::All => HashAlgorithm::Sha256,
                HashChoice::Sha384 => HashAlgorithm::Sha384,
                HashChoice::Sha512 => HashAlgorithm::Sha512,
            })
            .collect()
    };
    let algorithms: Vec<SignatureAlgorithm> = match key {
        PublicKeyKind::Ec { .. } => vec![SignatureAlgorithm::Ecdsa],
        PublicKeyKind::Rsa { .. } if args.pss => {
            vec![SignatureAlgorithm::RsaPkcs1v15, SignatureAlgorithm::RsaPss]
        }
        PublicKeyKind::Rsa { .. } => vec![SignatureAlgorithm::RsaPkcs1v15],
        PublicKeyKind::Unsupported { .. } => Vec::new(),
    };
    hashes
        .iter()
        .flat_map(|&hash| algorithms.iter().map(move |&alg| (hash, alg)))
        .collect()
}

fn random_digest(hash: HashAlgorithm) -> anyhow::Result<Vec<u8>> {
    let mut digest = vec![0; hash.digest_len()];
    getrandom::fill(&mut digest).map_err(|error| anyhow::anyhow!("no randomness: {error}"))?;
    Ok(digest)
}

/// The PIN never touches logs or output; it lives in a zeroizing buffer.
fn read_pin(from_env: Option<&str>) -> anyhow::Result<SecretString> {
    let pin = match from_env {
        Some(var) => {
            std::env::var(var).with_context(|| format!("environment variable {var} is not set"))?
        }
        None => rpassword::prompt_password("PKCS#11 PIN: ").context("cannot read the PIN")?,
    };
    Ok(SecretString::from(pin))
}
