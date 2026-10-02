//! SPEC §12: `present::caller`.
//!
//! Paths use `/` only, which every supported OS treats as a separator.

mod consent_key;
mod label;

use std::path::PathBuf;

use websign_core::present::caller::{
    CallerLabel, CodeSigner, DesktopCaller, caller_label, consent_key,
};

fn caller(path: &str, product: Option<&str>, signer: Option<CodeSigner>) -> DesktopCaller {
    DesktopCaller {
        executable: PathBuf::from(path),
        product_name: product.map(str::to_owned),
        signer,
    }
}

fn authenticode(subject: &str) -> Option<CodeSigner> {
    Some(CodeSigner::Authenticode {
        subject: subject.to_owned(),
    })
}

fn apple(team_id: &str, identifier: &str) -> Option<CodeSigner> {
    Some(CodeSigner::Apple {
        team_id: team_id.to_owned(),
        identifier: identifier.to_owned(),
    })
}

fn label(name: &str, detail: &str, verified: bool) -> CallerLabel {
    CallerLabel {
        name: name.to_owned(),
        detail: detail.to_owned(),
        verified,
        runs_scripts: false,
    }
}
