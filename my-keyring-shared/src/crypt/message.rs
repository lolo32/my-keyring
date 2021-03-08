use crate::crypt::{Salt, HMAC_LENGTH, SALT_LENGTH};
use crate::MyKeyringError;
use bincode::{deserialize, serialize};
use core::fmt;
use std::convert::TryFrom;

/// Represent a message encrypted, and the information needed to decrypt it
pub struct CryptedMessage {
    /// Public salt to derive passwords/iv for encryption/hmac
    pub(crate) salt: Salt,
    /// Data encrypted
    pub(crate) data: Vec<u8>,
    /// Hmac of the `data`
    pub(crate) hmac: [u8; HMAC_LENGTH],
}

impl fmt::Debug for CryptedMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CryptedMessage { .. }")
    }
}

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

impl serde::Serialize for CryptedMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Note: do not change the serialization format, or it may break
        // forward and backward compatibility of serialized data!

        (self.salt.to_vec(), self.hmac.to_vec(), self.data.clone()).serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for CryptedMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        type SerdeMessage = (Vec<u8>, Vec<u8>, Vec<u8>);

        let (salt, hmac, data): SerdeMessage = serde::Deserialize::deserialize(deserializer)?;

        Ok(CryptedMessage {
            salt: vec_to_salt(salt),
            data,
            hmac: vec_to_hmac(hmac),
        })
    }
}

#[inline]
fn vec_to_salt(data: Vec<u8>) -> Salt {
    let mut salt = [0; SALT_LENGTH];
    salt.copy_from_slice(&data);
    salt
}

#[inline]
fn vec_to_hmac(data: Vec<u8>) -> [u8; HMAC_LENGTH] {
    let mut mac = [0; HMAC_LENGTH];
    mac.copy_from_slice(&data);
    mac
}
