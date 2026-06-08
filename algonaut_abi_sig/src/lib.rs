//! Pure ARC-4 ABI signature and type grammar.
//!
//! This crate is the single source of truth for *what a valid ABI type and
//! method signature look like*. It is intentionally free of I/O and heavy
//! dependencies so that two very different consumers can share exactly the
//! same grammar:
//!
//! - [`algonaut_abi`](https://docs.rs/algonaut_abi)'s runtime parser
//!   (`AbiType::from_str`, `AbiMethod::from_signature`) maps the AST produced
//!   here onto its richer `AbiType`/`AbiMethod` types; and
//! - the `algonaut_abi_macros` `contract!` client generator validates the
//!   spec's method signatures at compile time and synthesizes per-argument
//!   marker types from the same AST.
//!
//! Because both paths call into this crate, a signature accepted (or rejected)
//! by the `contract!` macro is accepted (or rejected) identically by
//! `from_signature`, and vice-versa.
//!
//! The two entry points are [`parse_type`] (a single ABI type, e.g.
//! `"(uint64,byte[])"`) and [`parse_signature`] (a full method signature, e.g.
//! `"add(uint64,uint64)uint64"`).

use std::fmt;

/// A parsed ABI type. Structurally mirrors `algonaut_abi::abi_type::AbiType`,
/// but carries no encoding logic — it is the output of the grammar only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SigType {
    UInt { bit_size: u16 },
    Byte,
    UFixed { bit_size: u16, precision: u16 },
    Bool,
    Address,
    StaticArray { len: u16, child_type: Box<SigType> },
    DynamicArray { child_type: Box<SigType> },
    String,
    Tuple { child_types: Vec<SigType> },
}

/// A grammar error: the offending input plus a human-readable reason. The
/// `reason` is what the proc-macros surface in `compile_error!` and what
/// `algonaut_abi` threads into `AbiError`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SigError {
    pub input: String,
    pub reason: String,
}

impl SigError {
    fn new(input: impl Into<String>, reason: impl Into<String>) -> Self {
        SigError {
            input: input.into(),
            reason: reason.into(),
        }
    }
}

impl fmt::Display for SigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid ABI grammar {:?}: {}", self.input, self.reason)
    }
}

impl std::error::Error for SigError {}

/// Transaction-argument type names recognised in a method signature. `"txn"`
/// matches any transaction; the rest mirror
/// `algonaut_core::TransactionTypeEnum::from_api_str`.
pub const TRANSACTION_ARG_TYPES: &[&str] =
    &["txn", "pay", "keyreg", "acfg", "axfer", "afrz", "appl"];

/// Reference-argument type names recognised in a method signature.
pub const REFERENCE_ARG_TYPES: &[&str] = &["account", "asset", "application"];

/// Whether `s` names a transaction argument (`txn`, `pay`, …).
pub fn is_transaction_arg(s: &str) -> bool {
    TRANSACTION_ARG_TYPES.contains(&s)
}

/// Whether `s` names a reference argument (`account`, `asset`, `application`).
pub fn is_reference_arg(s: &str) -> bool {
    REFERENCE_ARG_TYPES.contains(&s)
}

/// One classified argument of a method signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgClass {
    /// A value argument with a concrete ABI type (`uint64`, `byte[]`, …).
    Value(SigType),
    /// A transaction argument (`txn`, `pay`, …); carries the raw type name.
    Transaction(String),
    /// A reference argument (`account`, `asset`, `application`).
    Reference(String),
}

/// A method's return type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReturnKind {
    Void,
    Type(SigType),
}

/// A fully parsed method signature: name, classified arguments, return type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    pub name: String,
    pub args: Vec<ArgClass>,
    pub ret: ReturnKind,
}

impl Signature {
    /// The number of trailing call arguments this signature consumes — one per
    /// argument slot, regardless of kind (matching the positional `format!`
    /// model where every specifier consumes exactly one argument).
    pub fn arg_count(&self) -> usize {
        self.args.len()
    }
}

