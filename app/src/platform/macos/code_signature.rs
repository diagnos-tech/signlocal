//! The verified code-signing identity of a running macOS program.
//!
//! The running process is checked first (`SecCodeCopyGuestWithAttributes`
//! by audit token or PID, then `SecCodeCheckValidity`), so a binary modified
//! after launch does not pass; when the kernel does not answer for the
//! process, the file on disk is checked instead. A signature counts only when it chains to Apple:
//! Apple's own programs (`anchor apple`, which carry no team ID and are
//! reported with team `Apple`, a value no real 10-character team ID takes)
//! or a Developer ID / App Store certificate (`anchor apple generic`).
//! Ad-hoc and self-signed programs are unverified.

use std::ffi::c_void;
use std::path::Path;
use std::str::FromStr;

use core_foundation::base::{CFType, TCFType};
use core_foundation::data::CFData;
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::string::{CFString, CFStringRef};
use core_foundation::url::CFURL;
use security_framework::os::macos::code_signing::{
    Flags, GuestAttributes, SecCode, SecRequirement, SecStaticCode,
};
use websign_core::present::caller::CodeSigner;

use super::peer::AuditToken;

// Not wrapped by `security-framework`.
#[link(name = "Security", kind = "framework")]
unsafe extern "C" {
    fn SecCodeCopySigningInformation(
        code: *const c_void,
        flags: u32,
        information: *mut CFDictionaryRef,
    ) -> i32;
    static kSecCodeInfoIdentifier: CFStringRef;
    static kSecCodeInfoTeamIdentifier: CFStringRef;
}

/// `kSecCSSigningInformation`: include the certificate-derived fields.
const SIGNING_INFORMATION: u32 = 1 << 1;
const APPLE_TEAM: &str = "Apple";

/// How the running process is named to the Security framework.
pub enum Guest {
    /// Exact: the token also names the executable image (survives `exec`).
    AuditToken(AuditToken),
    /// The process ID.
    Pid(i32),
}

pub fn signer(guest: &Guest, executable: &Path) -> Option<CodeSigner> {
    let code = Code::running(guest).or_else(|| Code::on_disk(executable))?;
    let apple_itself = code.satisfies("anchor apple");
    if !apple_itself && !code.satisfies("anchor apple generic") {
        return None;
    }
    let information = code.signing_information()?;
    let identifier = string(&information, info_key(KeyName::Identifier))?;
    let team_id = if apple_itself {
        APPLE_TEAM.to_owned()
    } else {
        string(&information, info_key(KeyName::Team))?
    };
    Some(CodeSigner::Apple {
        team_id,
        identifier,
    })
}

enum Code {
    Running(SecCode),
    OnDisk(SecStaticCode),
}

impl Code {
    fn running(guest: &Guest) -> Option<Self> {
        let mut attributes = GuestAttributes::new();
        match guest {
            Guest::AuditToken(audit) => {
                // The attributes dictionary retains it.
                let token = CFData::from_buffer(&audit.bytes());
                attributes.set_audit_token(token.as_concrete_TypeRef());
            }
            Guest::Pid(pid) => attributes.set_pid(*pid),
        }
        SecCode::copy_guest_with_attribues(None, &attributes, Flags::NONE)
            .ok()
            .map(Code::Running)
    }

    fn on_disk(executable: &Path) -> Option<Self> {
        let url = CFURL::from_path(executable, false)?;
        SecStaticCode::from_path(&url, Flags::NONE)
            .ok()
            .map(Code::OnDisk)
    }

    fn satisfies(&self, requirement: &str) -> bool {
        let Ok(requirement) = SecRequirement::from_str(requirement) else {
            return false;
        };
        match self {
            Code::Running(code) => code.check_validity(Flags::NONE, &requirement).is_ok(),
            Code::OnDisk(code) => code.check_validity(Flags::NONE, &requirement).is_ok(),
        }
    }

    fn signing_information(&self) -> Option<CFDictionary<CFString, CFType>> {
        let code = match self {
            Code::Running(code) => code.as_CFTypeRef(),
            Code::OnDisk(code) => code.as_CFTypeRef(),
        };
        let mut information: CFDictionaryRef = std::ptr::null();
        // SAFETY: `code` is a live `SecCode`/`SecStaticCode` (the call takes
        // either); on success we own the returned dictionary.
        let status =
            unsafe { SecCodeCopySigningInformation(code, SIGNING_INFORMATION, &mut information) };
        if status != 0 || information.is_null() {
            return None;
        }
        // SAFETY: a +1 dictionary from a `Copy` call, released on drop.
        Some(unsafe { CFDictionary::wrap_under_create_rule(information) })
    }
}

enum KeyName {
    Identifier,
    Team,
}

fn info_key(name: KeyName) -> CFString {
    // SAFETY: Security framework constants, valid for the process lifetime.
    let key = unsafe {
        match name {
            KeyName::Identifier => kSecCodeInfoIdentifier,
            KeyName::Team => kSecCodeInfoTeamIdentifier,
        }
    };
    // SAFETY: a valid, immortal `CFString`; the get rule retains it.
    unsafe { CFString::wrap_under_get_rule(key) }
}

fn string(information: &CFDictionary<CFString, CFType>, key: CFString) -> Option<String> {
    let value = information.find(&key)?.downcast::<CFString>()?.to_string();
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apple_programs_are_signed_by_apple() {
        let finder = Path::new("/System/Library/CoreServices/Finder.app/Contents/MacOS/Finder");
        let Some(CodeSigner::Apple {
            team_id,
            identifier,
        }) = signer(&Guest::Pid(-1), finder)
        else {
            panic!("Finder is signed by Apple");
        };
        assert_eq!(team_id, APPLE_TEAM);
        assert_eq!(identifier, "com.apple.finder");
    }

    #[test]
    fn missing_programs_are_unverified() {
        assert_eq!(signer(&Guest::Pid(-1), Path::new("/no/such/program")), None);
    }
}
