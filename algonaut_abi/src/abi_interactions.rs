use super::abi_type::AbiType;
use crate::abi_error::AbiError;
use algonaut_core::{error::CoreError, TransactionTypeEnum};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::{collections::HashMap, convert::TryInto};

// AnyTransactionType is the ABI argument type string for a nonspecific transaction argument
pub const ANY_TRANSACTION_TYPE: &str = "txn";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionArgType {
    Any, // denotes a placeholder for any of the types below
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
            CoreError::General(msg) => Self::Msg(msg),
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
            _ => Err(AbiError::Msg(format!(
                "Not supported reference arg type api string: {s}"
            ))),
        }
    }

    fn is_valid_api_str(s: &str) -> bool {
        Self::from_api_str(s).is_ok()
    }
}

/// Represents an ABI Method argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiMethodArg {
    /// User-friendly name for the argument
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The type of the argument as a string.
    /// See [get_type_object](get_type_object) to obtain the ABI type object
    #[serde(rename = "type")]
    pub(crate) type_: String,

    /// User-friendly description for the argument
    #[serde(rename = "desc", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Cache that holds the parsed type object
    #[serde(skip)]
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
            _ => Err(AbiError::Msg(format!(
                "The arg: {type_:?} is not an ABI object."
            ))),
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
pub struct AbiReturn {
    /// The type of the argument as a string. See the [get_type_object](get_type_object) to
    /// obtain the ABI type object
    #[serde(rename = "type")]
    pub(crate) type_: String,

    /// User-friendly description for the argument
    #[serde(rename = "desc", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Cache that holds the parsed type object
    #[serde(skip)]
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
pub struct AbiMethod {
    /// The name of the method
    #[serde(rename = "name")]
    pub name: String,

    /// User-friendly description for the method
    #[serde(rename = "desc", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The arguments of the method, in order
    #[serde(default, rename = "args", skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<AbiMethodArg>,

    /// Information about the method's return value
    #[serde(rename = "returns")]
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
    pub fn from_signature(method_str: &str) -> Result<AbiMethod, AbiError> {
        let open_idx = method_str.chars().position(|c| c == '(').ok_or_else(|| {
            AbiError::Msg("method signature is missing an open parenthesis".to_owned())
        })?;

        let name = &method_str[..open_idx];
        if name.is_empty() {
            return Err(AbiError::Msg(
                "method must have a non empty name".to_owned(),
            ));
        }

        let (arg_types, close_idx) = parse_method_args(method_str, open_idx)?;

        let mut return_type = AbiReturn {
            type_: method_str[close_idx + 1..].to_owned(),
            description: None,
            parsed: None,
        };

        // fill type object cache
        return_type.type_()?;

        let mut args: Vec<AbiMethodArg> = Vec::with_capacity(arg_types.len());

        for (i, arg_type) in arg_types.into_iter().enumerate() {
            let arg = AbiMethodArg {
                type_: arg_type.clone(),
                name: None,
                description: None,
                parsed: None,
            };
            args.push(arg);

            if TransactionArgType::is_valid_api_str(&arg_type)
                || ReferenceArgType::is_valid_api_str(&arg_type)
            {
                continue;
            }

            // fill type object cache
            args[i].type_()?;
        }

        Ok(AbiMethod {
            name: name.to_owned(),
            args,
            returns: return_type,
            description: None,
        })
    }
}

/// Parses the arguments from a method signature string.
/// str_method is the complete method signature and start_idx is the index of the
/// opening parenthesis of the arguments list. This function returns a list of
/// the argument types from the method signature and the index of the closing
/// parenthesis of the arguments list.
fn parse_method_args(str_method: &str, start_idx: usize) -> Result<(Vec<String>, usize), AbiError> {
    // handle no args
    if start_idx < str_method.len() - 1 && str_method.chars().nth(start_idx + 1) == Some(')') {
        return Ok((vec![], start_idx + 1));
    }

    let mut arg_types = vec![];

    let mut paren_cnt = 1;
    let mut prev_pos = start_idx + 1;
    let mut close_idx = None;
    let init_prev_pos = prev_pos;

    for cur_pos in init_prev_pos..str_method.len() {
        let chars = str_method.chars().collect::<Vec<_>>();
        if chars[cur_pos] == '(' {
            paren_cnt += 1;
        } else if chars[cur_pos] == ')' {
            paren_cnt -= 1;
        }

        if paren_cnt < 0 {
            return Err(AbiError::Msg(
                "method signature parentheses mismatch".to_owned(),
            ));
        } else if paren_cnt > 1 {
            continue;
        }

        if chars[cur_pos] == ',' || paren_cnt == 0 {
            let str_arg = &str_method[prev_pos..cur_pos];
            arg_types.push(str_arg.to_owned());
            prev_pos = cur_pos + 1;
        }

        if paren_cnt == 0 {
            close_idx = Some(cur_pos);
            break;
        }
    }

    if let Some(close_idx) = close_idx {
        Ok((arg_types, close_idx))
    } else {
        Err(AbiError::Msg(
            "method signature parentheses mismatch".to_owned(),
        ))
    }
}

/// Represents an ABI interface, which is a logically grouped collection of methods
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbiInterface {
    /// The name of the interface
    #[serde(rename = "name")]
    pub name: String,

    /// User-friendly description for the interface
    #[serde(rename = "desc", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The methods that the interface contains
    #[serde(rename = "methods", skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<AbiMethod>,
}

/// ContractNetworkInfo contains network-specific information about the contract
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbiContractNetworkInfo {
    /// The application ID of the contract for this network
    #[serde(rename = "appID")]
    pub app_id: u64,
}

/// Represents an ABI contract, which is a concrete set of methods implemented by a single app
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbiContract {
    /// The name of the contract
    #[serde(rename = "name")]
    pub name: String,

    /// User-friendly description for the contract
    #[serde(rename = "desc", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional information about the contract's instances across different networks
    #[serde(
        default,
        rename = "networks",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub networks: HashMap<String, AbiContractNetworkInfo>,

    /// The methods that the interface contains
    #[serde(default, rename = "methods", skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<AbiMethod>,
}