/// Parse a single ABI type string into a [`SigType`].
///
/// ```
/// # use algonaut_abi_sig::{parse_type, SigType};
/// assert_eq!(parse_type("uint64").unwrap(), SigType::UInt { bit_size: 64 });
/// assert!(parse_type("unt64").is_err());
/// ```
pub fn parse_type(s: &str) -> Result<SigType, SigError> {
    // Dynamic array: `T[]`. Checked before the static-array case so the empty
    // brackets are not mistaken for a (missing) length.
    if let Some(inner) = s.strip_suffix("[]") {
        return Ok(SigType::DynamicArray {
            child_type: Box::new(parse_type(inner)?),
        });
    }

    // Static array: `T[N]`, with `N` a positive decimal with no leading zero.
    if s.ends_with(']') {
        let open = s
            .rfind('[')
            .ok_or_else(|| SigError::new(s, "invalid static array syntax"))?;
        let inner = &s[..open];
        let len_str = &s[open + 1..s.len() - 1];
        if inner.is_empty() {
            return Err(SigError::new(s, "static array has no element type"));
        }
        let len = parse_array_len(len_str)
            .ok_or_else(|| SigError::new(s, "invalid static array length"))?;
        return Ok(SigType::StaticArray {
            len,
            child_type: Box::new(parse_type(inner)?),
        });
    }

    // uintN
    if let Some(rest) = s.strip_prefix("uint") {
        let bits: u64 = rest
            .parse()
            .map_err(|_| SigError::new(s, "cannot parse uint bit size"))?;
        return make_uint(s, bits);
    }

    if s == "byte" {
        return Ok(SigType::Byte);
    }

    // ufixedNxM
    if let Some(rest) = s.strip_prefix("ufixed") {
        let (size_str, prec_str) = rest
            .split_once('x')
            .ok_or_else(|| SigError::new(s, "invalid ufixed syntax"))?;
        let bits = parse_positive_decimal(size_str)
            .ok_or_else(|| SigError::new(s, "invalid ufixed bit size"))?;
        let precision = parse_positive_decimal(prec_str)
            .ok_or_else(|| SigError::new(s, "invalid ufixed precision"))?;
        return make_ufixed(s, bits, precision);
    }

    if s == "bool" {
        return Ok(SigType::Bool);
    }
    if s == "address" {
        return Ok(SigType::Address);
    }
    if s == "string" {
        return Ok(SigType::String);
    }

    // Tuple: `(T0,T1,...)`, possibly empty `()`.
    if s.len() >= 2 && s.starts_with('(') && s.ends_with(')') {
        let parts = split_tuple_content(&s[1..s.len() - 1])?;
        let mut child_types = Vec::with_capacity(parts.len());
        for part in parts {
            child_types.push(parse_type(&part)?);
        }
        if child_types.len() >= u16::MAX as usize {
            return Err(SigError::new(s, "tuple element count exceeds u16 maximum"));
        }
        return Ok(SigType::Tuple { child_types });
    }

    Err(SigError::new(s, "unrecognized ABI type"))
}

/// Parse a full method signature, classifying each argument.
///
/// ```
/// # use algonaut_abi_sig::{parse_signature, ArgClass, ReturnKind, SigType};
/// let sig = parse_signature("add(uint64,uint64)uint64").unwrap();
/// assert_eq!(sig.name, "add");
/// assert_eq!(sig.args.len(), 2);
/// assert_eq!(sig.ret, ReturnKind::Type(SigType::UInt { bit_size: 64 }));
/// ```
pub fn parse_signature(method: &str) -> Result<Signature, SigError> {
    let RawSignature { name, args, ret } = split_signature(method)?;

    let mut classified = Vec::with_capacity(args.len());
    for arg in &args {
        classified.push(classify_arg(arg)?);
    }

    let ret = if ret == "void" {
        ReturnKind::Void
    } else {
        ReturnKind::Type(parse_type(&ret)?)
    };

    Ok(Signature {
        name,
        args: classified,
        ret,
    })
}

/// The raw, unclassified split of a method signature: name, argument type
/// strings, and the return-type string. `algonaut_abi`'s `from_signature`
/// builds on this directly so its `AbiMethodArg`s keep their original lazy
/// classification semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawSignature {
    pub name: String,
    pub args: Vec<String>,
    pub ret: String,
}

/// Split a method signature into name, argument type strings, and return type
/// without classifying or fully parsing the arguments.
pub fn split_signature(method: &str) -> Result<RawSignature, SigError> {
    let open = method
        .find('(')
        .ok_or_else(|| SigError::new(method, "missing an open parenthesis"))?;

    let name = &method[..open];
    if name.is_empty() {
        return Err(SigError::new(method, "method must have a non-empty name"));
    }

    let (args, close) = parse_method_args(method, open)?;
    let ret = method[close + 1..].to_owned();

    Ok(RawSignature {
        name: name.to_owned(),
        args,
        ret,
    })
}

/// Classify a single signature argument as a value (an ABI type), a
/// transaction type (`pay`, `axfer`, …), or a reference type (`account`,
/// `asset`, `application`).
pub fn classify_arg(s: &str) -> Result<ArgClass, SigError> {
    if is_transaction_arg(s) {
        Ok(ArgClass::Transaction(s.to_owned()))
    } else if is_reference_arg(s) {
        Ok(ArgClass::Reference(s.to_owned()))
    } else {
        Ok(ArgClass::Value(parse_type(s)?))
    }
}

