//! `websign-protocol` numbers → `extension/src/shared/limits.gen.ts`.
//!
//! The extension enforces the same version, message sources, lengths and
//! timers as the app; reading them from the crate (not from a hand copy)
//! means a change there shows up as a stale file in `check generated`.

use std::time::Duration;

use websign_protocol::limits;
use websign_protocol::page::{EXTENSION_SOURCE, PAGE_SOURCE};
use websign_protocol::version::PROTOCOL_VERSION;

use super::text::ts_header;
use crate::plan::Plan;

const TARGET: &str = "extension/src/shared/limits.gen.ts";

/// A value as a TypeScript literal.
enum Value {
    /// Decimal digits.
    Number(String),
    Text(&'static str),
}

fn millis(duration: Duration) -> Value {
    Value::Number(duration.as_millis().to_string())
}

fn count(value: usize) -> Value {
    Value::Number(value.to_string())
}

/// `(name, doc, value)` of every exported constant, in file order.
fn constants() -> [(&'static str, &'static str, Value); 8] {
    [
        (
            "PROTOCOL_VERSION",
            "The protocol version this extension speaks.",
            Value::Number(PROTOCOL_VERSION.to_string()),
        ),
        (
            "PAGE_SOURCE",
            "`source` of messages posted by the SDK.",
            Value::Text(PAGE_SOURCE),
        ),
        (
            "EXTENSION_SOURCE",
            "`source` of messages posted by the content script.",
            Value::Text(EXTENSION_SOURCE),
        ),
        (
            "MAX_REQUEST_ID_LEN",
            "Longest request id the app accepts, in bytes.",
            count(limits::MAX_REQUEST_ID_LEN),
        ),
        (
            "MAX_ORIGIN_LEN",
            "Longest origin the app accepts, in bytes.",
            count(limits::MAX_ORIGIN_LEN),
        ),
        (
            "MAX_IN_FLIGHT_PER_CONNECTION",
            "Requests of any kind in flight on one connection.",
            count(limits::MAX_IN_FLIGHT_PER_CONNECTION),
        ),
        (
            "APP_RESPONSE_TIMEOUT_MS",
            "How long to wait for the `hello` reply of an app that answered before.",
            millis(limits::APP_RESPONSE_TIMEOUT),
        ),
        (
            "EXTENSION_IDLE_CLOSE_MS",
            "How long an idle native messaging port stays open.",
            millis(limits::EXTENSION_IDLE_CLOSE),
        ),
    ]
}

pub fn plan() -> Result<Plan, String> {
    let mut out = ts_header("crates/websign-protocol");
    for (name, doc, value) in constants() {
        let literal = match value {
            Value::Number(digits) => group_digits(&digits),
            Value::Text(text) => serde_json::to_string(text).map_err(|e| e.to_string())?,
        };
        out.push_str(&format!(
            "\n/** {doc} */\nexport const {name} = {literal};\n"
        ));
    }
    let mut plan = Plan::default();
    plan.add(TARGET, out);
    Ok(plan)
}

/// `60000` → `60_000`, as Biome and people write numbers of five digits or more.
fn group_digits(digits: &str) -> String {
    if digits.len() < 5 {
        return digits.to_owned();
    }
    let mut out = String::new();
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push('_');
        }
        out.push(digit);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_numbers_get_separators() {
        assert_eq!(group_digits("3000"), "3000");
        assert_eq!(group_digits("60000"), "60_000");
        assert_eq!(group_digits("1048576"), "1_048_576");
    }

    #[test]
    fn every_constant_is_exported_with_its_doc() {
        let plan = plan().unwrap();
        let content = &plan.files[0].content;
        assert!(content.contains("/** How long an idle native messaging port stays open. */\n"));
        assert!(content.contains("export const EXTENSION_IDLE_CLOSE_MS = 60_000;\n"));
        assert!(content.contains("export const PAGE_SOURCE = \"websign-page\";\n"));
    }
}
