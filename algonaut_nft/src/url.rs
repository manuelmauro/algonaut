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
//! This module is the offline transform both ways:
//! - [`resolve_url`] / [`cid_from_reserve`]: Reserve → `ipfs://` URL (reading);
//! - [`reserve_from_cid`]: CID → Reserve address (minting / updating).
//!
//! Per ARC-19, clients MUST support CID v0 and v1, the `raw` and `dag-pb`
//! multicodecs, and the `sha2-256` multihash.

use algonaut_core::Address;
use data_encoding::BASE32_NOPAD;

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

    fn from_code(code: u8) -> Option<Codec> {
        match code {
            0x55 => Some(Codec::Raw),
            0x70 => Some(Codec::DagPb),
            _ => None,
        }
    }
}

/// A parsed `template-ipfs://…` asset URL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateIpfsUrl {
    /// CID version to reconstruct.
    pub version: CidVersion,
    /// Multicodec content type.
    pub codec: Codec,
    /// Anything after the `{…}` template (e.g. `/arc3.json`).
    pub suffix: String,
}

/// A reconstructed IPFS content identifier carrying a 32-byte `sha2-256` digest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cid {
    /// CID version.
    pub version: CidVersion,
    /// Multicodec content type.
    pub codec: Codec,
    /// The 32-byte `sha2-256` digest (the Reserve address bytes).
    pub digest: [u8; 32],
}

impl Cid {
    /// Render the CID to its canonical string form (base58btc for v0, multibase
    /// base32 — `b…` — for v1).
    pub fn to_cid_string(&self) -> String {
        let multihash = [&[MULTIHASH_SHA2_256, SHA2_256_LEN][..], &self.digest[..]].concat();
        match self.version {
            CidVersion::V0 => base58btc_encode(&multihash),
            CidVersion::V1 => {
                let bytes = [&[0x01, self.codec.code()][..], &multihash[..]].concat();
                format!("b{}", BASE32_NOPAD.encode(&bytes).to_lowercase())
            }
        }
    }

    /// Parse a CID string, extracting its 32-byte `sha2-256` digest.
    pub fn parse(cid: &str) -> Result<Cid, crate::NftError> {
        let bad = |m: &str| crate::NftError::MalformedCid(m.to_string());
        if let Some(rest) = cid.strip_prefix('b') {
            // CIDv1, multibase base32 (lowercase).
            let bytes = BASE32_NOPAD
                .decode(rest.to_uppercase().as_bytes())
                .map_err(|_| bad("invalid base32"))?;
            if bytes.len() != 36 || bytes[0] != 0x01 {
                return Err(bad("not a v1 sha2-256 CID"));
            }
            let codec = Codec::from_code(bytes[1]).ok_or_else(|| bad("unsupported codec"))?;
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

/// Parse a `template-ipfs://…` asset URL.
pub fn parse_template_url(au: &str) -> Result<TemplateIpfsUrl, crate::NftError> {
    let bad = |m: String| crate::NftError::BadTemplateUrl(m);
    let rest = au
        .strip_prefix(TEMPLATE_PREFIX)
        .ok_or_else(|| bad(format!("missing {TEMPLATE_PREFIX} scheme")))?;
    if !rest.starts_with('{') {
        return Err(bad("expected '{' after scheme".into()));
    }
    let close = rest
        .find('}')
        .ok_or_else(|| bad("missing closing '}'".into()))?;
    let inner = &rest[1..close];
    let suffix = rest[close + 1..].to_string();

    let parts: Vec<&str> = inner.split(':').collect();
    if parts.len() != 5 || parts[0] != "ipfscid" {
        return Err(bad(format!("malformed template body: {inner:?}")));
    }
    let version = match parts[1] {
        "0" => CidVersion::V0,
        "1" => CidVersion::V1,
        v => return Err(crate::NftError::UnsupportedCid(format!("CID version {v}"))),
    };
    let codec = match parts[2] {
        "raw" => Codec::Raw,
        "dag-pb" => Codec::DagPb,
        c => return Err(crate::NftError::UnsupportedCid(format!("multicodec {c}"))),
    };
    if parts[3] != "reserve" {
        return Err(bad(format!("field must be 'reserve', got {:?}", parts[3])));
    }
    if parts[4] != "sha2-256" {
        return Err(crate::NftError::UnsupportedCid(format!(
            "hash {}",
            parts[4]
        )));
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

/// Reconstruct the CID a template points at, given the current Reserve address.
pub fn cid_from_reserve(template: &TemplateIpfsUrl, reserve: Address) -> Cid {
    Cid {
        version: template.version,
        codec: template.codec,
        digest: reserve.0,
    }
}

/// Derive the Reserve address that makes an asset point at the given CID.
///
/// This is the minting / updating direction: set the ASA's Reserve to this
/// address (e.g. via `UpdateAsset::reserve`) to repoint an ARC-19 NFT.
pub fn reserve_from_cid(cid: &Cid) -> Address {
    Address(cid.digest)
}

/// Resolve a `template-ipfs://…` URL plus a Reserve address into an `ipfs://` URL.
pub fn resolve_url(au: &str, reserve: Address) -> Result<String, crate::NftError> {
    let template = parse_template_url(au)?;
    let cid = cid_from_reserve(&template, reserve);
    Ok(format!("ipfs://{}{}", cid.to_cid_string(), template.suffix))
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
    fn parses_template_variants() {
        let v1 = parse_template_url("template-ipfs://{ipfscid:1:raw:reserve:sha2-256}").unwrap();
        assert_eq!(v1.version, CidVersion::V1);
        assert_eq!(v1.codec, Codec::Raw);
        assert_eq!(v1.suffix, "");

        let v0 =
            parse_template_url("template-ipfs://{ipfscid:0:dag-pb:reserve:sha2-256}/arc3.json")
                .unwrap();
        assert_eq!(v0.version, CidVersion::V0);
        assert_eq!(v0.codec, Codec::DagPb);
        assert_eq!(v0.suffix, "/arc3.json");
    }

    #[test]
    fn rejects_unsupported_and_malformed() {
        assert!(matches!(
            parse_template_url("ipfs://Qmfoo"),
            Err(crate::NftError::BadTemplateUrl(_))
        ));
        assert!(matches!(
            parse_template_url("template-ipfs://{ipfscid:2:raw:reserve:sha2-256}"),
            Err(crate::NftError::UnsupportedCid(_))
        ));
        assert!(matches!(
            parse_template_url("template-ipfs://{ipfscid:1:raw:reserve:blake3}"),
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

        // The CID string between ipfs:// and the suffix decodes back to reserve.
        let cid_str = url
            .strip_prefix("ipfs://")
            .unwrap()
            .strip_suffix("/meta.json")
            .unwrap();
        let parsed = Cid::parse(cid_str).unwrap();
        assert_eq!(reserve_from_cid(&parsed), reserve);
        assert_eq!(parsed.codec, Codec::Raw);
    }

    #[test]
    fn reserve_round_trips_through_cid_v0() {
        let reserve = Address([0u8; 32]);
        let template =
            parse_template_url("template-ipfs://{ipfscid:0:dag-pb:reserve:sha2-256}").unwrap();
        let cid = cid_from_reserve(&template, reserve);
        let s = cid.to_cid_string();
        assert!(s.starts_with("Qm"));
        assert_eq!(reserve_from_cid(&Cid::parse(&s).unwrap()), reserve);
    }
}
