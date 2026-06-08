//! Support surface for the `abi_call!` / `abi_method!` macros.
//!
//! Everything the macros' *expansion* references lives here: the per-ABI-type
//! marker types, the [`AbiArg`] trait that pins a Rust value to an ABI type at
//! compile time, and [`MethodInvocation`], the checked value the macros
//! produce. Call sites use the macros, not these items directly; they are
//! `pub` because macro output must name them.
//!
//! The design mirrors `format!`: a marker type plays the role of a format
//! specifier (`{}` ⇒ `Display`), and `AbiArg<Marker>` plays the role of the
//! trait the specifier selects. `abi_call!("…(uint64)…", x)` emits
//! `AbiArg::<Uint<64>>::encode(x)`; if `x`'s type has no `AbiArg<Uint<64>>`
//! impl, the type-checker rejects it, spanned to `x`.

use crate::abi_type::AbiValue;
use algonaut_core::Address as CoreAddress;
use num_bigint::BigUint;
use std::fmt;
use std::marker::PhantomData;

use crate::abi_interactions::AbiMethod;

/// "This Rust type may stand in for ABI type `T`." Implemented for each Rust
/// representation accepted in an `abi_call!` argument slot of ABI type `T`;
/// [`encode`](AbiArg::encode) turns the value into its [`AbiValue`].
///
/// Reuses the `From<…> for AbiValue` conversions in
/// [`crate::abi_type`], so the macro path and the runtime path encode values
/// identically.
pub trait AbiArg<T> {
    /// Encode `self` as the [`AbiValue`] for ABI type `T`.
    fn encode(self) -> AbiValue;
}

// === Marker types =========================================================
//
// Zero-sized stand-ins for ABI types, synthesized by the macros from the
// parsed signature. They exist only to be the `T` in `AbiArg<T>`.

/// `uintN` marker (`N` = bit size, a multiple of 8 in `8..=512`).
pub struct Uint<const BITS: u16>;
/// `byte` marker.
pub struct Byte;
/// `bool` marker.
pub struct Bool;
/// `address` marker (distinct from [`algonaut_core::Address`], which is a
/// value type).
pub struct Address;
/// `string` marker.
pub struct AbiString;
/// `byte[]` marker — the one compound type with a canonical Rust rep.
pub struct Bytes;
/// `ufixedNxM` marker. The `contract!` macro and `abi_call!` accept the
/// [`Ufixed`] value newtype in this slot (its `AbiArg` impl is below).
pub struct UFixed<const BITS: u16, const PRECISION: u16>;
/// `T[]` marker.
pub struct DynArray<T>(PhantomData<T>);
/// `T[N]` marker.
pub struct StaticArray<T, const N: u16>(PhantomData<T>);
/// `(T0,T1,…)` marker.
pub struct Tuple<T>(PhantomData<T>);

// === AbiArg impls =========================================================

/// `uintN` ← native unsigned integers, generated per (Rust type, bit size).
/// A value of a `K`-bit native type always fits an `N`-bit ABI uint when
/// `K <= N`, so the impls only widen — passing a `u64` where `uint8` is
/// expected has no impl and fails to compile.
macro_rules! impl_abi_arg_uint_native {
    ($native:ty => [$($bits:literal),* $(,)?]) => {$(
        impl AbiArg<Uint<$bits>> for $native {
            #[inline]
            fn encode(self) -> AbiValue {
                AbiValue::Int(BigUint::from(self))
            }
        }
    )*};
}

impl_abi_arg_uint_native!(u8 => [8, 16, 32, 64, 128]);
impl_abi_arg_uint_native!(u16 => [16, 32, 64, 128]);
impl_abi_arg_uint_native!(u32 => [32, 64, 128]);
impl_abi_arg_uint_native!(u64 => [64, 128]);
impl_abi_arg_uint_native!(u128 => [128]);

/// `uintN` ← [`BigUint`], for every ABI uint width (including the non-native
/// widths like `uint24` or `uint256`). The value is range-checked at encode
/// time by `AbiType::encode`, as with any `BigUint`-sourced argument.
macro_rules! impl_abi_arg_uint_bigint {
    ([$($bits:literal),* $(,)?]) => {$(
        impl AbiArg<Uint<$bits>> for BigUint {
            #[inline]
            fn encode(self) -> AbiValue {
                AbiValue::Int(self)
            }
        }
    )*};
}