/// `bits` must be in `[8, 512]` and a multiple of 8.
fn make_uint(input: &str, bits: u64) -> Result<SigType, SigError> {
    if !bits.is_multiple_of(8) || !(8..=512).contains(&bits) {
        return Err(SigError::new(
            input,
            "uint bit size must be 8..=512 and a multiple of 8",
        ));
    }
    Ok(SigType::UInt {
        bit_size: bits as u16, // bounds checked above
    })
}

/// `bits` in `[8, 512]` multiple of 8; `precision` in `[1, 160]`.
fn make_ufixed(input: &str, bits: u64, precision: u64) -> Result<SigType, SigError> {
    if !bits.is_multiple_of(8) || !(8..=512).contains(&bits) {
        return Err(SigError::new(
            input,
            "ufixed bit size must be 8..=512 and a multiple of 8",
        ));
    }
    if !(1..=160).contains(&precision) {
        return Err(SigError::new(input, "ufixed precision must be 1..=160"));
    }
    Ok(SigType::UFixed {
        bit_size: bits as u16,       // bounds checked above
        precision: precision as u16, // bounds checked above
    })
}

/// Parse a static-array length: a positive decimal with no leading zero, fit
/// into `u16`. Returns `None` on any violation.
fn parse_array_len(s: &str) -> Option<u16> {
    parse_positive_decimal(s).and_then(|n| u16::try_from(n).ok())
}

