use crate::abi_error::AbiError;
use algonaut_core::Address;
use lazy_static::lazy_static;
use num_bigint::BigUint;
use regex::Regex;
use std::{convert::TryInto, fmt::Display, str::FromStr};

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
            AbiType::DynamicArray { .. } | AbiType::String { .. } => true,
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
        if type_size % 8 != 0 || !(8..=512).contains(&type_size) {
            return Err(AbiError::Msg(format!(
                "unsupported uint type bitSize: {type_size}"
            )));
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
        if type_size % 8 != 0 || !(8..=512).contains(&type_size) {
            return Err(AbiError::Msg(format!(
                "unsupported ufixed type bitSize: {type_size}"
            )));
        }
        if !(1..=160).contains(&type_precision) {
            return Err(AbiError::Msg(format!(
                "unsupported ufixed type precision: {type_precision}"
            )));
        }

        Ok(AbiType::UFixed {
            bit_size: type_size as u16,       // cast: safe bounds checked in this fn
            precision: type_precision as u16, // cast: safe bounds checked in this fn
        })
    }

    /// Makes tuple ABI type with argument types
    pub fn tuple(argument_types: Vec<AbiType>) -> Result<AbiType, AbiError> {
        if argument_types.len() >= u16::MAX as usize {
            return Err(AbiError::Msg(
                "tuple type child type number larger than maximum uint16 error".to_owned(),
            ));
        }

        Ok(AbiType::Tuple {
            len: argument_types.len() as u16, // cast: safe bounds checked in this fn
            child_types: argument_types,
        })
    }
}

impl FromStr for AbiType {
    type Err = AbiError;

    /// Parses an ABI type string.
    /// For example: `from_str("(uint64,byte[])")`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(stripped) = s.strip_suffix("[]") {
            let array_arg_type = stripped.parse()?;
            Ok(AbiType::dynamic_array(array_arg_type))
        } else if s.ends_with(']') {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"^([a-z\d\[\](),]+)\[([1-9][\d]*)]$").unwrap();
            }
            let caps = RE.captures(s).ok_or_else(|| {
                AbiError::Msg(format!("Regex for ] ending string: {s} didn't match"))
            })?;

            if caps.len() != 3 {
                return Err(AbiError::Msg(format!("ill formed uint type: {s}")));
            }
            let array_type = caps[1].parse()?;
            let array_len_s = caps[2].to_owned();

            let array_len: usize = array_len_s.parse().map_err(|e| {
                AbiError::Msg(format!("Error parsing array len: {array_len_s}: {e:?}"))
            })?;

            Ok(AbiType::static_array(
                array_type,
                array_len.try_into().map_err(|_| {
                    AbiError::Msg("Couldn't convert array_len: {array_len} in u16".to_owned())
                })?,
            ))
        } else if let Some(stripped) = s.strip_prefix("uint") {
            let type_size = stripped
                .parse()
                .map_err(|e| AbiError::Msg(format!("Ill formed uint type: {s}: {e:?}")))?;

            AbiType::uint(type_size)
        } else if s == "byte" {
            Ok(AbiType::byte())
        } else if s.starts_with("ufixed") {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"^ufixed([1-9][\d]*)x([1-9][\d]*)$").unwrap();
            }
            let caps = RE
                .captures(s)
                .ok_or_else(|| AbiError::Msg(format!("Regex for ufixed: {s} didn't match")))?;

            if caps.len() != 3 {
                return Err(AbiError::Msg(format!("ill formed ufixed type: {s}")));
            }
            let ufixed_size_s = &caps[1].to_owned();
            let ufixed_size = ufixed_size_s.parse().map_err(|e| {
                AbiError::Msg(format!("Error parsing ufixed size: {ufixed_size_s}: {e:?}"))
            })?;

            let ufixed_precision_s = &caps[2].to_owned();
            let ufixed_precision = ufixed_precision_s.parse().map_err(|e| {
                AbiError::Msg(format!(
                    "Error parsing ufixed precision: {ufixed_precision_s}: {e:?}"
                ))
            })?;

            AbiType::ufixed(ufixed_size, ufixed_precision)
        } else if s == "bool" {
            Ok(AbiType::bool())
        } else if s == "address" {
            Ok(AbiType::address())
        } else if s == "string" {
            Ok(AbiType::string())
        } else if s.len() >= 2 && s.starts_with('(') && s.ends_with(')') {
            let tuple_content = parse_tuple_content(&s[1..s.len() - 1])?;
            let mut tuple_types = Vec::with_capacity(tuple_content.len());

            for t in tuple_content {
                let ti = t.parse()?;
                tuple_types.push(ti);
            }

            AbiType::tuple(tuple_types)
        } else {
            Err(AbiError::Msg(format!(
                "cannot convert string: `{s}` to ABI type"
            )))
        }
    }
}