impl_abi_arg_uint_bigint!([
    8, 16, 24, 32, 40, 48, 56, 64, 72, 80, 88, 96, 104, 112, 120, 128, 136, 144, 152, 160, 168,
    176, 184, 192, 200, 208, 216, 224, 232, 240, 248, 256, 264, 272, 280, 288, 296, 304, 312, 320,
    328, 336, 344, 352, 360, 368, 376, 384, 392, 400, 408, 416, 424, 432, 440, 448, 456, 464, 472,
    480, 488, 496, 504, 512
]);

impl AbiArg<Byte> for u8 {
    #[inline]
    fn encode(self) -> AbiValue {
        AbiValue::Byte(self)
    }
}

impl AbiArg<Bool> for bool {
    #[inline]
    fn encode(self) -> AbiValue {
        AbiValue::Bool(self)
    }
}

impl AbiArg<Address> for CoreAddress {
    #[inline]
    fn encode(self) -> AbiValue {
        AbiValue::Address(self)
    }
}

impl AbiArg<AbiString> for &str {
    #[inline]
    fn encode(self) -> AbiValue {
        AbiValue::String(self.to_owned())
    }
}

impl AbiArg<AbiString> for String {
    #[inline]
    fn encode(self) -> AbiValue {
        AbiValue::String(self)
    }
}

impl AbiArg<Bytes> for Vec<u8> {
    #[inline]
    fn encode(self) -> AbiValue {
        // Reuses `From<Vec<u8>> for AbiValue`: each byte becomes a `Byte`
        // element of a dynamic array, the canonical `byte[]` representation.
        AbiValue::from(self)
    }
}

// === Typed decoding =======================================================
//
// The reverse of `AbiArg<T>`: turn an [`AbiValue`] back into the Rust type the
// macro picked for ABI type `T`. The generated struct `abi_decode`, the typed
// return/state/event accessors, and the scalar decoders all funnel through
// these impls, so the encode and decode mappings stay symmetric by
// construction.

/// An error decoding an [`AbiValue`] into its generated Rust type — a shape
/// mismatch (e.g. an integer where a tuple was expected) or an out-of-range
/// integer (e.g. a `uint64` value that does not fit the chosen `u32`).
///
/// Named here, in the runtime crate, rather than emitted per `contract!`
/// invocation, so two clients in one module share one error type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbiDecodeError(pub String);

impl AbiDecodeError {
    /// Build an error with a human-readable message.
    pub fn new(msg: impl Into<String>) -> Self {
        AbiDecodeError(msg.into())
    }
}

impl fmt::Display for AbiDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ABI decode error: {}", self.0)
    }
}

impl std::error::Error for AbiDecodeError {}

/// "This Rust type is the decoded form of ABI type `T`." The dual of
/// [`AbiArg`]: [`decode`](AbiDecode::decode) turns an [`AbiValue`] back into the
/// Rust value, failing on a shape or range mismatch.
pub trait AbiDecode<T>: Sized {
    /// Decode `value` as the Rust representation of ABI type `T`.
    fn decode(value: AbiValue) -> Result<Self, AbiDecodeError>;
}

/// Pull the [`BigUint`] out of an [`AbiValue::Int`] (or a single
/// [`AbiValue::Byte`], the `byte`/`uint8` overlap), erroring otherwise.
fn as_biguint(value: AbiValue) -> Result<BigUint, AbiDecodeError> {
    match value {
        AbiValue::Int(n) => Ok(n),
        AbiValue::Byte(b) => Ok(BigUint::from(b)),
        other => Err(AbiDecodeError(format!(
            "expected an integer, got {other:?}"
        ))),
    }
}

/// `uintN` → native unsigned integers, range-checked by the target type's
/// `TryFrom<&BigUint>`. Mirrors `impl_abi_arg_uint_native`.
macro_rules! impl_abi_decode_uint_native {
    ($native:ty => [$($bits:literal),* $(,)?]) => {$(
        impl AbiDecode<Uint<$bits>> for $native {
            fn decode(value: AbiValue) -> Result<Self, AbiDecodeError> {
                let n = as_biguint(value)?;
                <$native>::try_from(&n).map_err(|_| {
                    AbiDecodeError(format!(
                        "value {n} does not fit {}",
                        stringify!($native),
                    ))
                })
            }
        }
    )*};
}

impl_abi_decode_uint_native!(u8 => [8, 16, 32, 64, 128]);
impl_abi_decode_uint_native!(u16 => [16, 32, 64, 128]);
impl_abi_decode_uint_native!(u32 => [32, 64, 128]);
impl_abi_decode_uint_native!(u64 => [64, 128]);
impl_abi_decode_uint_native!(u128 => [128]);

