//! Contains functions to encrypt/decrypt a message, based on any shared secret
//!
//! # Examples
//!
//! ```
//! # fn main() -> my_keyring_shared::Result<()> {
//! # use x448::{Secret, PublicKey};
//! # use rand_core::OsRng;
//! # let secret_1 = Secret::new(&mut OsRng);
//! # let secret_2 = Secret::new(&mut OsRng);
//! # let public_key_2 = PublicKey::from(&secret_2);
//! # let shared_secret = secret_1.as_diffie_hellman(&public_key_2).unwrap();
//! use my_keyring_shared::crypt::{crypt, decrypt, CryptedMessage};
//! use std::convert::TryFrom;
//!
//! let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec et ultricies augue.";
//!
//! // Encrypt data
//! let encrypted_data = crypt(shared_secret, text.as_bytes(), None)?;
//!
//! // Serialize the data to send over a network link or store it
//! let array_data: Vec<u8> = encrypted_data.into();
//!
//! // Deserialize the data after download/storage read
//! let encrypted_data = CryptedMessage::try_from(array_data)?;
//!
//! // Decrypt the data, can return an error if shared_secret and/or iterations is invalid
//! # let shared_secret = secret_1.as_diffie_hellman(&public_key_2).unwrap();
//! let data = decrypt(shared_secret, encrypted_data, None)?;
//! let data_text = String::from_utf8(data).expect("valid utf8 string");
//!
//! assert_eq!(text, data_text);
//! # Ok(())
//! # }
//! ```
use std::convert::{Into, TryInto};

use chacha20poly1305::{
    aead::{Aead, NewAead},
    Key as KeyPoly, XChaCha20Poly1305, XNonce,
};
use rand_core::{OsRng, RngCore};
use sha2::Sha512;
use x448::SharedSecret;

pub use self::message::CryptedMessage;
use crate::MyKeyringError;

mod message;

/// Salt array length
pub const SALT_LENGTH: usize = 16;
/// Derivation key array length
pub const KEY_LENGTH: usize = 32;
/// IV array length
pub const NONCE_LENGTH: usize = 24;
/// Length of the derived PBKDF2 used
const DERIVED_LENGTH: usize = KEY_LENGTH + NONCE_LENGTH;

/// Represent a salt for PBKDF2 derivation
pub type Salt = [u8; SALT_LENGTH];
/// Key for encryption and AEAD signature
type Key = [u8; KEY_LENGTH];
/// Nonce of the AEAD signature and encryption
type Nonce = [u8; NONCE_LENGTH];

/// Encrypt a message, needing a `shared_secret`, the `data` and an optional
/// context and application specific `context` for deriving the keys.
///
/// # Examples
///
/// ```
/// # let shared_secret = {
/// #     use x448::{Secret, PublicKey};
/// #     use rand_core::OsRng;
/// #     let secret_1 = Secret::new(&mut OsRng);
/// #     let secret_2 = Secret::new(&mut OsRng);
/// #     let public_key_2 = PublicKey::from(&secret_2);
/// #     secret_1.as_diffie_hellman(&public_key_2).unwrap()
/// # };
/// use my_keyring_shared::crypt::crypt;
///
/// let encrypted_data = crypt(
///     shared_secret,
///     b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec et ultricies augue.",
///     None,
/// );
/// ```
pub fn crypt(
    shared_secret: SharedSecret,
    data: &[u8],
    context: Option<&[u8]>,
) -> crate::Result<CryptedMessage> {
    // Derive the shared_secret
    let (salt, key, nonce) =
        derive_keys(shared_secret.as_bytes(), None, context.unwrap_or_default())?;

    // store data
    let mut encrypted_message = CryptedMessage {
        salt,
        data: data[..].to_vec(),
    };

    // Encrypt
    encrypted_message.data = XChaCha20Poly1305::new(&key)
        .encrypt(&nonce, data)
        .map_err(|_| MyKeyringError::DataLengthExceeded)?;

    Ok(encrypted_message)
}

/// Decrypt an encrypted message, based on `based_secret`, the `encrypted`
/// message data and an optional context and application specific `context`.
///
/// # Examples
///
/// ```
/// # use x448::{Secret, PublicKey};
/// # use rand_core::OsRng;
/// # let secret_1 = Secret::new(&mut OsRng);
/// # let secret_2 = Secret::new(&mut OsRng);
/// # let public_key_2 = PublicKey::from(&secret_2);
/// # let shared_secret = secret_1.as_diffie_hellman(&public_key_2).unwrap();
/// # let receive_encrypted_message = || {
/// #     use my_keyring_shared::crypt::crypt;
/// #     crypt(
/// #         secret_1.as_diffie_hellman(&public_key_2).unwrap(),
/// #         b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec et ultricies augue.",
/// #         None,
/// #     ).unwrap()
/// # };
/// use my_keyring_shared::crypt::decrypt;
///
/// let decrypted_data = decrypt(shared_secret, receive_encrypted_message(), None);
///
/// assert!(decrypted_data.is_ok());
/// assert_eq!(
///     decrypted_data.unwrap(),
///     b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec et ultricies augue.",
/// );
/// ```
///
/// # Errors
///
/// The value `MyKeyringError::IncorrectHmac` can be returned if the
/// `shared_secret` is not valid, so the HMAC signature cannot be checked, or if
/// the message has been altered
pub fn decrypt(
    shared_secret: SharedSecret,
    encrypted: CryptedMessage,
    context: Option<&[u8]>,
) -> crate::Result<Vec<u8>> {
    // Derive the shared_secret
    let (_salt, key, nonce) = derive_keys(
        shared_secret.as_bytes(),
        Some(encrypted.salt),
        context.unwrap_or_default(),
    )?;

    // Verify the Tag message, and so check the Key and Nonce, and decrypt
    let data = XChaCha20Poly1305::new(&key)
        .decrypt(&nonce, encrypted.data.as_ref())
        .map_err(|_| MyKeyringError::IncorrectHmac)?;

    Ok(data)
}

