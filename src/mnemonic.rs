use crate::Error;
use sha2::Digest;
use static_assertions::const_assert_eq;

mod wordlist;

const BITS_PER_WORD: usize = 11;
const CHECKSUM_LEN_WORDS: usize = 1;
const KEY_LEN_BYTES: usize = 32;
const MNEM_LEN_WORDS: usize = 25; // includes checksum word
const PADDING_ZEROS: usize = BITS_PER_WORD - ((KEY_LEN_BYTES * 8) % BITS_PER_WORD);
const MNEMONIC_DELIM: &str = " ";
type ChecksumAlg = sha2::Sha512Trunc256;

const_assert_eq!(mnemonic_constants; MNEM_LEN_WORDS * BITS_PER_WORD - (CHECKSUM_LEN_WORDS*BITS_PER_WORD), KEY_LEN_BYTES * 8 + PADDING_ZEROS);

/// Converts a 32-byte key into a 25 word mnemonic. The generated
/// mnemonic includes a checksum. Each word in the mnemonic represents 11 bits
/// of data, and the last 11 bits are reserved for the checksum.
pub fn from_key(key: &[u8]) -> Result<String, Error> {
    if key.len() != KEY_LEN_BYTES {
        return Err(Error::Api(format!(
            "key length must be {} bytes",
            KEY_LEN_BYTES
        )));
    }
    let check_word = checksum(key);
    let mut words: Vec<_> = to_u11_array(key).into_iter().map(get_word).collect();
    words.push(check_word);
    Ok(words.join(MNEMONIC_DELIM))
}

/// Converts a mnemonic generated using the library into the source
/// key used to create it. It returns an error if the passed mnemonic has
/// an incorrect checksum, if the number of words is unexpected, or if one
/// of the passed words is not found in the words list.
pub fn to_key(string: &str) -> Result<[u8; KEY_LEN_BYTES], Error> {
    let mut mnemonic: Vec<&str> = string.split(MNEMONIC_DELIM).collect();
    if mnemonic.len() != MNEM_LEN_WORDS {
        return Err(Error::Api(format!(
            "mnemonic {:?} needed {} words, had {}",
            mnemonic,
            MNEM_LEN_WORDS,
            mnemonic.len()
        )));
    }
    let check_word = mnemonic.pop().unwrap();
    let mut nums = Vec::with_capacity(mnemonic.len());
    for word in mnemonic {
        let n = wordlist::WORDLIST.get_full(word).ok_or_else(|| {
            Error::Api("mnemonic contains word that is not in word list".to_string())
        })?;
        nums.push(n.0 as u32);
    }
    let mut bytes = to_byte_array(&nums);
    if bytes.len() != KEY_LEN_BYTES + 1 {
        return Err(Error::Api(format!(
            "wrong key length {}, should be {}",
            bytes.len(),
            KEY_LEN_BYTES + 1
        )));
    }
    let _ = bytes.pop();
    if check_word != checksum(&bytes) {
        return Err(Error::Api("checksum failed to validate".to_string()));
    }
    let mut key = [0; KEY_LEN_BYTES];
    key.copy_from_slice(&bytes);
    Ok(key)
}

// Returns a word corresponding to the 11 bit checksum of the data
fn checksum(data: &[u8]) -> &str {
    let d = ChecksumAlg::digest(data);
    get_word(to_u11_array(&d[0..2])[0])
}

// Assumes little-endian
fn to_u11_array(bytes: &[u8]) -> Vec<u32> {
    let mut buf = 0u32;
    let mut bit_count = 0;
    let mut out = Vec::with_capacity((bytes.len() * 8 + BITS_PER_WORD - 1) / BITS_PER_WORD);
    for &b in bytes {
        buf |= (u32::from(b)) << bit_count;
        bit_count += 8;
        if bit_count >= BITS_PER_WORD as u32 {
            out.push(buf & 0x7ff);
            buf >>= BITS_PER_WORD as u32;
            bit_count -= BITS_PER_WORD as u32;
        }
    }
    if bit_count != 0 {
        out.push(buf & 0x7ff);
    }
    out
}

// takes an array of 11 byte numbers and converts them to 8 bit numbers
fn to_byte_array(nums: &[u32]) -> Vec<u8> {
    let mut buf = 0;
    let mut bit_count = 0;
    let mut out = Vec::with_capacity((nums.len() * BITS_PER_WORD + 8 - 1) / 8);
    for &n in nums {
        buf |= n << bit_count;
        bit_count += BITS_PER_WORD as u32;
        while bit_count >= 8 {
            out.push((buf & 0xff) as u8);
            buf >>= 8;
            bit_count -= 8;
        }
    }
    if bit_count != 0 {
        out.push((buf & 0xff) as u8)
    }
    out
}

// Gets the word corresponding to the 11 bit number from the word list
fn get_word(i: u32) -> &'static str {
    wordlist::WORDLIST
        .get_index(i as usize)
        .expect("Word out of range")
}
