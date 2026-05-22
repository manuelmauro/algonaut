use crate::abi_error::AbiError;
use algonaut_abi_sig::SigType;
use algonaut_core::Address;
use num_bigint::BigUint;
use std::{fmt::Display, str::FromStr};

pub const ADDRESS_BYTE_SIZE: usize = 32;
pub const LENGTH_ENCODE_BYTE_SIZE: usize = 2;
pub const SINGLE_BYTE_SIZE: usize = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbiType {
    UInt { bit_size: u16 },
    Byte,
    UFixed { bit_size: u16, precision: u16 },
    Bool,
    Address,
    StaticArray { len: u16, child_type: Box<AbiType> },
    DynamicArray { child_type: Box<AbiType> },
    String,
    Tuple { len: u16, child_types: Vec<AbiType> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbiValue {
    Bool(bool),
    Byte(u8),
    Int(BigUint),
    Address(Address),
    String(String),
    Array(Vec<AbiValue>),
}

// Ergonomic constructors for the common cases, so callers write
// `AbiValue::from(2u64)` (or `2u64.into()`) instead of
// `AbiValue::Int(BigUint::from(2u64))`. The ARC-4 ABI supports `uint8`
// through `uint512`, hence `BigUint` for the general case; these cover
// the native integer widths that fit without ceremony.
impl From<u64> for AbiValue {
    fn from(n: u64) -> Self {
        AbiValue::Int(BigUint::from(n))
    }
}

impl From<u128> for AbiValue {
    fn from(n: u128) -> Self {
        AbiValue::Int(BigUint::from(n))
    }
}

impl From<bool> for AbiValue {
    fn from(b: bool) -> Self {
        AbiValue::Bool(b)
    }
}

impl From<Address> for AbiValue {
    fn from(a: Address) -> Self {
        AbiValue::Address(a)
    }
}

impl From<&str> for AbiValue {
    fn from(s: &str) -> Self {
        AbiValue::String(s.to_owned())
    }
}

impl From<String> for AbiValue {
    fn from(s: String) -> Self {
        AbiValue::String(s)
    }
}

/// A `byte[]` ABI value: each byte becomes an [`AbiValue::Byte`] element of
/// a dynamic array, the canonical ARC-4 representation.
impl From<Vec<u8>> for AbiValue {
    fn from(bytes: Vec<u8>) -> Self {
        AbiValue::Array(bytes.into_iter().map(AbiValue::Byte).collect())
    }
}

impl AbiType {
    /// Returns true if the type has children and any of the children is dynamic, false otherwise.
    fn has_dynamic_child(&self) -> bool {
        match self {
            AbiType::StaticArray { child_type, .. } | AbiType::DynamicArray { child_type, .. } => {
                child_type.is_dynamic()
            }
            AbiType::Tuple { child_types, .. } => child_types.iter().any(|t| t.is_dynamic()),
            _ => false,
        }
    }

    /// Returns references to element's children. Variants that don't specify children return an empty vector.
    pub fn children(&self) -> &[AbiType] {
        match self {
            AbiType::StaticArray { child_type, .. } | AbiType::DynamicArray { child_type, .. } => {
                std::slice::from_ref(child_type)
            }
            AbiType::Tuple { child_types, .. } => child_types,
            _ => &[],
        }
    }

    /// Determines whether the ABI type is dynamic or static.
    pub fn is_dynamic(&self) -> bool {
        match self {
            AbiType::DynamicArray { .. } | AbiType::String => true,
            _ => self.has_dynamic_child(),
        }
    }
}

impl Display for AbiType {
    /// Serialize an ABI Type to a string in ABI encoding.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            AbiType::UInt { bit_size } => format!("uint{}", bit_size),
            AbiType::Byte => "byte".to_owned(),
            AbiType::UFixed {
                bit_size,
                precision,
            } => format!("ufixed{}x{}", bit_size, precision),
            AbiType::Bool => "bool".to_owned(),
            AbiType::StaticArray { len, child_type } => {
                format!("{}[{}]", child_type, len)
            }
            AbiType::DynamicArray { child_type } => format!("{}[]", child_type),
            AbiType::String => "string".to_owned(),
            AbiType::Address => "address".to_owned(),
            AbiType::Tuple { child_types, .. } => {
                let mut type_strings = Vec::with_capacity(child_types.len());
                for child_type in child_types {
                    type_strings.push(child_type.to_string())
                }
                format!("({})", type_strings.join(","))
            }
        };
        write!(f, "{}", str)
    }
}