/// Split the derived keys into usable array data
fn split_keys(keys: &[u8]) -> crate::Result<(KeyPoly, XNonce)> {
    // Extract the Nonce
    let nonce: Nonce = (&keys[..NONCE_LENGTH])
        .try_into()
        .map_err(|_| MyKeyringError::InvalidKeyLength)?;
    // extract the first key (encryption)
    let key: Key = (&keys[NONCE_LENGTH..(NONCE_LENGTH + KEY_LENGTH)])
        .try_into()
        .map_err(|_| MyKeyringError::InvalidKeyLength)?;

    Ok((key.into(), nonce.into()))
}

/// Derive a password
///
/// The `shared` is the original password to be derived, the `nonce` is
/// calculated randomly if not specified and `context` represents hardcoded
/// data, globally unique, and application-specific.
///
/// It returns the `salt`, either randomly generated if not specified or the
/// `nonce` value if passed, a derived `key` and a `nonce` based on the input
/// parameters.
fn derive_keys(
    shared: &[u8],
    nonce: Option<Salt>,
    context: &[u8],
) -> crate::Result<(Salt, KeyPoly, XNonce)> {
    let mut hex = [0; DERIVED_LENGTH];
    // Generate a salt if none
    let salt = match nonce {
        Some(nonce) => nonce,
        None => {
            // Compute a random salt
            let mut nonce = [0; SALT_LENGTH];
            OsRng::fill_bytes(&mut OsRng, &mut nonce[..]);
            nonce
        }
    };
    hkdf::Hkdf::<Sha512>::new(Some(&salt), shared)
        .expand(context, &mut hex)
        .unwrap();

    let (key, nonce) = split_keys(&hex)?;
    // Return the array
    Ok((salt, key, nonce))
}

#[cfg(test)]
mod tests {
    use test::Bencher;
    use x448::{PublicKey, Secret};

    use super::*;
    use crate::MyKeyringError;

    const MESSAGE: &[u8] = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec et ultricies augue. Etiam ultrices massa diam, id laoreet neque lobortis.";

    #[bench]
    fn bench_hkdf(b: &mut Bencher) {
        b.iter(move || {
            let shared = [0; 1024];
            let nonce = [1; 16];
            let context = "BLAKE3 context";
            let mut hex = [0; DERIVED_LENGTH];

            test::black_box(
                hkdf::Hkdf::<Sha512>::new(Some(&nonce), &shared)
                    .expand(context.as_bytes(), &mut hex)
                    .unwrap(),
            );
        })
    }

    #[bench]
    fn crypt_(b: &mut Bencher) {
        b.iter(move || {
            let secret_1 = Secret::new(&mut OsRng);

            let secret_2 = Secret::new(&mut OsRng);
            let public_key_2 = PublicKey::from(&secret_2);

            let shared = secret_1.as_diffie_hellman(&public_key_2).unwrap();

            test::black_box(super::crypt(shared, MESSAGE, None).unwrap());
        })
    }

    #[test]
    fn crypt_then_decrypt() -> crate::Result<()> {
        let secret_1 = Secret::new(&mut OsRng);
        let public_key_1 = PublicKey::from(&secret_1);

        let secret_2 = Secret::new(&mut OsRng);
        let public_key_2 = PublicKey::from(&secret_2);

        let shared_1 = secret_1.as_diffie_hellman(&public_key_2).unwrap();
        let shared_2 = secret_2.as_diffie_hellman(&public_key_1).unwrap();

        let encrypted = crypt(shared_1, MESSAGE, Some(b"a"))?;
        let clear = decrypt(shared_2, encrypted, Some(b"a"))?;
        assert_eq!(MESSAGE, &clear);

        Ok(())
    }

    #[test]
    fn crypt_then_wrong_decrypt() -> crate::Result<()> {
        let secret_1 = Secret::new(&mut OsRng);
        let public_key_1 = PublicKey::from(&secret_1);

        let secret_2 = Secret::new(&mut OsRng);
        let public_key_2 = PublicKey::from(&secret_2);
        let secret_3 = Secret::new(&mut OsRng);

        let shared_1 = secret_1.as_diffie_hellman(&public_key_2).unwrap();
        let shared_2 = secret_3.as_diffie_hellman(&public_key_1).unwrap();

        let encrypted = crypt(shared_1, MESSAGE, None)?;

        let wrong = decrypt(shared_2, encrypted, None);
        assert!(wrong.is_err());
        assert_eq!(wrong.unwrap_err(), MyKeyringError::IncorrectHmac);

        Ok(())
    }

    #[test]
    fn crypt_then_different_context() -> crate::Result<()> {
        let secret_1 = Secret::new(&mut OsRng);
        let public_key_1 = PublicKey::from(&secret_1);

        let secret_2 = Secret::new(&mut OsRng);
        let public_key_2 = PublicKey::from(&secret_2);

        let shared_1 = secret_1.as_diffie_hellman(&public_key_2).unwrap();
        let shared_2 = secret_2.as_diffie_hellman(&public_key_1).unwrap();

        let encrypted = crypt(shared_1, MESSAGE, None)?;

        let wrong = decrypt(shared_2, encrypted, Some(b"Other context"));
        assert!(wrong.is_err());
        assert_eq!(wrong.unwrap_err(), MyKeyringError::IncorrectHmac);

        Ok(())
    }
}
