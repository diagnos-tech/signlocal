//! Issuer certificates of a leaf, as the macOS chain engine builds them.
//!
//! `SecTrust` searches the keychains in the search list and the system roots
//! for intermediates, which is what the signed document needs to embed.
//! Network fetching is switched off: listing a chain must not reach out to
//! the CA's AIA URL on every call, and the result is best effort anyway.

use std::ptr;

use core_foundation::array::{CFArray, CFArrayRef};
use core_foundation::base::TCFType;
use security_framework::certificate::SecCertificate;
use security_framework::policy::SecPolicy;
use security_framework::trust::SecTrust;
use security_framework_sys::trust::{SecTrustEvaluateWithError, SecTrustRef};

/// Longest chain returned; real PKIs stay well below it, and it bounds what
/// a hostile keychain could make the app embed.
const MAX_ISSUERS: usize = 8;

// `security-framework-sys` exposes this only behind its `macos-12` feature.
// The app requires macOS 13, so the symbol is always present.
#[link(name = "Security", kind = "framework")]
unsafe extern "C" {
    fn SecTrustCopyCertificateChain(trust: SecTrustRef) -> CFArrayRef;
}

/// Issuers of `cert_der`, nearest first, leaf excluded; empty when the
/// engine finds none or fails.
pub fn issuers(cert_der: &[u8]) -> Vec<Vec<u8>> {
    let Ok(leaf) = SecCertificate::from_der(cert_der) else {
        return Vec::new();
    };
    let Ok(mut trust) = SecTrust::create_with_certificates(&[leaf], &[SecPolicy::create_x509()])
    else {
        return Vec::new();
    };
    if trust.set_network_fetch_allowed(false).is_err() {
        return Vec::new();
    }
    // The verdict is irrelevant (an expired or unknown root still yields the
    // chain built so far); evaluating is what makes the engine build it.
    // SAFETY: `trust` keeps the SecTrustRef alive for the call; a NULL error
    // pointer is allowed and means "do not report the error".
    let _trusted =
        unsafe { SecTrustEvaluateWithError(trust.as_concrete_TypeRef(), ptr::null_mut()) };
    evaluated_chain(&trust)
        .into_iter()
        .skip(1)
        .map(|certificate| certificate.to_der())
        .filter(|der| der != cert_der)
        .take(MAX_ISSUERS)
        .collect()
}

/// The chain of an evaluated trust, leaf first.
fn evaluated_chain(trust: &SecTrust) -> Vec<SecCertificate> {
    // SAFETY: `trust` keeps the SecTrustRef alive for the call. The result
    // follows the Copy rule and may be NULL, which is checked before wrapping.
    let raw = unsafe { SecTrustCopyCertificateChain(trust.as_concrete_TypeRef()) };
    if raw.is_null() {
        return Vec::new();
    }
    // SAFETY: `raw` is a non-NULL CFArray we own, and Apple documents its
    // elements as SecCertificateRefs; the wrapper takes over the reference.
    let array: CFArray<SecCertificate> = unsafe { CFArray::wrap_under_create_rule(raw) };
    array
        .iter()
        .map(|certificate| certificate.clone())
        .collect()
}