impl AbiType {
    pub fn dynamic_array(arg_type: AbiType) -> AbiType {
        AbiType::DynamicArray {
            child_type: Box::new(arg_type),
        }
    }

    pub fn static_array(arg_type: AbiType, array_len: u16) -> AbiType {
        AbiType::StaticArray {
            len: array_len,
            child_type: Box::new(arg_type),
        }
    }

    /// Makes `Uint` ABI type by taking a type bitSize argument.
    /// The range of type bitSize is [8, 512] and type bitSize % 8 == 0.
    pub fn uint(type_size: usize) -> Result<AbiType, AbiError> {
        if !type_size.is_multiple_of(8) || !(8..=512).contains(&type_size) {
            return Err(AbiError::TypeParse {
                input: format!("uint{type_size}"),
                reason: "bit size must be 8..=512 and a multiple of 8".to_owned(),
            });
        }

        Ok(AbiType::UInt {
            bit_size: type_size as u16,
        })
    }

    pub fn address() -> AbiType {
        AbiType::Address
    }

    pub fn byte() -> AbiType {
        AbiType::Byte
    }

    pub fn bool() -> AbiType {
        AbiType::Bool
    }

    pub fn string() -> AbiType {
        AbiType::String
    }

    /// Makes `UFixed` ABI type by taking type bitSize and type precision as arguments.
    /// The range of type bitSize is [8, 512] and type bitSize % 8 == 0.
    /// The range of type precision is [1, 160].
    pub fn ufixed(type_size: usize, type_precision: usize) -> Result<AbiType, AbiError> {
        if !type_size.is_multiple_of(8) || !(8..=512).contains(&type_size) {
            return Err(AbiError::TypeParse {
                input: format!("ufixed{type_size}x{type_precision}"),
                reason: "bit size must be 8..=512 and a multiple of 8".to_owned(),
            });
        }
        if !(1..=160).contains(&type_precision) {
            return Err(AbiError::TypeParse {
                input: format!("ufixed{type_size}x{type_precision}"),
                reason: "precision must be 1..=160".to_owned(),
            });
        }

        Ok(AbiType::UFixed {
            bit_size: type_size as u16,       // cast: safe bounds checked in this fn
            precision: type_precision as u16, // cast: safe bounds checked in this fn
        })
    }

    /// Makes tuple ABI type with argument types
    pub fn tuple(argument_types: Vec<AbiType>) -> Result<AbiType, AbiError> {
        if argument_types.len() >= u16::MAX as usize {
            return Err(AbiError::TypeParse {
                input: format!("tuple with {} types", argument_types.len()),
                reason: "tuple type count exceeds uint16 maximum".to_owned(),
            });
        }

        Ok(AbiType::Tuple {
            len: argument_types.len() as u16, // cast: safe bounds checked in this fn
            child_types: argument_types,
        })
    }
}

impl FromStr for AbiType {
    type Err = AbiError;

    /// Parses an ABI type string, e.g. `"(uint64,byte[])"`.
    ///
    /// Delegates to the shared [`algonaut_abi_sig`] grammar, so the runtime
    /// parser and the `abi_call!`/`abi_method!` macros accept exactly the same
    /// inputs — a signature the macros reject at compile time fails here too,
    /// and vice-versa.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        algonaut_abi_sig::parse_type(s)
            .map(AbiType::from_sig_type)
            .map_err(AbiError::from)
    }
}

impl AbiType {
    /// Lift a grammar [`SigType`] into the richer [`AbiType`]. The grammar
    /// crate carries no encoding logic, so this is a pure structural mapping.
    fn from_sig_type(ty: SigType) -> AbiType {
        match ty {
            SigType::UInt { bit_size } => AbiType::UInt { bit_size },
            SigType::Byte => AbiType::Byte,
            SigType::UFixed {
                bit_size,
                precision,
            } => AbiType::UFixed {
                bit_size,
                precision,
            },
            SigType::Bool => AbiType::Bool,
            SigType::Address => AbiType::Address,
            SigType::StaticArray { len, child_type } => AbiType::StaticArray {
                len,
                child_type: Box::new(AbiType::from_sig_type(*child_type)),
            },
            SigType::DynamicArray { child_type } => AbiType::DynamicArray {
                child_type: Box::new(AbiType::from_sig_type(*child_type)),
            },
            SigType::String => AbiType::String,
            SigType::Tuple { child_types } => AbiType::Tuple {
                // The grammar already caps tuple arity at `u16::MAX`.
                len: child_types.len() as u16,
                child_types: child_types
                    .into_iter()
                    .map(AbiType::from_sig_type)
                    .collect(),
            },
        }
    }
}
