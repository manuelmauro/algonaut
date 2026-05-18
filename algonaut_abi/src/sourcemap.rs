//! TEAL source-map parser.
//!
//! `POST /v2/teal/compile?sourcemap=true` returns a Source Map v3 JSON
//! object describing how compiled program counters relate back to TEAL
//! source positions. This module exposes that mapping as typed Rust:
//!
//! ```ignore
//! use algonaut_abi::sourcemap::SourceMap;
//!
//! let json = r#"{"version":3,"sources":["main.teal"],"names":[],"mappings":";AAOA;AACA"}"#;
//! let map = SourceMap::from_json(json).unwrap();
//! assert_eq!(map.pc_to_line(0), Some(0));
//! ```
//!
//! V3 source-map specification:
//! <https://sourcemaps.info/spec.html>

use serde::Deserialize;
use std::collections::BTreeMap;

/// A decoded TEAL source map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMap {
    /// Spec version (must be `3`).
    pub version: u32,
    /// File this source map is for, when provided.
    pub file: Option<String>,
    /// Root URL prepended to every entry in `sources`.
    pub source_root: Option<String>,
    /// One entry per source TEAL file.
    pub sources: Vec<String>,
    /// Optional symbolic names referenced by some segments.
    pub names: Vec<String>,
    /// Raw `mappings` VLQ string. Kept for round-tripping.
    pub mappings: String,
    /// Per-PC mapping. The outer index is the generated PC (the index of
    /// the group between `;` separators); `None` means the PC has no
    /// recorded mapping. Decoded once at parse time.
    pub pc_to_segment: Vec<Option<Segment>>,
}

/// One decoded segment of a source-map line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Segment {
    /// 0-based column in the generated (compiled) code.
    pub generated_column: i64,
    /// Index into `sources` of the original file.
    pub source_index: usize,
    /// 0-based original source line.
    pub source_line: usize,
    /// 0-based original source column.
    pub source_column: usize,
    /// Index into `names`, when the segment carries a name.
    pub name_index: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct RawV3 {
    version: u32,
    #[serde(default)]
    file: Option<String>,
    #[serde(rename = "sourceRoot", default)]
    source_root: Option<String>,
    #[serde(default)]
    sources: Vec<String>,
    #[serde(default)]
    names: Vec<String>,
    #[serde(default)]
    mappings: String,
}

impl SourceMap {
    /// Parse from JSON text. Returns an error on invalid JSON, an
    /// unsupported version, or a malformed `mappings` string.
    pub fn from_json(json: &str) -> Result<Self, SourceMapError> {
        let raw: RawV3 = serde_json::from_str(json).map_err(SourceMapError::Json)?;
        if raw.version != 3 {
            return Err(SourceMapError::UnsupportedVersion(raw.version));
        }
        let pc_to_segment = decode_mappings(&raw.mappings)?;
        Ok(SourceMap {
            version: raw.version,
            file: raw.file,
            source_root: raw.source_root,
            sources: raw.sources,
            names: raw.names,
            mappings: raw.mappings,
            pc_to_segment,
        })
    }

    /// Source line associated with `pc`, if any.
    pub fn pc_to_line(&self, pc: usize) -> Option<usize> {
        self.pc_to_segment
            .get(pc)
            .copied()
            .flatten()
            .map(|s| s.source_line)
    }

    /// Every PC that maps to `line`, in ascending order.
    pub fn line_to_pcs(&self, line: usize) -> Vec<usize> {
        self.pc_to_segment
            .iter()
            .enumerate()
            .filter_map(|(pc, seg)| match seg {
                Some(s) if s.source_line == line => Some(pc),
                _ => None,
            })
            .collect()
    }

    /// Last PC that maps to `line`, if any. Convenience used by
    /// algorand-sdk-testing's `getting the last pc associated with a
    /// line ...` scenario.
    pub fn last_pc_for_line(&self, line: usize) -> Option<usize> {
        self.line_to_pcs(line).into_iter().next_back()
    }

