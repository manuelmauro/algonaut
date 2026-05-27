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
/// `ufixedNxM` marker. No `AbiArg` impl yet (no native Rust type); present so
/// the macro can name the slot.
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

// === base64 helpers =======================================================
//
// The `contract!` state accessors compute storage keys at run time and compare
// them against algod's base64-encoded key strings. These keep the generated
// code free of a direct `base64` dependency, which a downstream consumer of the
// macro might not have — generated code already names `algonaut_abi`.

/// Base64-encode `bytes` with the standard alphabet (matches algod's key
/// encoding for global/local state entries).
pub fn b64_encode(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// Base64-decode `s` with the standard alphabet. Returns `None` on malformed
/// input.
pub fn b64_decode(s: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(s).ok()
}
