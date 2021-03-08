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
//! let encrypted_data = crypt(shared_secret, text.as_bytes(), 20_000);
//!
//! // Serialize the data to send over a network link or store it
//! let array_data: Vec<u8> = encrypted_data.into();
//!
//! // Deserialize the data after download/storage read
//! let encrypted_data = CryptedMessage::try_from(array_data)?;
//!
//! // Decrypt the data, can return an error if shared_secret and/or iterations is invalid
//! # let shared_secret = secret_1.as_diffie_hellman(&public_key_2).unwrap();
//! let data = decrypt(shared_secret, encrypted_data, 20_000)?;
//! let data_text = String::from_utf8(data).expect("valid utf8 string");
//!
//! assert_eq!(text, data_text);
//! # Ok(())
//! # }
//! ```

use c2_chacha::{
    stream_cipher::{NewStreamCipher, SyncStreamCipher},
    XChaCha20,
};
use hmac::Hmac;
use rand_core::{OsRng, RngCore};
use sha2::Sha512;
use x448::SharedSecret;

use crate::algo::Algorithm;

pub use self::message::CryptedMessage;

mod message;

/// HMAC array length
pub const HMAC_LENGTH: usize = 64;
/// Salt array length
pub const SALT_LENGTH: usize = 16;
/// Derivation key array length
pub const KEY_LENGTH: usize = 32;
/// IV array length
pub const IV_LENGTH: usize = 24;
/// Length of the derived PBKDF2 used
const DERIVED_LENGTH: usize = KEY_LENGTH * 2 + IV_LENGTH;

/// Represent a salt for PBKDF2 derivation
pub type Salt = [u8; SALT_LENGTH];
/// Key for encryption and Hmac signature
type Key = [u8; KEY_LENGTH];
/// IV of the HMAC signature
type Iv = [u8; IV_LENGTH];
/// Result of the PBKDF2
type Derived = [u8; DERIVED_LENGTH];

/// Encrypt a message, needing a `shared_secret`, the `data` and the number of `iterations`
/// for deriving the keys
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
///     20_000
/// );
/// ```
pub fn crypt(shared_secret: SharedSecret, data: &[u8], iterations: u32) -> CryptedMessage {
    // Derive the shared_secret using PBKDF2_HmacSha512
    let (salt, keys) = derive_keys(shared_secret.as_bytes(), None, iterations);

    // store data
    let mut encrypted_message = CryptedMessage {
        salt,
        data: data[..].to_vec(),
        hmac: [0; HMAC_LENGTH],
    };

    // split the derived password into usable keys/iv
    let (key_chacha, iv_chacha, key_hmac) = split_keys(&keys);

    // Create cipher instance
    let mut cipher = XChaCha20::new_var(&key_chacha, &iv_chacha).expect("XChaCha20 engine");
    // apply keystream (encrypt)
    cipher.apply_keystream(&mut encrypted_message.data);

    // compute HMAC
    let digest = Algorithm::Sha512.hmac(&key_hmac, &encrypted_message.data);
    encrypted_message.hmac.copy_from_slice(&digest);

    encrypted_message
}

/// Decrypt an encrypted message, based on `based_secret`, the `encrypted` message data and
/// the number of iterations
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
/// #         20_000
/// #     )
/// # };
/// use my_keyring_shared::crypt::decrypt;
///
/// let decrypted_data = decrypt(shared_secret, receive_encrypted_message(), 20_000 );
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
/// The value `MyKeyringError::IncorrectHmac` can be returned if the `shared_secret` is not
/// valid, so the HMAC signature cannot be checked, or if the message has been altered
pub fn decrypt(
    shared_secret: SharedSecret,
    mut encrypted: CryptedMessage,
    iterations: u32,
) -> crate::Result<Vec<u8>> {
    // Derive the shared_secret using PBKDF2_HmacSha512
    let (_salt, keys) = derive_keys(shared_secret.as_bytes(), Some(encrypted.salt), iterations);
    // split the derived password into usable keys/iv
    let (key_chacha, iv_chacha, key_hmac) = split_keys(&keys);

    // Check the Hmac signature
    Algorithm::Sha512.hmac_verify(&key_hmac, &encrypted.data, &encrypted.hmac)?;

    // Create cipher instance
    let mut cipher = XChaCha20::new_var(&key_chacha, &iv_chacha).expect("XChaCha20 engine");
    // apply keystream (encrypt)
    cipher.apply_keystream(&mut encrypted.data);

    Ok(encrypted.data)
}