/// Parse `[1-9][0-9]*` into a `u64`: non-empty, all ASCII digits, no leading
/// zero. Returns `None` otherwise. This is the lexical rule the original
/// regexes (`[1-9][\d]*`) enforced for ufixed sizes and array lengths.
fn parse_positive_decimal(s: &str) -> Option<u64> {
    if s.is_empty() || s.as_bytes()[0] == b'0' || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

/// Parse the arguments from a method signature string. `start_idx` is the index
/// of the opening parenthesis. Returns the argument type strings and the index
/// of the matching closing parenthesis. Paren-aware so nested tuple arguments
/// are split at the correct commas.
fn parse_method_args(method: &str, start_idx: usize) -> Result<(Vec<String>, usize), SigError> {
    let bytes = method.as_bytes();

    // No-args fast path: `name()`.
    if start_idx + 1 < bytes.len() && bytes[start_idx + 1] == b')' {
        return Ok((vec![], start_idx + 1));
    }

    let mut arg_types = vec![];
    let mut paren_cnt: i32 = 1;
    let mut prev_pos = start_idx + 1;
    let mut close_idx = None;

    for cur_pos in (start_idx + 1)..bytes.len() {
        match bytes[cur_pos] {
            b'(' => paren_cnt += 1,
            b')' => paren_cnt -= 1,
            _ => {}
        }

        if paren_cnt < 0 {
            return Err(SigError::new(method, "parentheses mismatch"));
        } else if paren_cnt > 1 {
            continue;
        }

        if bytes[cur_pos] == b',' || paren_cnt == 0 {
            arg_types.push(method[prev_pos..cur_pos].to_owned());
            prev_pos = cur_pos + 1;
        }

        if paren_cnt == 0 {
            close_idx = Some(cur_pos);
            break;
        }
    }

    close_idx
        .map(|close| (arg_types, close))
        .ok_or_else(|| SigError::new(method, "parentheses mismatch"))
}

/// Split the comma-separated content *between* a tuple's outer parentheses
/// into its top-level element strings, respecting nested parentheses. The
/// argument is the slice inside `(` … `)`.
fn split_tuple_content(content: &str) -> Result<Vec<String>, SigError> {
    if content.is_empty() {
        return Ok(vec![]);
    }

    let owned = || format!("({content})");

    if content.starts_with(',') || content.ends_with(',') {
        return Err(SigError::new(
            owned(),
            "tuple content must not start or end with a comma",
        ));
    }
    if content.contains(",,") {
        return Err(SigError::new(owned(), "consecutive commas not allowed"));
    }

    let bytes = content.as_bytes();
    let mut parts = Vec::new();
    let mut depth: i32 = 0;
    let mut start = 0usize;

    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth < 0 {
                    return Err(SigError::new(owned(), "unpaired closing parenthesis"));
                }
            }
            b',' if depth == 0 => {
                parts.push(content[start..i].to_owned());
                start = i + 1;
            }
            _ => {}
        }
    }

    if depth != 0 {
        return Err(SigError::new(owned(), "unpaired opening parenthesis"));
    }

    parts.push(content[start..].to_owned());
    Ok(parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uint(b: u16) -> SigType {
        SigType::UInt { bit_size: b }
    }
    fn dynarr(t: SigType) -> SigType {
        SigType::DynamicArray {
            child_type: Box::new(t),
        }
    }
    fn statarr(t: SigType, len: u16) -> SigType {
        SigType::StaticArray {
            len,
            child_type: Box::new(t),
        }
    }

    #[test]
    fn simple_types() {
        assert_eq!(parse_type("byte").unwrap(), SigType::Byte);
        assert_eq!(parse_type("bool").unwrap(), SigType::Bool);
        assert_eq!(parse_type("address").unwrap(), SigType::Address);
        assert_eq!(parse_type("string").unwrap(), SigType::String);
        assert_eq!(parse_type("uint64").unwrap(), uint(64));
        assert_eq!(
            parse_type("ufixed256x10").unwrap(),
            SigType::UFixed {
                bit_size: 256,
                precision: 10
            }
        );
    }

    #[test]
    fn uint_bounds() {
        for i in (8..=512).step_by(8) {
            assert_eq!(parse_type(&format!("uint{i}")).unwrap(), uint(i));
        }
        assert!(parse_type("uint0").is_err());
        assert!(parse_type("uint7").is_err());
        assert!(parse_type("uint520").is_err());
        assert!(parse_type("uint65").is_err());
    }

    #[test]
    fn arrays_and_tuples() {
        assert_eq!(parse_type("uint256[]").unwrap(), dynarr(uint(256)));
        assert_eq!(
            parse_type("address[100]").unwrap(),
            statarr(SigType::Address, 100)
        );
        assert_eq!(
            parse_type("uint64[][100]").unwrap(),
            statarr(dynarr(uint(64)), 100)
        );
        assert_eq!(
            parse_type("()").unwrap(),
            SigType::Tuple {
                child_types: vec![]
            }
        );
        assert_eq!(
            parse_type("(uint64,byte[])").unwrap(),
            SigType::Tuple {
                child_types: vec![uint(64), dynarr(SigType::Byte)]
            }
        );
    }

    /// Every invalid case from `algonaut_abi`'s `test_type_from_string_is_invalid`
    /// must stay rejected by the shared grammar.
    #[test]
    fn invalid_types_match_legacy() {
        for case in [
            "uint123x345",
            "uint 128",
            "uint8 ",
            "uint!8",
            "uint[32]",
            "uint-893",
            "uint#120\\",
            "ufixed000000000016x0000010",
            "ufixed123x345",
            "ufixed 128 x 100",
            "ufixed64x10 ",
            "ufixed!8x2 ",
            "ufixed[32]x16",
            "ufixed-64x+100",
            "ufixed16x+12",
            "uint256 []",
            "byte[] ",
            "[][][]",
            "stuff[]",
            "ufixed32x10[0]",
            "byte[10 ]",
            "uint64[0x21]",
            "(ufixed128x10))",
            "(,uint128,byte[])",
            "(address,ufixed64x5,)",
            "(byte[16],somethingwrong)",
            "(                )",
            "((uint32)",
            "(byte,,byte)",
            "((byte),,(byte))",
            "",
        ] {
            assert!(
                parse_type(case).is_err(),
                "expected {case:?} to be rejected"
            );
        }
    }

    #[test]
    fn signatures() {
        let sig = parse_signature("add(uint64,uint64)uint64").unwrap();
        assert_eq!(sig.name, "add");
        assert_eq!(
            sig.args,
            vec![ArgClass::Value(uint(64)), ArgClass::Value(uint(64))]
        );
        assert_eq!(sig.ret, ReturnKind::Type(uint(64)));

        let void = parse_signature("noop()void").unwrap();
        assert_eq!(void.args, vec![]);
        assert_eq!(void.ret, ReturnKind::Void);

        let mixed = parse_signature("m(pay,account,uint64)void").unwrap();
        assert_eq!(
            mixed.args,
            vec![
                ArgClass::Transaction("pay".to_owned()),
                ArgClass::Reference("account".to_owned()),
                ArgClass::Value(uint(64)),
            ]
        );
    }

    #[test]
    fn invalid_signatures() {
        assert!(parse_signature("noparen").is_err());
        assert!(parse_signature("(uint64)void").is_err()); // empty name
        assert!(parse_signature("add(unt64)uint64").is_err()); // typo'd type
        assert!(parse_signature("add(uint64)").is_err()); // empty return type
    }
}
