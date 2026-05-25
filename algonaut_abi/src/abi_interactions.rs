use super::abi_type::AbiType;
use crate::abi_error::AbiError;
use algonaut_abi_model as model;
use algonaut_core::{AppId, TransactionTypeEnum, error::CoreError};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::{collections::HashMap, convert::TryInto};

/// ABI argument type string for a nonspecific transaction argument
pub const ANY_TRANSACTION_TYPE: &str = "txn";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionArgType {
    Any, // placeholder for any of the types below
    One(TransactionTypeEnum),
}

impl TransactionArgType {
    fn from_api_str(s: &str) -> Result<TransactionArgType, AbiError> {
        match s {
            "txn" => Ok(TransactionArgType::Any),
            s => Ok(TransactionTypeEnum::from_api_str(s).map(TransactionArgType::One)?),
        }
    }

    #[allow(dead_code)] // from_api_str counterpart
    fn to_api_str(&self) -> &str {
        match self {
            TransactionArgType::Any => "txn",
            TransactionArgType::One(tx_type_enum) => tx_type_enum.to_api_str(),
        }
    }

    fn is_valid_api_str(s: &str) -> bool {
        Self::from_api_str(s).is_ok()
    }
}

impl From<CoreError> for AbiError {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::Base64Decode(err) => Self::Decode {
                reason: format!("base64 decode: {err}"),
            },
            CoreError::InvalidArraySize { expected, actual } => Self::Decode {
                reason: format!("expected {expected} bytes, got {actual}"),
            },
            CoreError::InvalidTransactionType(s) => Self::TypeParse {
                input: s.clone(),
                reason: format!("invalid transaction type: `{s}`"),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceArgType {
    Account,
    Asset,
    Application,
}

impl ReferenceArgType {
    fn from_api_str(s: &str) -> Result<ReferenceArgType, AbiError> {
        match s {
            "account" => Ok(ReferenceArgType::Account),
            "asset" => Ok(ReferenceArgType::Asset),
            "application" => Ok(ReferenceArgType::Application),
            _ => Err(AbiError::TypeParse {
                input: s.to_owned(),
                reason: "not a supported reference arg type".to_owned(),
            }),
        }
    }

    fn is_valid_api_str(s: &str) -> bool {
        Self::from_api_str(s).is_ok()
    }
}

/// Represents an ABI Method argument
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "model::AbiMethodArg", into = "model::AbiMethodArg")]
pub struct AbiMethodArg {
    /// User-friendly name for the argument
    pub name: Option<String>,

    /// The type of the argument as a string.
    /// See [get_type_object](get_type_object) to obtain the ABI type object
    pub(crate) type_: String,

    /// User-friendly description for the argument
    pub description: Option<String>,

    /// Cache that holds the parsed type object (not part of the JSON shape)
    pub(crate) parsed: Option<AbiType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbiArgType {
    Tx(TransactionArgType),
    Ref(ReferenceArgType),
    AbiObj(AbiType),
}

impl PartialEq for AbiMethodArg {
    fn eq(&self, other: &Self) -> bool {
        // excludes `parsed`, which is just a cache
        self.name == other.name
            && self.type_ == other.type_
            && self.description == other.description
    }
}
impl Eq for AbiMethodArg {}

impl AbiMethodArg {
    pub fn type_(&mut self) -> Result<AbiArgType, AbiError> {
        Ok(if let Some(tx_arg) = self.transaction_arg() {
            AbiArgType::Tx(tx_arg)
        } else if let Some(ref_arg) = self.reference_arg() {
            AbiArgType::Ref(ref_arg)
        } else {
            let type_ = self.type_.parse::<AbiType>()?;
            self.parsed = Some(type_.clone());
            AbiArgType::AbiObj(type_)
        })
    }

    pub fn abi_obj_or_err(&mut self) -> Result<AbiType, AbiError> {
        let type_ = self.type_()?;
        match type_ {
            AbiArgType::AbiObj(obj) => Ok(obj),
            _ => Err(AbiError::TypeParse {
                input: format!("{type_:?}"),
                reason: "not an ABI object".to_owned(),
            }),
        }
    }

    fn is_transaction_arg(&self) -> bool {
        self.transaction_arg().is_some()
    }

    fn transaction_arg(&self) -> Option<TransactionArgType> {
        TransactionArgType::from_api_str(&self.type_).ok()
    }

