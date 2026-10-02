//! `websign-probe sign`: signs random digests and verifies every signature.

use std::process::ExitCode;

use anyhow::{Context, bail};
use probe_core::{HashAlgorithm, PublicKeyKind, SignatureAlgorithm};
use secrecy::SecretString;
use zeroize::Zeroize as _;

use super::view;
use crate::keystores::inventory::{Entry, Inventory};
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
    for warning in inventory.warnings() {
        eprintln!("warning: {warning}");
    }
    let selected = select(&inventory, args)?;
    let results = run_cases(
        &mut inventory,
        &selected,
        args,
        |entry| println!("{}", view::describe(entry, true)),
        |result| println!("{}", result.line()),
    )?;
    let failed = results.iter().any(|result| result.outcome.is_err());
    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

/// The outcome of one (key, hash, algorithm) signing attempt.
#[derive(Debug)]
pub struct CaseResult {
    /// Index into `Inventory::entries`.
    pub entry: usize,
    pub hash: HashAlgorithm,
    pub algorithm: SignatureAlgorithm,
    /// Native API and milliseconds on success; the error chain on failure.
    pub outcome: Result<(&'static str, u128), String>,
}

impl CaseResult {
    pub fn line(&self) -> String {
        match &self.outcome {
            Ok((api, ms)) => format!(
                "   OK    {} {} via {api} in {ms} ms",
                self.hash, self.algorithm
            ),
            Err(error) => format!("   FAIL  {} {}: {error}", self.hash, self.algorithm),
        }
    }
}

/// Signs a fresh random digest for every case of every selected entry and
/// verifies each signature. `on_entry` runs before an entry's first case and
/// `on_result` after each case, so progress shows while tokens are slow.
pub fn run_cases(
    inventory: &mut Inventory,
    selected: &[usize],
    args: &Args,
    mut on_entry: impl FnMut(&Entry),
    mut on_result: impl FnMut(&CaseResult),
) -> anyhow::Result<Vec<CaseResult>> {
    let mut pin: Option<SecretString> = None;
    let mut results = Vec::new();
    for &index in selected {
        let entry = &inventory.entries[index];
        let Ok(info) = &entry.info else { continue };
        on_entry(entry);
        if matches!(
            entry.key.pin,
            PinPrompt::App {
                protected_path: false
            }
        ) && pin.is_none()
        {
            pin = Some(read_pin(args.pin_env.as_deref())?);
        }
        let key = entry.key.clone();
        let store = entry.store;
        for (hash, algorithm) in cases(&info.key, args) {
            let digest = random_digest(hash)?;
            let request = SignRequest {
                hash,
                algorithm,
                digest: &digest,
                pin: pin.as_ref(),
                parent_window: None,
            };
            let outcome = inventory.opened.keystores[store]
                .sign(&key, &request)
                .map_err(anyhow::Error::from)
                .and_then(|signature| {
                    probe_core::verify(&key.cert_der, hash, algorithm, &digest, &signature.bytes)
                        .context("signature does not verify")?;
                    Ok((signature.api, signature.elapsed.as_millis()))
                })
                .map_err(|error| format!("{error:#}"));
            let result = CaseResult {
                entry: index,
                hash,
                algorithm,
                outcome,
            };
            on_result(&result);
            results.push(result);
        }
    }
    Ok(results)
}

/// Indices into `inventory.entries` to sign with.
pub fn select(inventory: &Inventory, args: &Args) -> anyhow::Result<Vec<usize>> {
    let mut chosen = Vec::new();
    for group in inventory.groups() {
        let Ok(info) = &inventory.entries[group.primary].info else {
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
        chosen.push(group.primary);
        if args.every_path {
            chosen.extend(group.alternates);
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
    let mut pin = match from_env {
        Some(var) => {
            std::env::var(var).with_context(|| format!("environment variable {var} is not set"))?
        }
        None => rpassword::prompt_password("PKCS#11 PIN: ").context("cannot read the PIN")?,
    };
    // `SecretString::from(String)` may move the text to an exact-size buffer
    // and free the old one unwiped; copying and wiping leaves no PIN behind.
    let secret = SecretString::from(pin.as_str());
    pin.zeroize();
    Ok(secret)
}
