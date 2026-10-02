//! How a card's ATR is shown in the diagnostics report and on screen.
//!
//! The historical bytes of some cards carry a chip serial number, which is
//! personal data (`docs/research/tokens.md`). The interface bytes only
//! describe the card's transmission parameters, so they stay: they still help
//! tell card families apart. Historical bytes and the check byte are masked.

/// The text that stands for an ATR given as hex in any spelling (`3BD5…`,
/// `3b d5 …`, `3B:D5:…`).
///
/// - Well-formed ATR (ISO/IEC 7816-3 §8.2): upper-case bytes joined by `:`
///   up to the last interface byte, then `..` for every historical byte and
///   for TCK.
/// - Anything else that is whole hex bytes (truncated, trailing garbage, bad
///   TS): only its length, `19 bytes`.
/// - Text that is not hex bytes at all: `unreadable`. It is never echoed,
///   because it could hold the very bytes this function hides.
pub fn mask_atr(atr: &str) -> String {
    let Some(bytes) = parse_hex(atr) else {
        return "unreadable".to_owned();
    };
    match interface_len(&bytes) {
        Some((keep, masked)) => {
            let shown = bytes[..keep].iter().map(|byte| format!("{byte:02X}"));
            let hidden = std::iter::repeat_n("..".to_owned(), masked);
            shown.chain(hidden).collect::<Vec<_>>().join(":")
        }
        None => format!(
            "{} byte{}",
            bytes.len(),
            if bytes.len() == 1 { "" } else { "s" }
        ),
    }
}

fn parse_hex(text: &str) -> Option<Vec<u8>> {
    let digits: Vec<u8> = text
        .bytes()
        .filter(|byte| !matches!(byte, b':' | b' '))
        .collect();
    if digits.is_empty() || !digits.len().is_multiple_of(2) {
        return None;
    }
    digits
        .chunks(2)
        .map(|pair| {
            let high = char::from(pair[0]).to_digit(16)?;
            let low = char::from(pair[1]).to_digit(16)?;
            u8::try_from(high * 16 + low).ok()
        })
        .collect()
}

/// `(bytes kept, bytes masked)` when `bytes` is exactly one well-formed ATR:
/// TS, T0, the TA/TB/TC/TD chain, `K` historical bytes and TCK when any
/// protocol other than T=0 is offered.
fn interface_len(bytes: &[u8]) -> Option<(usize, usize)> {
    let (&ts, rest) = bytes.split_first()?;
    if ts != 0x3B && ts != 0x3F {
        return None;
    }
    let &t0 = rest.first()?;
    let historical = usize::from(t0 & 0x0F);
    let mut position = 2;
    let mut mask = t0 >> 4;
    let mut needs_tck = false;
    while mask != 0 {
        let present = (mask & 0b0111).count_ones() as usize;
        position += present;
        if mask & 0b1000 == 0 {
            break;
        }
        let td = *bytes.get(position)?;
        position += 1;
        // Only T=0 needs no check byte.
        needs_tck |= td & 0x0F != 0;
        mask = td >> 4;
    }
    let masked = historical + usize::from(needs_tck);
    (position + masked == bytes.len()).then_some((position, masked))
}

#[cfg(test)]
mod tests;
