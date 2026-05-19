//! Wire-format helpers for algod's hand-written `ext` response models.
//!
//! Algod answers `/v2/blocks/*` and `/v2/transactions/pending*` in either
//! JSON or msgpack, depending on the request's `format` query parameter:
//!
//! - **JSON** renders byte slices as base64 strings.
//! - **msgpack** renders byte slices as raw `bin` values.
//!
//! [`WireBytes`] bridges the two: on deserialize it accepts a base64 string
//! (JSON) *or* a raw byte buffer (msgpack); on serialize it emits a base64
//! string for human-readable formats and raw bytes otherwise. That lets the
//! same `ext` model decode from either wire format and, once decoded,
//! re-serialize losslessly to a `serde_json::Value`.

use data_encoding::BASE64;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// A byte slice that round-trips through both JSON (base64 string) and
/// msgpack (`bin`).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct WireBytes(pub Vec<u8>);

impl WireBytes {
    /// The base64 rendering of the bytes — algod's canonical JSON form.
    pub fn to_base64(&self) -> String {
        BASE64.encode(&self.0)
    }
}

impl Serialize for WireBytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_base64())
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

impl<'de> Deserialize<'de> for WireBytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct WireBytesVisitor;

        impl<'de> Visitor<'de> for WireBytesVisitor {
            type Value = WireBytes;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a base64 string or a byte buffer")
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<WireBytes, E> {
                BASE64
                    .decode(v.as_bytes())
                    .map(WireBytes)
                    .map_err(E::custom)
            }

            fn visit_bytes<E: Error>(self, v: &[u8]) -> Result<WireBytes, E> {
                Ok(WireBytes(v.to_vec()))
            }

            fn visit_byte_buf<E: Error>(self, v: Vec<u8>) -> Result<WireBytes, E> {
                Ok(WireBytes(v))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<WireBytes, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut bytes = Vec::new();
                while let Some(b) = seq.next_element::<u8>()? {
                    bytes.push(b);
                }
                Ok(WireBytes(bytes))
            }
        }

        deserializer.deserialize_any(WireBytesVisitor)
    }
}

/// Deserialize an `Option<String>` field that may arrive as a plain string
/// (JSON) or as a UTF-8 byte buffer (msgpack).
///
/// algod's textual block fields — `gen` (genesis id), `proto` (protocol
/// string) — are plain strings in JSON and `bin` values in msgpack. A raw
/// `Option<String>` only handles the JSON form; this accepts both, decoding
/// msgpack bytes as lossy UTF-8.
pub fn deserialize_opt_text<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptTextVisitor;

    impl<'de> Visitor<'de> for OptTextVisitor {
        type Value = Option<String>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a string, a byte buffer, or null")
        }

        fn visit_none<E: Error>(self) -> Result<Option<String>, E> {
            Ok(None)
        }

        fn visit_unit<E: Error>(self) -> Result<Option<String>, E> {
            Ok(None)
        }

        fn visit_some<D>(self, d: D) -> Result<Option<String>, D::Error>
        where
            D: Deserializer<'de>,
        {
            d.deserialize_any(TextVisitor).map(Some)
        }

        fn visit_str<E: Error>(self, v: &str) -> Result<Option<String>, E> {
            Ok(Some(v.to_owned()))
        }

        fn visit_bytes<E: Error>(self, v: &[u8]) -> Result<Option<String>, E> {
            Ok(Some(String::from_utf8_lossy(v).into_owned()))
        }

        fn visit_byte_buf<E: Error>(self, v: Vec<u8>) -> Result<Option<String>, E> {
            Ok(Some(String::from_utf8_lossy(&v).into_owned()))
        }
    }

    struct TextVisitor;

    impl<'de> Visitor<'de> for TextVisitor {
        type Value = String;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a string or a byte buffer")
        }

        fn visit_str<E: Error>(self, v: &str) -> Result<String, E> {
            Ok(v.to_owned())
        }

        fn visit_bytes<E: Error>(self, v: &[u8]) -> Result<String, E> {
            Ok(String::from_utf8_lossy(v).into_owned())
        }

        fn visit_byte_buf<E: Error>(self, v: Vec<u8>) -> Result<String, E> {
            Ok(String::from_utf8_lossy(&v).into_owned())
        }
    }

    deserializer.deserialize_option(OptTextVisitor)
}

