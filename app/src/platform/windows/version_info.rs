//! The product name in an executable's version resource.

use std::path::Path;

use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};
use windows::core::PCWSTR;

use super::wide::{from_wide, wide};

/// Language/code page pairs tried when the resource lists none that
/// answer: US English with Unicode, then with Windows-1252.
const FALLBACK_TRANSLATIONS: [(u16, u16); 2] = [(0x0409, 0x04B0), (0x0409, 0x04E4)];

/// `FileDescription`, else `ProductName`, in the first language that has it.
pub fn product_name(executable: &Path) -> Option<String> {
    let block = VersionBlock::read(executable)?;
    let translations: Vec<(u16, u16)> = block
        .translations()
        .into_iter()
        .chain(FALLBACK_TRANSLATIONS)
        .collect();
    ["FileDescription", "ProductName"].iter().find_map(|key| {
        translations.iter().find_map(|(language, code_page)| {
            block
                .string(&format!(
                    "\\StringFileInfo\\{language:04x}{code_page:04x}\\{key}"
                ))
                .filter(|value| !value.trim().is_empty())
        })
    })
}

/// A version resource, in `u32`s so the UTF-16 and `u32` values
/// `VerQueryValueW` points into are aligned.
struct VersionBlock(Vec<u32>);

impl VersionBlock {
    fn read(executable: &Path) -> Option<Self> {
        let path = wide(executable);
        // SAFETY: `path` is NUL-terminated and outlives the call.
        let size = unsafe { GetFileVersionInfoSizeW(PCWSTR(path.as_ptr()), None) };
        if size == 0 {
            return None;
        }
        let mut data = vec![0u32; (size as usize).div_ceil(4)];
        // SAFETY: `data` has at least `size` writable bytes.
        unsafe { GetFileVersionInfoW(PCWSTR(path.as_ptr()), None, size, data.as_mut_ptr().cast()) }
            .ok()?;
        Some(Self(data))
    }

    /// The value at `sub_block`, as the bytes `VerQueryValueW` points at,
    /// checked to lie inside the block. `len_unit` is the size of the unit
    /// Windows counts `len` in (bytes for binary values, 2 for strings).
    fn value(&self, sub_block: &str, len_unit: usize) -> Option<&[u8]> {
        let query = wide(sub_block);
        let mut pointer = std::ptr::null_mut();
        let mut len = 0u32;
        // SAFETY: the block is a version resource `GetFileVersionInfoW`
        // filled; `query` is NUL-terminated; both out-pointers are valid.
        let found = unsafe {
            VerQueryValueW(
                self.0.as_ptr().cast(),
                PCWSTR(query.as_ptr()),
                &mut pointer,
                &mut len,
            )
        };
        let bytes = (len as usize).checked_mul(len_unit)?;
        let start = (pointer as usize).checked_sub(self.0.as_ptr() as usize)?;
        let end = start.checked_add(bytes)?;
        if !found.as_bool() || pointer.is_null() || end > self.0.len() * 4 {
            return None;
        }
        // SAFETY: `start..end` was just checked to lie inside `self.0`.
        Some(unsafe { std::slice::from_raw_parts(pointer.cast::<u8>(), bytes) })
    }

    fn translations(&self) -> Vec<(u16, u16)> {
        self.value("\\VarFileInfo\\Translation", 1)
            .map(|bytes| {
                bytes
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|&[l0, l1, c0, c1]| {
                        (u16::from_le_bytes([l0, l1]), u16::from_le_bytes([c0, c1]))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn string(&self, sub_block: &str) -> Option<String> {
        let bytes = self.value(sub_block, 2)?;
        let units: Vec<u16> = bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&unit| u16::from_le_bytes(unit))
            .collect();
        Some(from_wide(&units))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_description_of_a_system_program() {
        let notepad = Path::new("C:\\Windows\\System32\\notepad.exe");
        if notepad.exists() {
            assert!(product_name(notepad).is_some());
        }
        assert_eq!(product_name(Path::new("C:\\no\\such\\file.exe")), None);
    }
}
