//! Catalog signatures: how Windows signs its own programs (`cmd.exe`,
//! `powershell.exe`, `notepad.exe`). Those files carry no signature; their
//! hash is listed in a catalog file that Microsoft signed. Looking the hash
//! up and verifying the catalog entry names them "Microsoft Windows" instead
//! of "Unverified program".
//!
//! The hash is computed from the same open handle `WinVerifyTrust` reads,
//! so the file cannot change between the lookup and the check.

use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::os::windows::io::AsRawHandle;
use std::path::Path;

use windows::Win32::Foundation::HANDLE;
use windows::Win32::Security::Cryptography::Catalog::{
    CATALOG_INFO, CryptCATAdminAcquireContext2, CryptCATAdminCalcHashFromFileHandle2,
    CryptCATAdminEnumCatalogFromHash, CryptCATAdminReleaseCatalogContext,
    CryptCATAdminReleaseContext, CryptCATCatalogInfoFromContext,
};
use windows::Win32::Security::WinTrust::WINTRUST_CATALOG_INFO;
use windows::core::{PCWSTR, w};

use super::trust::{Subject, verified_signer};
use super::wide::wide;

/// Longest hash a catalog uses (SHA-512 would be 64 bytes; SHA-256 is 32).
const MAX_HASH: usize = 64;

/// The verified signer of the catalog that lists `file`; `None` when no
/// catalog on this machine lists it. Current catalogs hash with SHA-256,
/// older ones (and some drivers' tools) with SHA-1, the API's default.
pub fn signer(executable: &Path, file: &File) -> Option<String> {
    [w!("SHA256"), PCWSTR::null()]
        .into_iter()
        .find_map(|algorithm| signer_with(algorithm, executable, file))
}

fn signer_with(algorithm: PCWSTR, executable: &Path, file: &File) -> Option<String> {
    let admin = Admin::acquire(algorithm)?;
    let mut hash = admin.hash(file)?;
    let catalog = admin.catalog_listing(&hash)?;
    let catalog_path = catalog.path()?;
    let tag = wide(hex_upper(&hash));
    let member = wide(executable);
    // `WinVerifyTrust` reads the file again through the handle.
    let mut rewound = file;
    rewound.seek(SeekFrom::Start(0)).ok()?;
    let mut info = WINTRUST_CATALOG_INFO {
        cbStruct: size_of::<WINTRUST_CATALOG_INFO>() as u32,
        pcwszCatalogFilePath: PCWSTR(catalog_path.as_ptr()),
        pcwszMemberTag: PCWSTR(tag.as_ptr()),
        pcwszMemberFilePath: PCWSTR(member.as_ptr()),
        hMemberFile: HANDLE(file.as_raw_handle()),
        pbCalculatedFileHash: hash.as_mut_ptr(),
        cbCalculatedFileHash: hash.len() as u32,
        hCatAdmin: admin.0,
        ..Default::default()
    };
    verified_signer(Subject::Catalog(&mut info))
}

/// The member tag of a catalog entry: its hash in uppercase hex.
fn hex_upper(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
}

/// A catalog administrator context, released on drop.
struct Admin(isize);

impl Admin {
    fn acquire(algorithm: PCWSTR) -> Option<Self> {
        let mut handle = 0isize;
        // SAFETY: `handle` is writable; `algorithm` is a static string or
        // null (the default, SHA-1); no subsystem = the default one.
        unsafe { CryptCATAdminAcquireContext2(&mut handle, None, algorithm, None, None) }.ok()?;
        (handle != 0).then(|| Self(handle))
    }

    /// The file's hash as catalogs of this context's algorithm list it.
    fn hash(&self, file: &File) -> Option<Vec<u8>> {
        let mut hash = vec![0u8; MAX_HASH];
        let mut len = MAX_HASH as u32;
        // SAFETY: `hash` has `len` writable bytes; Windows stores the length
        // written in `len`; the file handle is open for reading.
        unsafe {
            CryptCATAdminCalcHashFromFileHandle2(
                self.0,
                HANDLE(file.as_raw_handle()),
                &mut len,
                Some(hash.as_mut_ptr()),
                None,
            )
        }
        .ok()?;
        hash.truncate(len as usize);
        (!hash.is_empty()).then_some(hash)
    }

    /// The first catalog that lists `hash`.
    fn catalog_listing(&self, hash: &[u8]) -> Option<CatalogContext<'_>> {
        // SAFETY: a live context and a hash slice; 0 means none found.
        let info = unsafe { CryptCATAdminEnumCatalogFromHash(self.0, hash, None, None) };
        (info != 0).then(|| CatalogContext { admin: self, info })
    }
}

impl Drop for Admin {
    fn drop(&mut self) {
        // SAFETY: acquired in `acquire`, released once; catalog contexts
        // borrow `self`, so they are released before.
        unsafe {
            let _ = CryptCATAdminReleaseContext(self.0, 0);
        }
    }
}

/// A catalog found for a hash, released on drop.
struct CatalogContext<'a> {
    admin: &'a Admin,
    info: isize,
}

impl CatalogContext<'_> {
    /// The catalog file's path, NUL-terminated for `WinVerifyTrust`.
    fn path(&self) -> Option<Vec<u16>> {
        let mut info = CATALOG_INFO {
            cbStruct: size_of::<CATALOG_INFO>() as u32,
            ..Default::default()
        };
        // SAFETY: a live catalog context and a sized, writable structure.
        unsafe { CryptCATCatalogInfoFromContext(self.info, &mut info, 0) }.ok()?;
        let end = info.wszCatalogFile.iter().position(|&unit| unit == 0)?;
        Some(info.wszCatalogFile[..=end].to_vec())
    }
}

impl Drop for CatalogContext<'_> {
    fn drop(&mut self) {
        // SAFETY: the context `CryptCATAdminEnumCatalogFromHash` returned
        // for this admin context, released once.
        unsafe {
            let _ = CryptCATAdminReleaseCatalogContext(self.admin.0, self.info, 0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn member_tags_are_uppercase_hex() {
        assert_eq!(hex_upper(&[0x0a, 0xbc, 0xff]), "0ABCFF");
    }

    #[test]
    fn inbox_programs_are_signed_by_microsoft_windows() {
        let cmd = Path::new(r"C:\Windows\System32\cmd.exe");
        let Ok(file) = File::open(cmd) else {
            return;
        };
        assert_eq!(signer(cmd, &file).as_deref(), Some("Microsoft Windows"));
    }
}