    /// Map `(source, line)` to its first PC at the smallest column.
    pub fn source_line_to_pc(&self, source: &str, line: usize) -> Option<(usize, usize)> {
        let mut best: Option<(usize, usize)> = None; // (pc, column)
        for (pc, seg) in self.pc_to_segment.iter().enumerate() {
            if let Some(s) = seg
                && self.sources.get(s.source_index).map(String::as_str) == Some(source)
                && s.source_line == line
                && best.is_none_or(|(_, col)| s.source_column < col)
            {
                best = Some((pc, s.source_column));
            }
        }
        best
    }

    /// All distinct (pc → segment) entries, in PC order. Useful for
    /// dumping the map or building secondary indices.
    pub fn segments(&self) -> impl Iterator<Item = (usize, Segment)> + '_ {
        self.pc_to_segment
            .iter()
            .enumerate()
            .filter_map(|(pc, seg)| seg.map(|s| (pc, s)))
    }

    /// For each PC that has a mapping, emit `"pc:line"` separated by
    /// `;`. Order is ascending by PC.
    pub fn pc_line_string(&self) -> String {
        let entries: BTreeMap<usize, usize> =
            self.segments().map(|(pc, s)| (pc, s.source_line)).collect();
        entries
            .into_iter()
            .map(|(pc, line)| format!("{pc}:{line}"))
            .collect::<Vec<_>>()
            .join(";")
    }
}

/// Errors produced while parsing a source map.
#[derive(Debug, thiserror::Error)]
pub enum SourceMapError {
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported source-map version: {0}")]
    UnsupportedVersion(u32),
    #[error("invalid base64 VLQ character: {0:?}")]
    InvalidBase64(char),
    #[error("truncated VLQ sequence in mappings")]
    Truncated,
}

fn decode_mappings(mappings: &str) -> Result<Vec<Option<Segment>>, SourceMapError> {
    // Spec: across `;` boundaries the generated-column resets to 0 but
    // source_index / source_line / source_column / name_index continue
    // accumulating.
    //
    // For TEAL the generated-line is the PC. The algorand-sdk-testing
    // reference implementations treat empty groups as inheriting the
    // most recent (source_line, source_column, source_index) — TEAL
    // opcodes that span multiple bytes share the source position of
    // their first byte. PC 0 inherits the implicit start state
    // (line 0 / column 0 / source 0), so it always has a segment.
    let mut source_index: i64 = 0;
    let mut source_line: i64 = 0;
    let mut source_column: i64 = 0;
    let mut name_index: i64 = 0;

    let mut last_segment = Segment {
        generated_column: 0,
        source_index: 0,
        source_line: 0,
        source_column: 0,
        name_index: None,
    };
    let mut out: Vec<Option<Segment>> = Vec::new();

    for group in mappings.split(';') {
        let mut generated_column: i64 = 0;
        let mut explicit: Option<Segment> = None;
        for segment in group.split(',') {
            if segment.is_empty() {
                continue;
            }
            let values = decode_vlq_sequence(segment)?;
            match values.len() {
                1 => {
                    generated_column += values[0];
                }
                4 | 5 => {
                    generated_column += values[0];
                    source_index += values[1];
                    source_line += values[2];
                    source_column += values[3];
                    if values.len() == 5 {
                        name_index += values[4];
                    }
                    let seg = Segment {
                        generated_column,
                        source_index: source_index as usize,
                        source_line: source_line as usize,
                        source_column: source_column as usize,
                        name_index: if values.len() == 5 {
                            Some(name_index as usize)
                        } else {
                            None
                        },
                    };
                    if explicit.is_none() {
                        explicit = Some(seg);
                    }
                }
                _ => return Err(SourceMapError::Truncated),
            }
        }
        match explicit {
            Some(seg) => {
                last_segment = seg;
                out.push(Some(seg));
            }
            None => out.push(Some(last_segment)),
        }
    }
    Ok(out)
}

fn decode_vlq_sequence(s: &str) -> Result<Vec<i64>, SourceMapError> {
    let mut out = Vec::new();
    let mut chars = s.chars().peekable();
    while chars.peek().is_some() {
        out.push(decode_vlq(&mut chars)?);
    }
    Ok(out)
}

