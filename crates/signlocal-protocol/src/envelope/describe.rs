//! Names the field that made a message body fail to parse (`SPEC.md` §5 step
//! 6), without echoing any value.
//!
//! serde's errors either name a field of our own schema (`missing field`) or
//! describe a value (`invalid type: string "…"`), which may be personal data.
//! So the culprit is found by elimination instead: the body is offered again
//! with one field at a time, and the search descends into the field whose
//! lone presence still breaks parsing (`filter` → `filter.algorithms`).

use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

/// The longest name taken from a frame that is echoed back in an error.
const MAX_ECHOED_NAME_LEN: usize = 32;

/// How deep the search descends; the schema itself is shallower.
const MAX_DEPTH: usize = 8;

/// The tag of internally tagged types, kept in every probe so the probe still
/// selects the same variant.
const TAG: &str = "type";

/// Text that is safe to put in an error message: short printable ASCII only.
pub(super) fn echoable(text: &str) -> Option<&str> {
    let safe = text.len() <= MAX_ECHOED_NAME_LEN && text.bytes().all(|b| b.is_ascii_graphic());
    safe.then_some(text)
}

/// A developer-facing sentence saying why `body` is not a `T`; `error` is
/// serde's own message for the whole body.
pub(super) fn failure<T: DeserializeOwned>(error: &str, body: &Value) -> String {
    if let Some(name) = missing_field(error) {
        return format!("required field {name} is missing");
    }
    let mut path = Vec::new();
    let mut node = body;
    while path.len() < MAX_DEPTH {
        let Some((step, child)) = culprit::<T>(body, &path, node) else {
            break;
        };
        if step.render().is_none() {
            return match render(&path) {
                Some(parent) => format!("a field of {parent} is unknown or has an invalid value"),
                None => "a field is unknown or has an invalid value".to_owned(),
            };
        }
        path.push(step);
        if wrong_kind::<T>(body, &path, child) {
            break;
        }
        node = child;
    }
    match render(&path) {
        Some(path) => format!("field {path} is unknown or has an invalid value"),
        None => "message body is invalid".to_owned(),
    }
}

/// The name in serde's "missing field `x`": always one of our own fields.
fn missing_field(error: &str) -> Option<&str> {
    error
        .strip_prefix("missing field `")
        .and_then(|rest| rest.split('`').next())
}

/// One step from a node to a child.
#[derive(Clone)]
enum Step {
    Key(String),
    Index(usize),
}

impl Step {
    /// The step as written in a path, when its key is safe to echo.
    fn render(&self) -> Option<String> {
        match self {
            Step::Key(key) => echoable(key).map(|key| format!(".{key}")),
            Step::Index(index) => Some(format!("[{index}]")),
        }
    }
}

/// `filter.algorithms`, `certificates[0].key`; `None` for the empty path.
fn render(path: &[Step]) -> Option<String> {
    let text: String = path.iter().filter_map(Step::render).collect();
    let text = text.strip_prefix('.').map(str::to_owned).unwrap_or(text);
    (!text.is_empty()).then_some(text)
}

/// The first child of `node` (reached from `body` by `path`) that breaks
/// parsing on its own. The tag is tried first, alone.
fn culprit<'a, T: DeserializeOwned>(
    body: &Value,
    path: &[Step],
    node: &'a Value,
) -> Option<(Step, &'a Value)> {
    let children: Vec<(Step, &Value)> = match node {
        Value::Object(object) => {
            let tag = object.get_key_value(TAG);
            let rest = object.iter().filter(|(key, _)| key.as_str() != TAG);
            tag.into_iter()
                .chain(rest)
                .map(|(key, value)| (Step::Key(key.clone()), value))
                .collect()
        }
        Value::Array(items) => items
            .iter()
            .enumerate()
            .map(|(i, v)| (Step::Index(i), v))
            .collect(),
        _ => Vec::new(),
    };
    children.into_iter().find(|(step, child)| {
        let mut probe_path = path.to_vec();
        probe_path.push(step.clone());
        // The empty stand-in first: it convicts unknown fields and values of
        // the wrong kind without copying a large value.
        stand_in(child).is_some_and(|empty| breaks::<T>(&probe(body, &probe_path, empty)))
            || breaks::<T>(&probe(body, &probe_path, (*child).clone()))
    })
}

/// An empty container of the same kind as `value`; `None` for scalars.
fn stand_in(value: &Value) -> Option<Value> {
    match value {
        Value::Object(_) => Some(Value::Object(Map::new())),
        Value::Array(_) => Some(Value::Array(Vec::new())),
        _ => None,
    }
}

/// Whether the child at `path` is itself of the wrong kind (a string where an
/// object belongs, an unknown field): an empty container in its place still
/// breaks parsing, so there is nothing to find inside it.
fn wrong_kind<T: DeserializeOwned>(body: &Value, path: &[Step], child: &Value) -> bool {
    stand_in(child).is_none_or(|empty| breaks::<T>(&probe(body, path, empty)))
}

/// Fails for a reason other than a field the probe left out.
fn breaks<T: DeserializeOwned>(probe: &Value) -> bool {
    match T::deserialize(probe) {
        Ok(_) => false,
        Err(error) => missing_field(&error.to_string()).is_none(),
    }
}

/// `body` reduced to the path: each object keeps only the next key (and its
/// tag), each array only the next element, and `leaf` sits at the end.
fn probe(body: &Value, path: &[Step], leaf: Value) -> Value {
    let Some((step, rest)) = path.split_first() else {
        return leaf;
    };
    match (body, step) {
        (Value::Object(object), Step::Key(key)) => {
            let mut reduced = Map::new();
            if let Some(tag) = object.get(TAG) {
                reduced.insert(TAG.to_owned(), tag.clone());
            }
            let child = object.get(key).unwrap_or(&Value::Null);
            reduced.insert(key.clone(), probe(child, rest, leaf));
            Value::Object(reduced)
        }
        (Value::Array(items), Step::Index(index)) => {
            let child = items.get(*index).unwrap_or(&Value::Null);
            Value::Array(vec![probe(child, rest, leaf)])
        }
        _ => leaf,
    }
}
