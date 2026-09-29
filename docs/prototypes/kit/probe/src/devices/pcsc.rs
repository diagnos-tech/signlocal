//! Smart card readers and the ATR of the card in each, through PC/SC.
//!
//! The reader state is read without connecting to the card: no reset, no
//! exclusive access, nothing that could disturb another application that is
//! using it.

use std::time::Duration;

use pcsc::{Context, Error, ReaderState, Scope, State};
use serde::Serialize;

/// What a reader holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CardState {
    Empty,
    Present,
    /// A card that does not answer to the reset (dirty contacts, wrong side).
    Unresponsive,
    /// The PC/SC service could not tell.
    Unknown,
}

/// One reader.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Reader {
    pub name: String,
    pub card: CardState,
    /// Another application holds the card exclusively.
    pub in_use: bool,
    /// The card's Answer To Reset, uppercase hex without separators.
    pub atr: Option<String>,
}

/// The readers, and why none could be listed if the service is unavailable.
#[derive(Debug, Serialize)]
pub struct ReaderScan {
    pub readers: Vec<Reader>,
    pub problem: Option<String>,
}

pub fn scan() -> ReaderScan {
    match read_readers() {
        Ok(readers) => ReaderScan {
            readers,
            problem: None,
        },
        Err(error) => ReaderScan {
            readers: Vec::new(),
            problem: Some(describe(error)),
        },
    }
}

fn read_readers() -> Result<Vec<Reader>, Error> {
    let context = Context::establish(Scope::User)?;
    let names = match context.list_readers_owned() {
        Ok(names) => names,
        Err(Error::NoReadersAvailable) => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut states: Vec<ReaderState> = names
        .into_iter()
        .map(|name| ReaderState::new(name, State::UNAWARE))
        .collect();
    if states.is_empty() {
        return Ok(Vec::new());
    }
    context.get_status_change(Duration::from_millis(500), &mut states)?;
    Ok(states
        .iter()
        .map(|state| {
            reader(
                &state.name().to_string_lossy(),
                state.event_state(),
                state.atr(),
            )
        })
        .collect())
}

fn reader(name: &str, state: State, atr: &[u8]) -> Reader {
    let card = if state.contains(State::MUTE) {
        CardState::Unresponsive
    } else if state.contains(State::PRESENT) {
        CardState::Present
    } else if state.contains(State::EMPTY) {
        CardState::Empty
    } else {
        CardState::Unknown
    };
    Reader {
        name: name.to_owned(),
        card,
        in_use: state.intersects(State::INUSE | State::EXCLUSIVE),
        atr: (card == CardState::Present && !atr.is_empty()).then(|| super::format::hex_upper(atr)),
    }
}

/// A sentence for the person reading the output, not an error code.
fn describe(error: Error) -> String {
    match error {
        Error::NoService | Error::ServiceStopped => {
            "the PC/SC service is not running (start pcscd on Linux, or the Smart Card service on Windows)".to_owned()
        }
        Error::NoReadersAvailable => "no smart card readers are connected".to_owned(),
        other => format!("PC/SC is not available: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_present_card_reports_its_atr_in_hex() {
        let reader = reader(
            "Reader 00 00",
            State::PRESENT | State::CHANGED,
            &[0x3B, 0x8F, 0x80, 0x01],
        );
        assert_eq!(reader.card, CardState::Present);
        assert_eq!(reader.atr.as_deref(), Some("3B8F8001"));
        assert!(!reader.in_use);
    }

    #[test]
    fn an_empty_reader_has_no_atr() {
        let reader = reader("Reader 00 00", State::EMPTY, &[]);
        assert_eq!((reader.card, reader.atr), (CardState::Empty, None));
    }

    #[test]
    fn a_mute_card_is_unresponsive_and_a_held_card_is_in_use() {
        assert_eq!(
            reader("r", State::PRESENT | State::MUTE, &[0x3B]).card,
            CardState::Unresponsive
        );
        assert!(reader("r", State::PRESENT | State::INUSE, &[0x3B]).in_use);
        assert!(reader("r", State::PRESENT | State::EXCLUSIVE, &[0x3B]).in_use);
        assert_eq!(reader("r", State::UNAWARE, &[]).card, CardState::Unknown);
    }

    #[test]
    fn a_missing_service_gets_an_actionable_sentence() {
        assert!(describe(Error::NoService).contains("pcscd"));
        assert!(describe(Error::InternalError).starts_with("PC/SC is not available"));
    }
}
