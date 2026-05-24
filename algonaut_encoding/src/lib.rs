use data_encoding::BASE64;
use serde::de::Error;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryInto;
use std::ops::Deref;

/// A byte slice that round-trips through both JSON (base64 string) and
/// msgpack (`bin`).
///
/// Algorand's JSON renders byte slices as base64 strings, msgpack as raw
/// `bin`. `Bytes` bridges the two: on deserialize it accepts a base64
/// string, a raw byte buffer, or a sequence of bytes; on serialize it emits
/// a base64 string for human-readable formats and raw bytes otherwise.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Bytes(pub Vec<u8>);

impl Bytes {
    /// The base64 rendering of the bytes — algod's canonical JSON form.
    pub fn to_base64(&self) -> String {
        BASE64.encode(&self.0)
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(v: Vec<u8>) -> Self {
        Bytes(v)
    }
}

impl Serialize for Bytes {
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

impl<'de> Deserialize<'de> for Bytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BytesVisitor;

        impl<'de> Visitor<'de> for BytesVisitor {
            type Value = Bytes;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a base64 string or a byte buffer")
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<Bytes, E> {
                BASE64.decode(v.as_bytes()).map(Bytes).map_err(E::custom)
            }

            fn visit_bytes<E: Error>(self, v: &[u8]) -> Result<Bytes, E> {
                Ok(Bytes(v.to_vec()))
            }

            fn visit_byte_buf<E: Error>(self, v: Vec<u8>) -> Result<Bytes, E> {
                Ok(Bytes(v))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Bytes, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut bytes = Vec::new();
                while let Some(b) = seq.next_element::<u8>()? {
                    bytes.push(b);
                }
                Ok(Bytes(bytes))
            }
        }

        deserializer.deserialize_any(BytesVisitor)
    }
}

/// A string that round-trips through both JSON (string) and msgpack (`bin`
/// rendered as lossy UTF-8).
///
/// Some algod text fields — `gen`, `proto`, the transaction `type`
/// discriminant — are plain strings in JSON but `bin` values in msgpack.
/// A plain `String` only handles the JSON form; `Text` accepts both,
/// decoding msgpack bytes as lossy UTF-8.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Text(pub String);

impl From<String> for Text {
    fn from(s: String) -> Self {
        Text(s)
    }
}

impl From<&str> for Text {
    fn from(s: &str) -> Self {
        Text(s.to_owned())
    }
}

impl AsRef<str> for Text {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for Text {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl Serialize for Text {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.0)
        } else {
            serializer.serialize_bytes(self.0.as_bytes())
        }
    }
}

/// Deserialize an `Option<String>` byte-ish field leniently: a string is
/// kept verbatim, a byte buffer is base64-encoded.
///
/// Use this for fields that algod renders inconsistently across wire formats
/// — e.g. an algod block's `fees` / `rwd` (a base32-checksum address in
/// JSON, raw `bin` in msgpack) and `prev` (a `blk-…`-prefixed string in
/// JSON, raw `bin` in msgpack). [`Bytes`] is the right type for fields that
/// are *always* base64-or-bin; this helper is for the genuinely-mixed ones.
pub fn deserialize_opt_lenient_str<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptLenientStrVisitor;

    impl<'de> Visitor<'de> for OptLenientStrVisitor {
        type Value = Option<String>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
            d.deserialize_any(LenientStrVisitor).map(Some)
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

    struct LenientStrVisitor;

    impl<'de> Visitor<'de> for LenientStrVisitor {
        type Value = String;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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

    deserializer.deserialize_option(OptLenientStrVisitor)
}

impl<'de> Deserialize<'de> for Text {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TextVisitor;

        impl<'de> Visitor<'de> for TextVisitor {
            type Value = Text;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a string or a byte buffer")
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<Text, E> {
                Ok(Text(v.to_owned()))
            }

            fn visit_string<E: Error>(self, v: String) -> Result<Text, E> {
                Ok(Text(v))
            }

            fn visit_bytes<E: Error>(self, v: &[u8]) -> Result<Text, E> {
                Ok(Text(String::from_utf8_lossy(v).into_owned()))
            }

            fn visit_byte_buf<E: Error>(self, v: Vec<u8>) -> Result<Text, E> {
                Ok(Text(String::from_utf8_lossy(&v).into_owned()))
            }
        }

        deserializer.deserialize_any(TextVisitor)
    }
}

pub struct SignatureVisitor;

impl<'de> Visitor<'de> for SignatureVisitor {
    type Value = [u8; 64];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a 64 byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.len() == 64 {
            let mut bytes = [0; 64];
            bytes.copy_from_slice(v);
            Ok(bytes)
        } else {
            Err(E::custom(format!("Invalid signature length: {}", v.len())))
        }
    }
}

pub struct U8_32Visitor;

