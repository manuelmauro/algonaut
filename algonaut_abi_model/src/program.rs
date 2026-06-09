//! Deploy and compile metadata: the program pair, source maps, compiler info,
//! and TEAL template / scratch variables.

use serde::{Deserialize, Serialize};

/// A base64-encoded approval/clear program pair (used by `source` and
/// `byteCode`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgramPair {
    /// Approval program, base64-encoded.
    pub approval: String,

    /// Clear-state program, base64-encoded.
    pub clear: String,
}

/// Source-map info for the approval and clear programs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgramSourceInfoPair {
    /// Approval-program source info.
    pub approval: ProgramSourceInfo,

    /// Clear-state-program source info.
    pub clear: ProgramSourceInfo,
}

/// Per-program source info: a list of source entries and the PC-offset method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgramSourceInfo {
    /// Per-PC source entries.
    #[serde(rename = "sourceInfo", default, skip_serializing_if = "Vec::is_empty")]
    pub source_info: Vec<SourceInfo>,

    /// How program-counter offsets are encoded: `"none"` or `"cblocks"`.
    #[serde(rename = "pcOffsetMethod")]
    pub pc_offset_method: String,
}

/// A single source-map entry tying program counters to source/teal/error info.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceInfo {
    /// Program-counter values this entry applies to.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pc: Vec<u64>,

    /// Error message associated with these PCs, if any.
    #[serde(
        rename = "errorMessage",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub error_message: Option<String>,

    /// 1-based line number in the compiled TEAL, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teal: Option<u64>,

    /// 1-based line number in the original source, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Which compiler produced the contract, and its version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerInfo {
    /// Compiler name, e.g. `"algod"` or `"puya"`.
    pub compiler: String,

    /// Compiler version.
    #[serde(rename = "compilerVersion")]
    pub compiler_version: CompilerVersion,
}

/// A semantic compiler version, with an optional commit hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerVersion {
    /// Major version.
    pub major: u64,

    /// Minor version.
    pub minor: u64,

    /// Patch version.
    pub patch: u64,

    /// Optional source commit hash.
    #[serde(
        rename = "commitHash",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub commit_hash: Option<String>,
}

/// A TEAL template variable: its encoding and an optional value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Encoding of the variable (ABI type, AVM type, or struct name).
    #[serde(rename = "type")]
    pub type_: String,

    /// The value, base64-encoded, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// A scratch-slot assignment: which slot, and what it holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScratchVariable {
    /// Scratch slot number.
    pub slot: u64,

    /// Encoding of the slot's value (ABI type, AVM type, or struct name).
    #[serde(rename = "type")]
    pub type_: String,
}
