//! ARC-19 — templating of NFT ASA URLs for mutability.
//!
//! ARC-19 makes a 1-of-1 NFT's metadata pointer *mutable* by repurposing the
//! ASA's otherwise-unused 32-byte Reserve address as a "bitbucket" holding an
//! IPFS content hash. The asset URL stores a template such as
//! `template-ipfs://{ipfscid:1:raw:reserve:sha2-256}/arc3.json`; a client
//! reconstructs the CID from the template parameters plus the current Reserve
//! address and renders `ipfs://<cid>/arc3.json`. "Updating" the NFT is a single
//! asset-config that changes only the Reserve address.
//!
//! The transform is offline and bidirectional:
//! - reading: [`TemplateIpfsUrl::resolve`] / [`resolve_url`] turn a template +
//!   Reserve into an `ipfs://` URL;
//! - minting / updating: [`Address::from`]`(cid)` derives the Reserve that makes
//!   an asset point at a [`Cid`].
//!
//! Per ARC-19, clients MUST support CID v0 and v1, the `raw` and `dag-pb`
//! multicodecs, and the `sha2-256` multihash.

use algonaut_core::Address;
use data_encoding::BASE32_NOPAD;
use std::fmt;
use std::str::FromStr;

const TEMPLATE_PREFIX: &str = "template-ipfs://";
const MULTIHASH_SHA2_256: u8 = 0x12;
const SHA2_256_LEN: u8 = 0x20;

/// The CID version named by a template.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CidVersion {
    /// CIDv0 — base58btc, implicitly `dag-pb` + `sha2-256`.
    V0,
    /// CIDv1 — multibase base32, explicit multicodec.
    V1,
}

impl CidVersion {
    fn as_str(self) -> &'static str {
        match self {
            CidVersion::V0 => "0",
            CidVersion::V1 => "1",
        }
    }
}

/// The IPFS multicodec content type named by a template.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Codec {
    /// `raw` (0x55) — a single file added with `--cid-version=1`.
    Raw,
    /// `dag-pb` (0x70) — directories / UnixFS.
    DagPb,
}

impl Codec {
    fn code(self) -> u8 {
        match self {
            Codec::Raw => 0x55,
            Codec::DagPb => 0x70,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Codec::Raw => "raw",
            Codec::DagPb => "dag-pb",
        }
    }

    fn from_code(code: u8) -> Option<Codec> {
        match code {
            0x55 => Some(Codec::Raw),
            0x70 => Some(Codec::DagPb),
            _ => None,
        }
    }
}

/// A parsed `template-ipfs://…` asset URL.
///
/// Parse with [`str::parse`] / [`FromStr`], render with [`fmt::Display`], and go
/// to/from a concrete [`Cid`] with [`TemplateIpfsUrl::from_cid`] /
/// [`TemplateIpfsUrl::cid`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateIpfsUrl {
    /// CID version to reconstruct.
    pub version: CidVersion,
    /// Multicodec content type.
    pub codec: Codec,
    /// Anything after the `{…}` template (e.g. `/arc3.json`).
    pub suffix: String,
}

impl TemplateIpfsUrl {
    /// The template that resolves a `cid` (with an optional path `suffix`).
    pub fn from_cid(cid: &Cid, suffix: impl Into<String>) -> Self {
        TemplateIpfsUrl {
            version: cid.version,
            codec: cid.codec,
            suffix: suffix.into(),
        }
    }

    /// Reconstruct the concrete [`Cid`] this template points at, given the
    /// current Reserve address.
    pub fn cid(&self, reserve: Address) -> Cid {
        match self.version {
            CidVersion::V0 => Cid::v0(reserve.0),
            CidVersion::V1 => Cid::v1(self.codec, reserve.0),
        }
    }

    /// Resolve to a concrete `ipfs://<cid><suffix>` URL for a Reserve address.
    pub fn resolve(&self, reserve: Address) -> String {
        format!("ipfs://{}{}", self.cid(reserve), self.suffix)
    }
}

impl fmt::Display for TemplateIpfsUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{TEMPLATE_PREFIX}{{ipfscid:{}:{}:reserve:sha2-256}}{}",
            self.version.as_str(),
            self.codec.as_str(),
            self.suffix
        )
    }
}

impl FromStr for TemplateIpfsUrl {
    type Err = crate::NftError;