impl<'de> Visitor<'de> for U8_32Visitor {
    type Value = [u8; 32];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a 32 byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.len() == 32 {
            let mut bytes = [0; 32];
            bytes.copy_from_slice(v);
            Ok(bytes)
        } else {
            Err(E::custom(format!("Invalid byte array length: {}", v.len())))
        }
    }
}

pub struct U8_64Visitor;

impl<'de> Visitor<'de> for U8_64Visitor {
    type Value = [u8; 64];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a 64 byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.len() == 64 {
            let mut bytes = [0; 64];
            bytes.copy_from_slice(v);
            Ok(bytes)
        } else {
            Err(E::custom(format!("Invalid byte array length: {}", v.len())))
        }
    }
}

pub fn deserialize_bytes32<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
where
    D: Deserializer<'de>,
{
    let s = <&str>::deserialize(deserializer)?;
    let mut decoded = [0; 32];
    decoded.copy_from_slice(&BASE64.decode(s.as_bytes()).unwrap());
    Ok(decoded)
}

pub fn deserialize_bytes64<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
where
    D: Deserializer<'de>,
{
    let s = <&str>::deserialize(deserializer)?;
    let mut decoded = [0; 64];
    let bytes = BASE64.decode(s.as_bytes()).map_err(D::Error::custom)?;
    decoded.copy_from_slice(&bytes);
    Ok(decoded)
}

pub fn deserialize_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <&str>::deserialize(deserializer)?;
    Ok(BASE64.decode(s.as_bytes()).unwrap())
}

pub fn serialize_bytes<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&BASE64.encode(bytes))
}

pub fn decode_base64(bytes: &[u8]) -> Result<Vec<u8>, String> {
    BASE64.decode(bytes).map_err(|e| e.to_string())
}

pub fn deserialize_byte32_arr<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
where
    D: Deserializer<'de>,
{
    let slice = <&[u8]>::deserialize(deserializer)?;
    slice_to_byte32_arr::<D>(slice)
}

pub fn deserialize_byte32_arr_opt<'de, D>(deserializer: D) -> Result<Option<[u8; 32]>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match <Option<&[u8]>>::deserialize(deserializer)? {
        Some(slice) => Some(slice_to_byte32_arr::<D>(slice)?),
        None => None,
    })
}

fn slice_to_byte32_arr<'de, D>(slice: &[u8]) -> Result<[u8; 32], D::Error>
where
    D: Deserializer<'de>,
{
    slice.try_into().map_err(D::Error::custom)
}

pub fn deserialize_vec_opt_to_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<Vec<String>> = Deserialize::deserialize(deserializer)?;
    Ok(s.unwrap_or_default())
}

/// Deserialize a possibly-`null` value as `T::default()`.
///
/// algod returns `null` (rather than `[]`) for some array fields — e.g.
/// `TealDryrun200Response.txns` when the dryrun reports a top-level error —
/// which a plain `Vec<T>` field cannot decode. Pairing this with
/// `#[serde(default, deserialize_with = "deserialize_null_default")]` maps both
/// an explicit `null` and a missing field onto the empty/default value.
pub fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

/// The zero address in base32-checksum format (32 bytes of zeros).
///
/// The Algorand indexer returns this value for optional address fields (like
/// `clawback`, `freeze`, `manager`, `reserve`) when they were not set during
/// asset creation. This should be interpreted as `None` rather than a real
/// address.
pub const ZERO_ADDRESS: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAY5HFKQ";

/// Deserialize an optional address string, treating the zero address as `None`.
///
/// The Algorand indexer returns the zero address
/// (`AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAY5HFKQ`) for optional
/// address fields that were not set. This deserializer converts such values to
/// `None`, which better represents their semantic meaning.
///
/// See: <https://github.com/manuelmauro/algonaut/issues/142>
pub fn deserialize_opt_addr_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt.filter(|s| s != ZERO_ADDRESS))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestAsset {
        #[serde(deserialize_with = "deserialize_opt_addr_string", default)]
        clawback: Option<String>,
    }

    #[test]
    fn zero_address_becomes_none() {
        let json = r#"{"clawback": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAY5HFKQ"}"#;
        let asset: TestAsset = serde_json::from_str(json).unwrap();
        assert_eq!(asset.clawback, None);
    }

    #[test]
    fn real_address_is_preserved() {
        let addr = "7ZUECA7HFLZTXENRV24SHLU4AVPUTMTTDUFUBNBD64C73F3UHRTHAIOF6Q";
        let json = format!(r#"{{"clawback": "{}"}}"#, addr);
        let asset: TestAsset = serde_json::from_str(&json).unwrap();
        assert_eq!(asset.clawback, Some(addr.to_string()));
    }

    #[test]
    fn null_address_becomes_none() {
        let json = r#"{"clawback": null}"#;
        let asset: TestAsset = serde_json::from_str(json).unwrap();
        assert_eq!(asset.clawback, None);
    }

    #[test]
    fn missing_address_becomes_none() {
        let json = r#"{}"#;
        let asset: TestAsset = serde_json::from_str(json).unwrap();
        assert_eq!(asset.clawback, None);
    }
}