fn decode_vlq<I: Iterator<Item = char>>(chars: &mut I) -> Result<i64, SourceMapError> {
    let mut value: u64 = 0;
    let mut shift: u32 = 0;
    loop {
        let c = chars.next().ok_or(SourceMapError::Truncated)?;
        let digit = b64_digit(c)?;
        let continuation = digit & 0b10_0000 != 0;
        let chunk = (digit & 0b01_1111) as u64;
        value |= chunk << shift;
        shift += 5;
        if !continuation {
            break;
        }
    }
    // Bit 0 is the sign, bits 1.. are the magnitude.
    let magnitude = (value >> 1) as i64;
    if value & 1 == 1 {
        Ok(-magnitude)
    } else {
        Ok(magnitude)
    }
}

fn b64_digit(c: char) -> Result<u8, SourceMapError> {
    match c {
        'A'..='Z' => Ok(c as u8 - b'A'),
        'a'..='z' => Ok(c as u8 - b'a' + 26),
        '0'..='9' => Ok(c as u8 - b'0' + 52),
        '+' => Ok(62),
        '/' => Ok(63),
        _ => Err(SourceMapError::InvalidBase64(c)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE_V1: &str = r#"{"version":3,"sources":["<body>"],"names":[],"mappings":";AAOA;;;;;;;;;;;;;;;;;;;;;;;;;;;;AACA;AACA;;;AACA;;;AACA;AACA;;AACA;AACA;;;AACA;AACA;AACA;;AACA;;AACA;AACA;AACA"}"#;

    #[test]
    fn parses_fixture_v1() {
        let m = SourceMap::from_json(FIXTURE_V1).unwrap();
        assert_eq!(m.version, 3);
        assert_eq!(m.sources, vec!["<body>"]);
        // The first non-empty PC is 1, mapping to source line 7.
        assert_eq!(m.pc_to_line(0), Some(0));
        assert_eq!(m.pc_to_line(1), Some(7));
        assert_eq!(m.pc_to_line(29), Some(8));
        assert_eq!(m.pc_to_line(51), Some(21));
    }

    #[test]
    fn pc_line_string_matches_fixture() {
        let m = SourceMap::from_json(FIXTURE_V1).unwrap();
        let expected = "0:0;1:7;2:7;3:7;4:7;5:7;6:7;7:7;8:7;9:7;10:7;11:7;12:7;13:7;14:7;15:7;16:7;17:7;18:7;19:7;20:7;21:7;22:7;23:7;24:7;25:7;26:7;27:7;28:7;29:8;30:9;31:9;32:9;33:10;34:10;35:10;36:11;37:12;38:12;39:13;40:14;41:14;42:14;43:15;44:16;45:17;46:17;47:18;48:18;49:19;50:20;51:21";
        assert_eq!(m.pc_line_string(), expected);
    }

    #[test]
    fn last_pc_for_line() {
        let m = SourceMap::from_json(FIXTURE_V1).unwrap();
        assert_eq!(m.last_pc_for_line(0), Some(0));
        assert_eq!(m.last_pc_for_line(7), Some(28));
        assert_eq!(m.last_pc_for_line(9), Some(32));
        assert_eq!(m.last_pc_for_line(21), Some(51));
    }

    const FIXTURE_V2: &str = r#"{"version":3,"sources":["sourcemap-test.teal","lib.teal"],"names":[],"mappings":";;;;AAGA;;AAAmB;;AAAO;AAAI;;;AAG1B;;AAAO;;AAAO;AACd;;AAAO;AAAO;ACNG;AAAI;ADUrB;AACA"}"#;

    #[test]
    fn cross_source_mapping() {
        let m = SourceMap::from_json(FIXTURE_V2).unwrap();
        let s4 = m.pc_to_segment[4].unwrap();
        assert_eq!(s4.source_line, 3);
        assert_eq!(s4.source_column, 0);
        assert_eq!(m.sources[s4.source_index], "sourcemap-test.teal");

        // PC 21 jumps to a different source.
        let s21 = m.pc_to_segment[21].unwrap();
        assert_eq!(m.sources[s21.source_index], "lib.teal");
        assert_eq!(s21.source_line, 1);
        assert_eq!(s21.source_column, 21);
    }
}