/// `uintN` → [`BigUint`], for every ABI uint width (including non-native ones).
macro_rules! impl_abi_decode_uint_bigint {
    ([$($bits:literal),* $(,)?]) => {$(
        impl AbiDecode<Uint<$bits>> for BigUint {
            fn decode(value: AbiValue) -> Result<Self, AbiDecodeError> {
                as_biguint(value)
            }
        }
    )*};
}

impl_abi_decode_uint_bigint!([
    8, 16, 24, 32, 40, 48, 56, 64, 72, 80, 88, 96, 104, 112, 120, 128, 136, 144, 152, 160, 168,
    176, 184, 192, 200, 208, 216, 224, 232, 240, 248, 256, 264, 272, 280, 288, 296, 304, 312, 320,
    328, 336, 344, 352, 360, 368, 376, 384, 392, 400, 408, 416, 424, 432, 440, 448, 456, 464, 472,
    480, 488, 496, 504, 512
]);

impl AbiDecode<Byte> for u8 {
    fn decode(value: AbiValue) -> Result<Self, AbiDecodeError> {
        match value {
            AbiValue::Byte(b) => Ok(b),
            AbiValue::Int(n) => u8::try_from(&n)
                .map_err(|_| AbiDecodeError(format!("byte value {n} does not fit u8"))),
            other => Err(AbiDecodeError(format!("expected a byte, got {other:?}"))),
        }
    }
}

impl AbiDecode<Bool> for bool {
    fn decode(value: AbiValue) -> Result<Self, AbiDecodeError> {
        match value {
            AbiValue::Bool(b) => Ok(b),
            other => Err(AbiDecodeError(format!("expected a bool, got {other:?}"))),
        }
    }
}

impl AbiDecode<Address> for CoreAddress {
    fn decode(value: AbiValue) -> Result<Self, AbiDecodeError> {
        match value {
            AbiValue::Address(a) => Ok(a),
            other => Err(AbiDecodeError(format!(
                "expected an address, got {other:?}"
            ))),
        }
    }
}

impl AbiDecode<AbiString> for String {
    fn decode(value: AbiValue) -> Result<Self, AbiDecodeError> {
        match value {
            AbiValue::String(s) => Ok(s),
            other => Err(AbiDecodeError(format!("expected a string, got {other:?}"))),
        }
    }
}

impl AbiDecode<Bytes> for Vec<u8> {
    fn decode(value: AbiValue) -> Result<Self, AbiDecodeError> {
        match value {
            // `byte[]` decodes to an array of `Byte` elements.
            AbiValue::Array(items) => items
                .into_iter()
                .map(|item| match item {
                    AbiValue::Byte(b) => Ok(b),
                    AbiValue::Int(n) => u8::try_from(&n)
                        .map_err(|_| AbiDecodeError(format!("byte value {n} does not fit u8"))),
                    other => Err(AbiDecodeError(format!("expected a byte, got {other:?}"))),
                })
                .collect(),
            other => Err(AbiDecodeError(format!("expected byte[], got {other:?}"))),
        }
    }
}

/// Pull the elements out of an [`AbiValue::Array`], erroring otherwise. The
/// generated array/struct decoders use this to walk a tuple or array value
/// before decoding each element.
pub fn decode_array_items(value: AbiValue) -> Result<Vec<AbiValue>, AbiDecodeError> {
    match value {
        AbiValue::Array(items) => Ok(items),
        other => Err(AbiDecodeError(format!(
            "expected a tuple/array, got {other:?}"
        ))),
    }
}

/// `ufixedNxM` ← the raw, *unscaled* `N`-bit integer, carrying its
/// `M`-decimal-place scale in the type. ARC-4 encodes a `ufixedNxM` exactly as
/// a `uintN`: the on-wire value is `round(real * 10^M)`, so this newtype holds
/// that already-scaled integer directly (a `ufixed64x2` value of `1.50` is
/// `Ufixed::<64, 2>::new(150u64)`). No `AbiValue::UFixed` variant is needed —
/// it shares the `AbiValue::Int` representation and the `uintN` encoder.
///
/// The bit size and precision are const generic so the macro can pin them from
/// the signature and the type-checker rejects mixing, say, a `ufixed64x2` value
/// where a `ufixed64x3` is expected. The width is range-checked at encode time
/// by `AbiType::encode`, as with any `BigUint`-sourced argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ufixed<const BITS: u16, const PRECISION: u16>(BigUint);

