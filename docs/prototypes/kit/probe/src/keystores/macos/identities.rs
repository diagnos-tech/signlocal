//! Finding identities (certificate plus private key) without prompting.
//!
//! Chromium's client certificate store relies on one plain identity query
//! (`net/ssl/client_cert_store_mac.cc`). That query covers the keychain files
//! in the user's search list (login, System and any keychain added to it);
//! whether it also returns smart card identities depends on how
//! Security.framework routes it. So tokens get a second query that names the
//! token access group (`kSecAttrAccessGroupToken`), Apple's documented way to
//! reach CryptoTokenKit items. Every app may read that group, sandboxed or
//! not, without a `keychain-access-groups` entitlement.

use security_framework::identity::SecIdentity;
use security_framework::item::{ItemClass, ItemSearchOptions, Limit, Reference, SearchResult};
use security_framework_sys::base::errSecItemNotFound;

use super::{errors, token};
use crate::keystores::KeystoreError;

/// Where to look for identities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Keys stored in keychain files.
    Keychains,
    /// Keys held by CryptoTokenKit tokens (smart cards, USB tokens).
    Tokens,
}

/// An identity with the facts the key source needs up front.
#[derive(Debug)]
pub struct Identity {
    pub sec_identity: SecIdentity,
    pub cert_der: Vec<u8>,
    /// `kSecAttrTokenID` of the private key; `None` for keychain files.
    pub token_id: Option<String>,
}

/// What a search found. Identities that could not be read are counted in
/// `unreadable` instead of failing the whole search, so one broken item
/// never hides the others.
#[derive(Debug, Default)]
pub struct Found {
    pub identities: Vec<Identity>,
    pub unreadable: Vec<KeystoreError>,
}

/// Every identity in `scope`. Reading references never shows a PIN dialog:
/// the token or keychain asks only when a key is used.
pub fn find(scope: Scope) -> Result<Found, KeystoreError> {
    let mut found = Found::default();
    for identity in search(scope)? {
        match read(identity) {
            // The plain query may also return token identities; they belong
            // to the token scope, where their driver is reported.
            Ok(item) if belongs_to(scope, &item) => found.identities.push(item),
            Ok(_) => {}
            Err(error) => found.unreadable.push(error),
        }
    }
    Ok(found)
}

fn search(scope: Scope) -> Result<Vec<SecIdentity>, KeystoreError> {
    let mut query = ItemSearchOptions::new();
    query
        .class(ItemClass::identity())
        .load_refs(true)
        .limit(Limit::All);
    if scope == Scope::Tokens {
        query.access_group_token();
    }
    match query.search() {
        Ok(results) => Ok(results.into_iter().filter_map(as_identity).collect()),
        Err(error) if error.code() == errSecItemNotFound => Ok(Vec::new()),
        Err(error) => Err(errors::from_status("SecItemCopyMatching", error.code())),
    }
}

fn as_identity(result: SearchResult) -> Option<SecIdentity> {
    match result {
        SearchResult::Ref(Reference::Identity(identity)) => Some(identity),
        _ => None,
    }
}

fn read(identity: SecIdentity) -> Result<Identity, KeystoreError> {
    let certificate = identity
        .certificate()
        .map_err(|e| errors::from_status("SecIdentityCopyCertificate", e.code()))?;
    let key = identity
        .private_key()
        .map_err(|e| errors::from_status("SecIdentityCopyPrivateKey", e.code()))?;
    Ok(Identity {
        cert_der: certificate.to_der(),
        token_id: token::token_id(&key),
        sec_identity: identity,
    })
}

fn belongs_to(scope: Scope, item: &Identity) -> bool {
    match scope {
        Scope::Keychains => item.token_id.is_none(),
        // Everything in the token access group is a token item, even if its
        // key did not report a token ID.
        Scope::Tokens => true,
    }
}
