//! Serde glue that makes the derived impls as strict as the protocol.
//!
//! serde's derives accept a JSON array wherever a struct is expected (`[1, 1]`
//! for `{"min": 1, "max": 1}`) and `null` for any `Option` field. Either would
//! give a message a second spelling, and a relay that validates one spelling
//! could be walked around with the other (`SPEC.md` §1). So:
//!
//! * every struct and internally tagged enum derives with
//!   `#[serde(remote = "Self")]` and gets its `Serialize`/`Deserialize` impls
//!   from [`object_serde!`], which deserializes through [`ObjectOnly`];
//! * every optional field deserializes with [`present`], which refuses `null`.
//!
//! `remote = "Self"` turns the derived code into inherent
//! `X::serialize`/`X::deserialize` functions that do not refuse arrays. Always
//! go through the traits (`serde_json::from_*` does); never call
//! `X::deserialize` by path.

use std::fmt;

use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};

/// Wraps a deserializer so that the value must be a JSON object: the
/// visitor's `visit_seq` is never reached. Only valid for struct and
/// internally tagged enum visitors, which accept nothing but maps and
/// sequences.
pub(crate) struct ObjectOnly<D>(pub D);

impl<'de, D: Deserializer<'de>> Deserializer<'de> for ObjectOnly<D> {
    type Error = D::Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_any(MapsOnly(visitor))
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_map(MapsOnly(visitor))
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, D::Error> {
        self.0.deserialize_struct(name, fields, MapsOnly(visitor))
    }

    fn is_human_readable(&self) -> bool {
        self.0.is_human_readable()
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct enum identifier ignored_any
    }
}

/// Passes maps to the wrapped visitor; everything else, sequences included,
/// gets the visitor's own "invalid type" error.
struct MapsOnly<V>(V);

impl<'de, V: Visitor<'de>> Visitor<'de> for MapsOnly<V> {
    type Value = V::Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.expecting(formatter)
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<V::Value, A::Error> {
        self.0.visit_map(map)
    }
}

/// `deserialize_with` for optional fields: absent is `None` (with
/// `#[serde(default)]`), present must be a real value, never `null`.
pub(crate) fn present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

/// Implements `Serialize` and `Deserialize` for types derived with
/// `#[serde(remote = "Self")]`: serialization as derived, deserialization
/// through [`ObjectOnly`].
macro_rules! object_serde {
    ($($ty:ty),+ $(,)?) => {$(
        impl ::serde::Serialize for $ty {
            fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                <$ty>::serialize(self, serializer)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $ty {
            fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                <$ty>::deserialize($crate::strict::ObjectOnly(deserializer))
            }
        }
    )+};
}

pub(crate) use object_serde;
