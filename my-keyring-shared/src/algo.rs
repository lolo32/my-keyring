use hmac::{Hmac, Mac, NewMac};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Sha256, Sha512};

use crate::errors::MyKeyringError;

/// Algorithms that can be used to compute a hmac
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Algorithm {
    /// Sha-1 variant
    Sha1,
    /// Sha2-256 variant
    Sha256,
    /// Sha2-512 variant
    Sha512,
}

impl Algorithm {
    /// Compute the hmac of `data` using a `key` secret
    ///
    /// # Examples
    ///
    /// ```
    /// use my_keyring_shared::Algorithm;
    ///
    /// let digest = Algorithm::Sha256.hmac(
    ///     b"My_secr3tP@55w0rd",
    ///     b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec et ultricies augue."
    /// );
    /// assert_eq!(&digest, &[133, 77, 197, 178, 150, 26, 234, 99, 31, 141, 206, 214, 240, 207,
    ///     68, 255, 59, 25, 141, 27, 124, 19, 60, 3, 134, 225, 149, 137, 9, 104, 109, 180]
    /// );
    /// ```
    #[inline]
    pub fn hmac(self, key: &[u8], data: &[u8]) -> Vec<u8> {
        macro_rules! hmac_hash {
            ($hash:ty) => {{
                // Create the HMAC
                let mut mac = <Hmac<$hash>>::new_varkey(key).expect("Hmac engine");
                // Do the hashing
                mac.update(data);
                // Return the result
                mac.finalize().into_bytes().to_vec()
            }};
        }
        match self {
            Self::Sha1 => hmac_hash!(Sha1),
            Self::Sha256 => hmac_hash!(Sha256),
            Self::Sha512 => hmac_hash!(Sha512),
        }
    }

    /// Check the hmac `signature` of `data` using a `key` secret
    ///
    /// # Examples
    ///
    /// Correct signature
    ///
    /// ```
    /// use my_keyring_shared::Algorithm;
    ///
    /// let verification = Algorithm::Sha256.hmac_verify(
    ///     b"My_secr3tP@55w0rd",
    ///     b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec et ultricies augue.",
    ///     &[133, 77, 197, 178, 150, 26, 234, 99, 31, 141, 206, 214, 240, 207, 68, 255,
    ///         59, 25, 141, 27, 124, 19, 60, 3, 134, 225, 149, 137, 9, 104, 109, 180]
    /// );
    /// assert!(verification.is_ok());
    /// ```
    ///
    /// Inorrect signature
    ///
    /// ```
    /// use my_keyring_shared::{Algorithm, MyKeyringError};
    ///
    /// let verification = Algorithm::Sha256.hmac_verify(
    ///     b"My_secr3tP@55w0rd",
    ///     b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec et ultricies augue.",
    ///     b"12345678901234567890123456789012"
    /// );
    /// assert!(verification.is_err());
    /// assert_eq!(verification.unwrap_err(), MyKeyringError::IncorrectHmac);
    /// ```
    ///
    /// # Errors
    ///
    /// The value `MyKeyringError::IncorrectHmac` is return if the hmac
    /// verification is invalid
    #[inline]
    pub fn hmac_verify(self, key: &[u8], data: &[u8], signature: &[u8]) -> crate::Result<()> {
        macro_rules! hmac_verify {
            ($hash:ty) => {{
                // Create the HMAC
                let mut mac = <Hmac<$hash>>::new_varkey(key).expect("Hmac engine");
                // Do the hashing
                mac.update(data);
                // Check the result
                mac.verify(&signature)
                    .map_err(|_| MyKeyringError::IncorrectHmac)
            }};
        }
        match self {
            Self::Sha1 => hmac_verify!(Sha1),
            Self::Sha256 => hmac_verify!(Sha256),
            Self::Sha512 => hmac_verify!(Sha512),
        }
    }
}