    fn from_str(au: &str) -> Result<Self, Self::Err> {
        let bad = crate::NftError::BadTemplateUrl;
        let rest = au
            .strip_prefix(TEMPLATE_PREFIX)
            .ok_or_else(|| bad(format!("missing {TEMPLATE_PREFIX} scheme")))?;
        let inner = rest
            .strip_prefix('{')
            .ok_or_else(|| bad("expected '{' after scheme".into()))?;
        let close = inner
            .find('}')
            .ok_or_else(|| bad("missing closing '}'".into()))?;
        let body = &inner[..close];
        let suffix = inner[close + 1..].to_string();

        let [tag, version, codec, field, hash] = body.split(':').collect::<Vec<_>>()[..] else {
            return Err(bad(format!("malformed template body: {body:?}")));
        };
        if tag != "ipfscid" {
            return Err(bad(format!("expected 'ipfscid', got {tag:?}")));
        }
        let version = match version {
            "0" => CidVersion::V0,
            "1" => CidVersion::V1,
            v => return Err(crate::NftError::UnsupportedCid(format!("CID version {v}"))),
        };
        let codec = match codec {
            "raw" => Codec::Raw,
            "dag-pb" => Codec::DagPb,
            c => return Err(crate::NftError::UnsupportedCid(format!("multicodec {c}"))),
        };
        if field != "reserve" {
            return Err(bad(format!("field must be 'reserve', got {field:?}")));
        }
        if hash != "sha2-256" {
            return Err(crate::NftError::UnsupportedCid(format!("hash {hash}")));
        }
        if version == CidVersion::V0 && codec != Codec::DagPb {
            return Err(crate::NftError::UnsupportedCid(
                "CIDv0 must be dag-pb".into(),
            ));
        }
        Ok(TemplateIpfsUrl {
            version,
            codec,
            suffix,
        })
    }
}

/// A reconstructed IPFS content identifier carrying a 32-byte `sha2-256` digest.
///
/// Construct with [`Cid::v0`] / [`Cid::v1`] (which keep the version/codec
/// invariant — a CIDv0 is always `dag-pb` — so an inconsistent CID is
/// unrepresentable). Render with [`fmt::Display`] (base58btc for v0, multibase
/// base32 — `b…` — for v1), parse with [`str::parse`] / [`FromStr`], and obtain
/// the Reserve address it maps to with `Address::from(cid)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cid {
    version: CidVersion,
    codec: Codec,
    digest: [u8; 32],
}

impl Cid {
    /// A CIDv1 with the given multicodec and 32-byte `sha2-256` digest.
    pub fn v1(codec: Codec, digest: [u8; 32]) -> Cid {
        Cid {
            version: CidVersion::V1,
            codec,
            digest,
        }
    }

    /// A CIDv0 (implicitly `dag-pb` + `sha2-256`) with the given 32-byte digest.
    pub fn v0(digest: [u8; 32]) -> Cid {
        Cid {
            version: CidVersion::V0,
            codec: Codec::DagPb,
            digest,
        }
    }

    /// The CID version.
    pub fn version(&self) -> CidVersion {
        self.version
    }

    /// The multicodec content type.
    pub fn codec(&self) -> Codec {
        self.codec
    }

    /// The 32-byte `sha2-256` digest (the Reserve address bytes).
    pub fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

impl fmt::Display for Cid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let multihash = [&[MULTIHASH_SHA2_256, SHA2_256_LEN][..], &self.digest[..]].concat();
        match self.version {
            CidVersion::V0 => f.write_str(&base58btc_encode(&multihash)),
            CidVersion::V1 => {
                let bytes = [&[0x01, self.codec.code()][..], &multihash[..]].concat();
                write!(f, "b{}", BASE32_NOPAD.encode(&bytes).to_lowercase())
            }
        }
    }
}

impl FromStr for Cid {
    type Err = crate::NftError;

    fn from_str(cid: &str) -> Result<Self, Self::Err> {
        let bad = |m: &str| crate::NftError::MalformedCid(m.to_string());
        if let Some(rest) = cid.strip_prefix('b') {
            // CIDv1, multibase base32 (lowercase).
            let bytes = BASE32_NOPAD
                .decode(rest.to_uppercase().as_bytes())
                .map_err(|_| bad("invalid base32"))?;
            if bytes.len() != 36 || bytes[0] != 0x01 {
                return Err(bad("not a v1 sha2-256 CID"));
            }
            // Well-formed but unsupported codec is a distinct, structured error.
            let codec = Codec::from_code(bytes[1]).ok_or_else(|| {
                crate::NftError::UnsupportedCid(format!("multicodec 0x{:02x}", bytes[1]))
            })?;
            check_multihash(&bytes[2..], &bad)?;
            Ok(Cid {
                version: CidVersion::V1,
                codec,
                digest: digest_from_multihash(&bytes[2..]),
            })
        } else if cid.starts_with("Qm") {
            // CIDv0, base58btc multihash.
            let mh = base58btc_decode(cid).map_err(|_| bad("invalid base58btc"))?;
            check_multihash(&mh, &bad)?;
            Ok(Cid {
                version: CidVersion::V0,
                codec: Codec::DagPb,
                digest: digest_from_multihash(&mh),
            })
        } else {
            Err(bad("unrecognised CID encoding"))
        }
    }
}

/// The Reserve address that makes an asset point at this CID — the minting /
/// updating direction (set it via `UpdateAsset::reserve`).
impl From<Cid> for Address {
    fn from(cid: Cid) -> Address {
        Address(cid.digest)
    }
}

fn check_multihash(
    mh: &[u8],
    bad: &impl Fn(&str) -> crate::NftError,
) -> Result<(), crate::NftError> {
    if mh.len() != 34 || mh[0] != MULTIHASH_SHA2_256 || mh[1] != SHA2_256_LEN {
        Err(bad("multihash is not 32-byte sha2-256"))
    } else {
        Ok(())
    }
}