impl<const BITS: u16, const PRECISION: u16> Ufixed<BITS, PRECISION> {
    /// Wrap an already-scaled, unscaled-integer value (`round(real * 10^M)`).
    #[inline]
    pub fn new(raw: impl Into<BigUint>) -> Self {
        Ufixed(raw.into())
    }

    /// The raw, unscaled integer this value wraps.
    #[inline]
    pub fn into_raw(self) -> BigUint {
        self.0
    }
}

impl<const BITS: u16, const PRECISION: u16> AbiArg<UFixed<BITS, PRECISION>>
    for Ufixed<BITS, PRECISION>
{
    #[inline]
    fn encode(self) -> AbiValue {
        // ufixed shares the uint wire encoding: emit the raw integer.
        AbiValue::Int(self.0)
    }
}

impl<const BITS: u16, const PRECISION: u16> AbiDecode<UFixed<BITS, PRECISION>>
    for Ufixed<BITS, PRECISION>
{
    fn decode(value: AbiValue) -> Result<Self, AbiDecodeError> {
        // ufixed shares the uint wire encoding: read the raw integer back into
        // the unscaled newtype, mirroring `AbiArg::<UFixed>::encode`.
        Ok(Ufixed::new(as_biguint(value)?))
    }
}

// === MethodInvocation =====================================================

/// A checked ABI method invocation: an [`AbiMethod`] plus its already-encoded
/// value arguments. This is what `abi_call!` expands to; `MethodCall`'s builder
/// consumes it via `.invoke(...)`.
///
/// Construct one with the `abi_call!` macro (compile-time checked) or with
/// [`MethodInvocation::new`] (for runtime-sourced values, e.g. app-spec JSON).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodInvocation {
    method: AbiMethod,
    args: Vec<AbiValue>,
}

impl MethodInvocation {
    /// Pair a method with its already-encoded value arguments. The macro calls
    /// this after type-checking each argument; runtime callers can call it
    /// directly with values they encoded themselves.
    pub fn new(method: AbiMethod, args: Vec<AbiValue>) -> Self {
        MethodInvocation { method, args }
    }

    /// The method being invoked.
    pub fn method(&self) -> &AbiMethod {
        &self.method
    }

    /// The encoded value arguments, in signature order.
    pub fn args(&self) -> &[AbiValue] {
        &self.args
    }

    /// Decompose into the method and its encoded arguments.
    pub fn into_parts(self) -> (AbiMethod, Vec<AbiValue>) {
        (self.method, self.args)
    }
}

#[cfg(test)]
mod decode_tests {
    use super::*;

    #[test]
    fn scalar_decode_is_the_inverse_of_encode() {
        // Each `AbiDecode` impl inverts the matching `AbiArg::encode`.
        let v: AbiValue = AbiArg::<Uint<64>>::encode(42u64);
        assert_eq!(AbiDecode::<Uint<64>>::decode(v), Ok(42u64));

        let v: AbiValue = AbiArg::<Bool>::encode(true);
        assert_eq!(AbiDecode::<Bool>::decode(v), Ok(true));

        let v: AbiValue = AbiArg::<AbiString>::encode("hi".to_owned());
        assert_eq!(AbiDecode::<AbiString>::decode(v), Ok("hi".to_owned()));

        let v: AbiValue = AbiArg::<Bytes>::encode(vec![1u8, 2, 3]);
        assert_eq!(AbiDecode::<Bytes>::decode(v), Ok(vec![1u8, 2, 3]));

        let v: AbiValue = AbiArg::<Uint<256>>::encode(BigUint::from(7u64));
        assert_eq!(AbiDecode::<Uint<256>>::decode(v), Ok(BigUint::from(7u64)));
    }

    #[test]
    fn out_of_range_integer_is_an_error() {
        // A `uint64` value of 300 does not fit a `u8`.
        let v = AbiValue::from(300u64);
        let decoded: Result<u8, _> = AbiDecode::<Uint<8>>::decode(v);
        assert!(decoded.is_err());
    }

    #[test]
    fn shape_mismatch_is_an_error() {
        // A string is not an integer.
        let v = AbiValue::String("nope".to_owned());
        let decoded: Result<u64, _> = AbiDecode::<Uint<64>>::decode(v);
        assert!(decoded.is_err());

        // A scalar is not a tuple/array.
        assert!(decode_array_items(AbiValue::from(1u64)).is_err());
    }
}
