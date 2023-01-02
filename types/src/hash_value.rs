use std::fmt::{Debug, Display, Formatter, LowerHex};
use chrono::Local;
use crypto::digest::Digest;
use crypto::sha3::Sha3;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};

static ESMT_SALT: Lazy<HashValue> = Lazy::new(|| {
    let hasher = ESMTHasher::new();
    // add salt
    hasher.update("esmt".as_bytes()).finish()
});

/// Hash Value in ESMT
#[derive(Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Serialize, Deserialize)]
pub struct HashValue {
    hash: [u8; HashValue::LENGTH],
}

impl HashValue {
    /// the length of the hash in bytes
    pub const LENGTH: usize = 32;
    /// the length of the hash in bits
    pub const LENGTH_IN_BITS: usize = Self::LENGTH * 8;

    /// crate a [`HashVale`] from bytes array
    pub fn new(hash: [u8; HashValue::LENGTH]) -> Self {
        Self {
            hash
        }
    }

    /// crate from a slice
    pub fn from_slice(src: &[u8]) -> Option<Self> {
        if src.len() != HashValue::LENGTH {
            println!("{} ERROR [HashValue] HashValue decoding error due to length mismatch. expected: {}, input: {}",
                Local::now(),
                Self::LENGTH,
                src.len()
            );
            return None;
        }
        let mut value = HashValue::zero();
        value.hash.copy_from_slice(src);
        Some(value)
    }

    pub fn zero() -> Self {
        Self {
            hash: [0; 32],
        }
    }

    fn as_ref_mut(&mut self) -> &mut [u8] {
        &mut self.hash[..]
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.hash)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.hash.to_vec()
    }
}

impl LowerHex for HashValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for byte in &self.hash {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

/// print first 4 bytes of the hash value
impl Display for HashValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for byte in self.hash.iter().take(4) {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl Debug for HashValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "HashValue(")?;
        <Self as LowerHex>::fmt(self, f)?;
        write!(f, ")")?;
        Ok(())
    }
}

impl Default for HashValue {
    fn default() -> Self {
        HashValue::zero()
    }
}

impl AsRef<[u8; HashValue::LENGTH]> for HashValue {
    fn as_ref(&self) -> &[u8; HashValue::LENGTH] {
        &self.hash
    }
}


/// authentic_rtree Node HAasher
pub struct ESMTHasher {
    hasher: Sha3,
}

impl ESMTHasher {
    pub fn new() -> Self {
        Self {
            hasher: Sha3::sha3_256(),
        }
    }

    pub fn update(mut self, bytes: &[u8]) -> Self {
        self.hasher.input(bytes);
        self
    }

    pub fn finish(mut self) -> HashValue {
        let mut hash = HashValue::default();
        self.hasher.result(hash.as_ref_mut());
        hash
    }
}

impl Default for ESMTHasher {
    fn default() -> Self {
        ESMTHasher::new().update((*ESMT_SALT).as_ref())
    }
}

#[cfg(test)]
#[test]
fn test_hashvalue() {
    let mut bytes = Vec::with_capacity(64*256);
    for _ in 0..16 {
        bytes.extend([0u8; 1024]);
    }
    let hash = ESMTHasher::default().update(&bytes).finish();
    println!("\"hello world\" hashed to {:?}", hash);
    println!("\"hello world\" hashed to {}", hash);
    //assert_eq!(hash, *(ESMT_SALT));
}