    fn reference_arg(&self) -> Option<ReferenceArgType> {
        ReferenceArgType::from_api_str(&self.type_).ok()
    }
}

/// Represents an ABI method return value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "model::AbiReturn", into = "model::AbiReturn")]
pub struct AbiReturn {
    /// The type of the argument as a string. See the [get_type_object](get_type_object) to
    /// obtain the ABI type object
    pub(crate) type_: String,

    /// User-friendly description for the argument
    pub description: Option<String>,

    /// Cache that holds the parsed type object (not part of the JSON shape)
    pub(crate) parsed: Option<AbiType>,
}

impl PartialEq for AbiReturn {
    fn eq(&self, other: &Self) -> bool {
        // excludes `parsed`, which is just a cache
        self.type_ == other.type_ && self.description == other.description
    }
}
impl Eq for AbiReturn {}

impl AbiReturn {
    pub fn is_void(&self) -> bool {
        Self::is_void_str(&self.type_)
    }

    pub fn is_void_str(s: &str) -> bool {
        s == "void"
    }

    pub fn type_(&mut self) -> Result<AbiReturnType, AbiError> {
        if self.is_void() {
            Ok(AbiReturnType::Void)
        } else {
            if let Some(parsed) = &self.parsed {
                return Ok(AbiReturnType::Some(parsed.clone()));
            }

            let type_obj = self.type_.parse::<AbiType>()?;
            self.parsed = Some(type_obj.clone());

            Ok(AbiReturnType::Some(type_obj))
        }
    }
}

#[derive(Debug, Clone)]
pub enum AbiReturnType {
    Some(AbiType),
    Void,
}

/// Represents an ABI method return value
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(from = "model::AbiMethod", into = "model::AbiMethod")]
pub struct AbiMethod {
    /// The name of the method
    pub name: String,

    /// User-friendly description for the method
    pub description: Option<String>,

    /// The arguments of the method, in order
    pub args: Vec<AbiMethodArg>,

    /// Information about the method's return value
    pub returns: AbiReturn,
}

impl AbiMethod {
    /// Calculates and returns the signature of the method
    pub fn get_signature(&self) -> String {
        let method_signature = format!("{}{}", self.name, "(");
        let mut str_types: Vec<String> = vec![];
        for arg in &self.args {
            str_types.push(arg.type_.to_owned());
        }
        format!(
            "{method_signature}{}){}",
            str_types.join(","),
            self.returns.type_
        )
    }

    /// Calculates and returns the 4-byte selector of the method
    pub fn get_selector(&self) -> Result<[u8; 4], AbiError> {
        let sig = self.get_signature();
        let sig_hash = sha2::Sha512_256::digest(sig);
        Ok(sig_hash[..4]
            .try_into()
            .expect("Unexpected: couldn't get signature bytes from Sha512_256 digest"))
    }

    /// Returns the number of transactions required to invoke this method
    pub fn get_tx_count(&self) -> usize {
        1 + self.args.iter().filter(|a| a.is_transaction_arg()).count()
    }

    /// Decodes a method signature string into a Method object.
    ///
    /// The signature is split by the shared [`algonaut_abi_sig`] grammar — the
    /// same grammar the `abi_call!`/`abi_method!` macros validate against — and
    /// each argument's ABI type is parsed (and cached) exactly as before. Use
    /// this for signatures that arrive at run time (app-spec JSON, user input);
    /// for compile-time literals, prefer `abi_method!` / `abi_call!`, which
    /// perform this validation at build time.
    pub fn from_signature(method_str: &str) -> Result<AbiMethod, AbiError> {
        let sig = algonaut_abi_sig::split_signature(method_str).map_err(|e| {
            AbiError::MethodSignature {
                input: e.input,
                reason: e.reason,
            }
        })?;

        let mut return_type = AbiReturn {
            type_: sig.ret,
            description: None,
            parsed: None,
        };

        // fill type object cache (also validates the return type)
        return_type.type_()?;

        let mut args: Vec<AbiMethodArg> = Vec::with_capacity(sig.args.len());

        for arg_type in sig.args {
            let mut arg = AbiMethodArg {
                type_: arg_type.clone(),
                name: None,
                description: None,
                parsed: None,
            };

            // Transaction- and reference-typed args have no `AbiType`; for
            // everything else, parse and cache the type object (validating it).
            if !(TransactionArgType::is_valid_api_str(&arg_type)
                || ReferenceArgType::is_valid_api_str(&arg_type))
            {
                arg.type_()?;
            }

            args.push(arg);
        }

        Ok(AbiMethod {
            name: sig.name,
            args,
            returns: return_type,
            description: None,
        })
    }
}