/// Keeps track of the start and end of a segment in a string.
struct Segment {
    left: usize,
    right: usize,
}

/// Splits an ABI encoded string for tuple type into multiple sub-strings.
/// Each sub-string represents a content type of the tuple type.
/// The argument str is the content between parentheses of tuple, i.e.
/// (...... str ......)
///  ^               ^
fn parse_tuple_content(str: &str) -> Result<Vec<String>, AbiError> {
    // if the tuple type content is empty (which is also allowed)
    // just return the empty string list
    if str.is_empty() {
        return Ok(vec![]);
    }

    // the following 2 checks want to make sure input string can be separated by comma
    // with form: "...substr_0,...substr_1,...,...substr_k"

    // str should not have leading/tailing comma
    if str.ends_with(',') || str.starts_with(',') {
        return Err(AbiError::Msg(
            "parsing error: tuple content should not start with comma".to_owned(),
        ));
    }
    // str should not have consecutive commas
    if str.contains(",,") {
        return Err(AbiError::Msg("no consecutive commas".to_owned()));
    }

    let mut paren_segment_record = vec![];
    let mut stack = vec![];

    // get the most exterior parentheses segment (not overlapped by other parentheses)
    // illustration: "*****,(*****),*****" => ["*****", "(*****)", "*****"]
    // once iterate to left paren (, stack up by 1 in stack
    // iterate to right paren ), pop 1 in stack
    // if iterate to right paren ) with stack height 0, find a parenthesis segment "(******)"
    for (index, chr) in str.chars().enumerate() {
        if chr == '(' {
            stack.push(index);
        } else if chr == ')' {
            if stack.is_empty() {
                return Err(AbiError::Msg(format!("unpaired parentheses: {str}")));
            }

            let left_paren_index = stack[stack.len() - 1];
            stack.pop();
            if stack.is_empty() {
                paren_segment_record.push(Segment {
                    left: left_paren_index,
                    right: index,
                });
            }
        }
    }
    if !stack.is_empty() {
        return Err(AbiError::Msg(format!("unpaired parentheses: {str}")));
    }

    // take out tuple-formed type str in tuple argument
    let mut str_copied = str.to_owned();

    for paren_seg in paren_segment_record.iter().rev() {
        str_copied = format!(
            "{}{}",
            str_copied[..paren_seg.left].to_owned(),
            str_copied[paren_seg.right + 1..].to_owned()
        );
    }

    // split the string without parenthesis segments
    let tuple_str_segs: Vec<&str> = str_copied.split(',').collect();
    let mut tuple_str_segs_res: Vec<String> = Vec::with_capacity(tuple_str_segs.len());

    // the empty strings are placeholders for parenthesis segments
    // put the parenthesis segments back into segment list
    let mut paren_seg_count = 0;
    for seg_str in tuple_str_segs.iter() {
        if seg_str.is_empty() {
            let paren_seg = &paren_segment_record[paren_seg_count];
            tuple_str_segs_res.push(str[paren_seg.left..paren_seg.right + 1].to_owned());
            paren_seg_count += 1;
        } else {
            tuple_str_segs_res.push((*seg_str).to_owned());
        }
    }

    Ok(tuple_str_segs_res)
}
