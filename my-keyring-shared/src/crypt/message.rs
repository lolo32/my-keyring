use core::convert::TryFrom;

use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};

use crate::{crypt::Salt, MyKeyringError};

/// Represent a message encrypted, and the information needed to decrypt it
#[derive(Serialize, Deserialize)]
pub struct CryptedMessage {
    /// Public salt to derive passwords/iv for encryption/hmac
    pub(crate) salt: Salt,
    /// Data encrypted
    pub(crate) data: Vec<u8>,
}

opaque_debug::implement!(CryptedMessage);

impl TryFrom<&[u8]> for CryptedMessage {
    type Error = MyKeyringError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        deserialize(value).map_err(|_| MyKeyringError::InvalidCryptedMessage)
    }
}

impl TryFrom<Vec<u8>> for CryptedMessage {
    type Error = MyKeyringError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

impl Into<Vec<u8>> for CryptedMessage {
    fn into(self) -> Vec<u8> {
        serialize(&self).expect("serialized CryptedMessage struct data")
    }
}