/// Represents an ABI interface, which is a logically grouped collection of methods
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "model::AbiInterface", into = "model::AbiInterface")]
pub struct AbiInterface {
    /// The name of the interface
    pub name: String,

    /// User-friendly description for the interface
    pub description: Option<String>,

    /// The methods that the interface contains
    pub methods: Vec<AbiMethod>,
}

/// Network-specific information about the contract
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    from = "model::AbiContractNetworkInfo",
    into = "model::AbiContractNetworkInfo"
)]
pub struct AbiContractNetworkInfo {
    /// The application ID of the contract for this network
    pub app_id: AppId,
}

/// Represents an ABI contract, which is a concrete set of methods implemented by a single app
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "model::AbiContract", into = "model::AbiContract")]
pub struct AbiContract {
    /// The name of the contract
    pub name: String,

    /// User-friendly description for the contract
    pub description: Option<String>,

    /// Optional information about the contract's instances across different networks
    pub networks: HashMap<String, AbiContractNetworkInfo>,

    /// The methods that the interface contains
    pub methods: Vec<AbiMethod>,
}

// ---------------------------------------------------------------------------
// Conversions to/from the wire model (`algonaut_abi_model`).
//
// The JSON shape is single-sourced in the model crate; these runtime types add
// the lazily-parsed `AbiType` cache and a typed `AppId`, and (de)serialize by
// delegating to the model via the `#[serde(from / into)]` attributes above.
// ---------------------------------------------------------------------------

impl From<model::AbiMethodArg> for AbiMethodArg {
    fn from(m: model::AbiMethodArg) -> Self {
        Self {
            name: m.name,
            type_: m.type_,
            description: m.desc,
            parsed: None,
        }
    }
}

impl From<AbiMethodArg> for model::AbiMethodArg {
    fn from(a: AbiMethodArg) -> Self {
        Self {
            name: a.name,
            type_: a.type_,
            desc: a.description,
        }
    }
}

impl From<model::AbiReturn> for AbiReturn {
    fn from(m: model::AbiReturn) -> Self {
        Self {
            type_: m.type_,
            description: m.desc,
            parsed: None,
        }
    }
}

impl From<AbiReturn> for model::AbiReturn {
    fn from(r: AbiReturn) -> Self {
        Self {
            type_: r.type_,
            desc: r.description,
        }
    }
}

impl From<model::AbiMethod> for AbiMethod {
    fn from(m: model::AbiMethod) -> Self {
        Self {
            name: m.name,
            description: m.desc,
            args: m.args.into_iter().map(Into::into).collect(),
            returns: m.returns.into(),
        }
    }
}

impl From<AbiMethod> for model::AbiMethod {
    fn from(m: AbiMethod) -> Self {
        Self {
            name: m.name,
            desc: m.description,
            args: m.args.into_iter().map(Into::into).collect(),
            returns: m.returns.into(),
        }
    }
}

impl From<model::AbiInterface> for AbiInterface {
    fn from(m: model::AbiInterface) -> Self {
        Self {
            name: m.name,
            description: m.desc,
            methods: m.methods.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<AbiInterface> for model::AbiInterface {
    fn from(i: AbiInterface) -> Self {
        Self {
            name: i.name,
            desc: i.description,
            methods: i.methods.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<model::AbiContractNetworkInfo> for AbiContractNetworkInfo {
    fn from(m: model::AbiContractNetworkInfo) -> Self {
        Self {
            app_id: AppId(m.app_id),
        }
    }
}

impl From<AbiContractNetworkInfo> for model::AbiContractNetworkInfo {
    fn from(n: AbiContractNetworkInfo) -> Self {
        Self { app_id: n.app_id.0 }
    }
}

impl From<model::AbiContract> for AbiContract {
    fn from(m: model::AbiContract) -> Self {
        Self {
            name: m.name,
            description: m.desc,
            networks: m.networks.into_iter().map(|(k, v)| (k, v.into())).collect(),
            methods: m.methods.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<AbiContract> for model::AbiContract {
    fn from(c: AbiContract) -> Self {
        Self {
            name: c.name,
            desc: c.description,
            networks: c.networks.into_iter().map(|(k, v)| (k, v.into())).collect(),
            methods: c.methods.into_iter().map(Into::into).collect(),
        }
    }
}