fn digest_from_multihash(mh: &[u8]) -> [u8; 32] {
    let mut d = [0u8; 32];
    d.copy_from_slice(&mh[2..34]);
    d
}

/// Resolve a `template-ipfs://…` URL plus a Reserve address into an `ipfs://` URL.
pub fn resolve_url(au: &str, reserve: Address) -> Result<String, crate::NftError> {
    Ok(au.parse::<TemplateIpfsUrl>()?.resolve(reserve))
}

// --- minimal base58btc (Bitcoin alphabet) ---

const B58: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

fn base58btc_encode(input: &[u8]) -> String {
    let zeros = input.iter().take_while(|&&b| b == 0).count();
    let mut digits: Vec<u8> = Vec::new();
    for &byte in input {
        let mut carry = byte as u32;
        for d in digits.iter_mut() {
            carry += (*d as u32) << 8;
            *d = (carry % 58) as u8;
            carry /= 58;
        }
        while carry > 0 {
            digits.push((carry % 58) as u8);
            carry /= 58;
        }
    }
    let mut out = String::with_capacity(zeros + digits.len());
    for _ in 0..zeros {
        out.push('1');
    }
    for &d in digits.iter().rev() {
        out.push(B58[d as usize] as char);
    }
    out
}

fn base58btc_decode(s: &str) -> Result<Vec<u8>, ()> {
    let mut bytes: Vec<u8> = Vec::new();
    for c in s.chars() {
        let val = B58.iter().position(|&a| a as char == c).ok_or(())? as u32;
        let mut carry = val;
        for b in bytes.iter_mut() {
            carry += (*b as u32) * 58;
            *b = (carry & 0xff) as u8;
            carry >>= 8;
        }
        while carry > 0 {
            bytes.push((carry & 0xff) as u8);
            carry >>= 8;
        }
    }
    let leading_zeros = s.chars().take_while(|&c| c == '1').count();
    bytes.resize(bytes.len() + leading_zeros, 0);
    bytes.reverse();
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base58_handles_leading_zeros() {
        assert_eq!(base58btc_encode(&[0, 0, 1]), "112");
        assert_eq!(base58btc_decode("112").unwrap(), vec![0, 0, 1]);
    }

    #[test]
    fn parses_and_renders_template_variants() {
        let v1: TemplateIpfsUrl = "template-ipfs://{ipfscid:1:raw:reserve:sha2-256}"
            .parse()
            .unwrap();
        assert_eq!(v1.version, CidVersion::V1);
        assert_eq!(v1.codec, Codec::Raw);
        assert_eq!(v1.suffix, "");
        // Display round-trips.
        assert_eq!(
            v1.to_string(),
            "template-ipfs://{ipfscid:1:raw:reserve:sha2-256}"
        );

        let v0: TemplateIpfsUrl = "template-ipfs://{ipfscid:0:dag-pb:reserve:sha2-256}/arc3.json"
            .parse()
            .unwrap();
        assert_eq!(v0.version, CidVersion::V0);
        assert_eq!(v0.suffix, "/arc3.json");
        assert_eq!(
            v0.to_string(),
            "template-ipfs://{ipfscid:0:dag-pb:reserve:sha2-256}/arc3.json"
        );
    }

    #[test]
    fn rejects_unsupported_and_malformed() {
        assert!(matches!(
            "ipfs://Qmfoo".parse::<TemplateIpfsUrl>(),
            Err(crate::NftError::BadTemplateUrl(_))
        ));
        assert!(matches!(
            "template-ipfs://{ipfscid:2:raw:reserve:sha2-256}".parse::<TemplateIpfsUrl>(),
            Err(crate::NftError::UnsupportedCid(_))
        ));
        assert!(matches!(
            "template-ipfs://{ipfscid:1:raw:reserve:blake3}".parse::<TemplateIpfsUrl>(),
            Err(crate::NftError::UnsupportedCid(_))
        ));
    }

    #[test]
    fn reserve_round_trips_through_cid_v1() {
        let reserve = Address([7u8; 32]);
        let url = resolve_url(
            "template-ipfs://{ipfscid:1:raw:reserve:sha2-256}/meta.json",
            reserve,
        )
        .unwrap();
        assert!(url.starts_with("ipfs://b"));
        assert!(url.ends_with("/meta.json"));

        let cid_str = url
            .strip_prefix("ipfs://")
            .unwrap()
            .strip_suffix("/meta.json")
            .unwrap();
        let parsed: Cid = cid_str.parse().unwrap();
        assert_eq!(Address::from(parsed), reserve);
        assert_eq!(parsed.codec, Codec::Raw);
    }

    #[test]
    fn reserve_round_trips_through_cid_v0() {
        let reserve = Address([0u8; 32]);
        let template: TemplateIpfsUrl = "template-ipfs://{ipfscid:0:dag-pb:reserve:sha2-256}"
            .parse()
            .unwrap();
        let cid = template.cid(reserve);
        let s = cid.to_string();
        assert!(s.starts_with("Qm"));
        assert_eq!(Address::from(s.parse::<Cid>().unwrap()), reserve);
    }
}