/// Split the derived keys from PBKDF2 into usable array data
fn split_keys(keys: &[u8]) -> (Key, Iv, Key) {
    // extract the first key (encryption)
    let mut boundary = (0, KEY_LENGTH);
    let mut key_1 = [0; KEY_LENGTH];
    key_1.copy_from_slice(&keys[(boundary.0)..(boundary.1)]);
    // Extract the IV
    boundary = (boundary.1, boundary.1 + IV_LENGTH);
    let mut iv = [0; IV_LENGTH];
    iv.copy_from_slice(&keys[(boundary.0)..(boundary.1)]);
    // Extract the second key (hmac)
    boundary = (boundary.1, boundary.1 + KEY_LENGTH);
    let mut key_2 = [0; KEY_LENGTH];
    key_2.copy_from_slice(&keys[(boundary.0)..(boundary.1)]);

    (key_1, iv, key_2)
}

/// Derive a password using PBKDF2_HmacSha512 algorithm
///
/// The `shared` is the original password to be derived, the `nonce` is calculated randomly
/// if not specified and the number of `iterations` to used for derivation.
///
/// It returns the `salt`, either randomly generated if not specified or the `nonce` value if
/// passed, and the `derived` password.
fn derive_keys(shared: &[u8], nonce: Option<Salt>, iterations: u32) -> (Salt, Derived) {
    let mut hex = [0; DERIVED_LENGTH];
    // Generate a salt if none
    let nonce = match nonce {
        Some(nonce) => nonce,
        None => {
            // Compute a random salt
            let mut nonce = [0; SALT_LENGTH];
            OsRng::fill_bytes(&mut OsRng, &mut nonce[..]);
            nonce
        }
    };
    // Compute the PBKDF2, based on the selected $hash
    pbkdf2::pbkdf2::<Hmac<Sha512>>(&shared, &nonce, iterations, &mut hex);
    // Return the array
    (nonce, hex)
}

#[cfg(test)]
mod tests {
    use test::Bencher;

    use x448::{PublicKey, Secret};

    use crate::MyKeyringError;

    use super::*;

    const ITERATIONS: u32 = 100_000;
    const MESSAGE: &[u8] = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec et ultricies augue. Etiam ultrices massa diam, id laoreet neque lobortis.";

    #[bench]
    #[ignore]
    fn crypt_(b: &mut Bencher) {
        b.iter(move || {
            let secret_1 = Secret::new(&mut OsRng);

            let secret_2 = Secret::new(&mut OsRng);
            let public_key_2 = PublicKey::from(&secret_2);

            let shared = secret_1.as_diffie_hellman(&public_key_2).unwrap();

            test::black_box(super::crypt(shared, MESSAGE, ITERATIONS));
        })
    }

    #[test]
    #[ignore]
    fn crypt_then_decrypt() -> crate::Result<()> {
        let secret_1 = Secret::new(&mut OsRng);
        let public_key_1 = PublicKey::from(&secret_1);

        let secret_2 = Secret::new(&mut OsRng);
        let public_key_2 = PublicKey::from(&secret_2);

        let shared_1 = secret_1.as_diffie_hellman(&public_key_2).unwrap();
        let shared_2 = secret_2.as_diffie_hellman(&public_key_1).unwrap();

        let encrypted = crypt(shared_1, MESSAGE, ITERATIONS);
        let clear = decrypt(shared_2, encrypted, ITERATIONS)?;
        assert_eq!(MESSAGE, &clear);

        Ok(())
    }

    #[test]
    #[ignore]
    fn crypt_then_wrong_decrypt() {
        let secret_1 = Secret::new(&mut OsRng);
        let public_key_1 = PublicKey::from(&secret_1);

        let secret_2 = Secret::new(&mut OsRng);
        let public_key_2 = PublicKey::from(&secret_2);
        let secret_3 = Secret::new(&mut OsRng);

        let shared_1 = secret_1.as_diffie_hellman(&public_key_2).unwrap();
        let shared_2 = secret_3.as_diffie_hellman(&public_key_1).unwrap();

        let encrypted = crypt(shared_1, MESSAGE, ITERATIONS);
        let wrong = decrypt(shared_2, encrypted, ITERATIONS);

        assert!(wrong.is_err());
        assert_eq!(wrong.unwrap_err(), MyKeyringError::IncorrectHmac);
    }
}
