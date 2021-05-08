use std::{fmt, hash::Hasher};

use rand::RngCore;
use serde::{Deserialize, Serialize};
use siphasher::sip128::{Hasher128, SipHasher};
use ulid::Ulid;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SipHashKeys(pub u64, pub u64);

impl SipHashKeys {
    pub fn to_u64(&self) -> (u64, u64) {
        (self.0, self.1)
    }
}

impl From<Ulid> for SipHashKeys {
    fn from(id: Ulid) -> Self {
        let id: (u64, u64) = id.into();
        Self(id.0, id.1)
    }
}

impl From<SipHashKeys> for Ulid {
    fn from(id: SipHashKeys) -> Self {
        Self::from((id.0, id.1))
    }
}

impl From<&SipHashKeys> for Ulid {
    fn from(id: &SipHashKeys) -> Self {
        Self::from((id.0, id.1))
    }
}

impl fmt::Display for SipHashKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(Ulid::from(self).to_string().as_str())
    }
}
#[derive(Debug, Clone)]
pub struct SipHash {
    pub hash: u128,
    pub keys: SipHashKeys,
}

impl SipHash {
    pub fn new(data: &[u8]) -> Self {
        let keys = SipHashKeys(rand::thread_rng().next_u64(), rand::thread_rng().next_u64());
        Self::new_with_keys(keys, data)
    }

    pub fn new_with_keys(keys: SipHashKeys, data: &[u8]) -> Self {
        let mut sip = SipHasher::new_with_keys(keys.0, keys.1);
        sip.write(data);
        Self {
            hash: sip.finish128().as_u128(),
            keys,
        }
    }
}

impl fmt::Display for SipHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(Ulid(self.hash).to_string().as_str())
    }
}