/// Deserialize an `Option<String>` byte-ish field leniently: a string is
/// kept verbatim, a byte buffer is base64-encoded.
///
/// Some algod block fields — notably `prev` — are rendered inconsistently
/// across fixtures and wire formats (a raw `blk-…` block-hash string here, a
/// base64 string there, raw `bin` in msgpack). None of them are asserted on;
/// this keeps the decode infallible by storing whatever arrives as a string.
pub fn deserialize_opt_bytes_str<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptBytesStrVisitor;

    impl<'de> Visitor<'de> for OptBytesStrVisitor {
        type Value = Option<String>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a string, a byte buffer, or null")
        }

        fn visit_none<E: Error>(self) -> Result<Option<String>, E> {
            Ok(None)
        }

        fn visit_unit<E: Error>(self) -> Result<Option<String>, E> {
            Ok(None)
        }

        fn visit_some<D>(self, d: D) -> Result<Option<String>, D::Error>
        where
            D: Deserializer<'de>,
        {
            d.deserialize_any(BytesStrVisitor).map(Some)
        }

        fn visit_str<E: Error>(self, v: &str) -> Result<Option<String>, E> {
            Ok(Some(v.to_owned()))
        }

        fn visit_bytes<E: Error>(self, v: &[u8]) -> Result<Option<String>, E> {
            Ok(Some(BASE64.encode(v)))
        }

        fn visit_byte_buf<E: Error>(self, v: Vec<u8>) -> Result<Option<String>, E> {
            Ok(Some(BASE64.encode(&v)))
        }
    }

    struct BytesStrVisitor;

    impl<'de> Visitor<'de> for BytesStrVisitor {
        type Value = String;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a string or a byte buffer")
        }

        fn visit_str<E: Error>(self, v: &str) -> Result<String, E> {
            Ok(v.to_owned())
        }

        fn visit_bytes<E: Error>(self, v: &[u8]) -> Result<String, E> {
            Ok(BASE64.encode(v))
        }

        fn visit_byte_buf<E: Error>(self, v: Vec<u8>) -> Result<String, E> {
            Ok(BASE64.encode(&v))
        }
    }

    deserializer.deserialize_option(OptBytesStrVisitor)
}

/// Deserialize a `String` field that may arrive as a string (JSON) or as a
/// UTF-8 byte buffer (msgpack).
///
/// algod's `type` transaction discriminant is a plain string in JSON but a
/// `bin` value in msgpack. serde's `#[serde(tag = "...")]` machinery insists
/// the tag deserialize as a string, so the `ext` transaction enum reads it
/// through a flat intermediate struct whose `type` field uses this helper.
pub fn deserialize_text<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    struct TextVisitor;

    impl<'de> Visitor<'de> for TextVisitor {
        type Value = String;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a string or a byte buffer")
        }

        fn visit_str<E: Error>(self, v: &str) -> Result<String, E> {
            Ok(v.to_owned())
        }

        fn visit_string<E: Error>(self, v: String) -> Result<String, E> {
            Ok(v)
        }

        fn visit_bytes<E: Error>(self, v: &[u8]) -> Result<String, E> {
            Ok(String::from_utf8_lossy(v).into_owned())
        }

        fn visit_byte_buf<E: Error>(self, v: Vec<u8>) -> Result<String, E> {
            Ok(String::from_utf8_lossy(&v).into_owned())
        }
    }

    deserializer.deserialize_any(TextVisitor)
}
