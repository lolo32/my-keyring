use std::fmt;

use c2_chacha::{
    stream_cipher::{NewStreamCipher, SyncStreamCipher},
    XChaCha20,
};
use hmac::{Hmac, Mac, NewMac};
use rand_core::{OsRng, RngCore};
use sha2::Sha512;
use x448::SharedSecret;

use crate::errors::MyKeyringError;

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

/// Represent a message encrypted, and the information needed to decrypt it
pub struct CryptedMessage {
    /// Public salt to derive passwords/iv for encryption/hmac
    pub salt: Salt,
    /// Data encrypted
    pub data: Vec<u8>,
    /// Hmac of the `data`
    pub hmac: [u8; HMAC_LENGTH],
}

impl fmt::Debug for CryptedMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CryptedMessage { .. }")
    }
}

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
    let mut mac = <Hmac<Sha512>>::new_varkey(&key_hmac).expect("HmacSha512 engine");
    mac.update(&encrypted_message.data);
    // result of the HMAC
    let res = mac.finalize();
    encrypted_message.hmac.copy_from_slice(&res.into_bytes());

    encrypted_message
}

/// Decrypt an encrypted message, based on `based_secret`, the `encrypted` message data and
/// the number of iterations
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

    // Initialise HMAC
    let mut mac = <Hmac<Sha512>>::new_varkey(&key_hmac).expect("HmacSha512 engine");
    mac.update(&encrypted.data);
    // Check the HMAC
    mac.verify(&encrypted.hmac)
        .map_err(|_| MyKeyringError::IncorrectHmac)?;

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
    use x448::{PublicKey, Secret};

    use super::*;

    const ITERATIONS: u32 = 100_000;
    const MESSAGE: &[u8] = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec et ultricies augue. Etiam ultrices massa diam, id laoreet neque lobortis.";

    #[test]
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
        assert_eq!(wrong.err().unwrap(), MyKeyringError::IncorrectHmac);
    }
}
