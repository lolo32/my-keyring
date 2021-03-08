use std::fmt;

use c2_chacha::{
    stream_cipher::{NewStreamCipher, SyncStreamCipher},
    XChaCha20,
};
use rand_core::{OsRng, RngCore};
use x448::SharedSecret;

use crate::algo::Algorithm;
use hmac::Hmac;
use sha2::Sha512;

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
///
/// # Examples
///
/// ```
/// # use x448::{Secret, PublicKey};
/// # let secret_1 = Secret::from_bytes(&[4, 36, 208, 209, 253, 39, 109, 9, 136, 133, 28, 92, 92, 85, 39, 187, 162, 244, 121, 149, 65, 178, 13, 45, 102, 116, 29, 119, 43, 134, 133, 48, 12, 113, 211, 217, 171, 72, 181, 1, 247, 71, 235, 9, 227, 186, 34, 54, 82, 153, 32, 57, 204, 178, 227, 212]).unwrap();
/// # let secret_2 = Secret::from_bytes(&[60, 126, 173, 166, 53, 200, 49, 76, 45, 1, 94, 173, 141, 43, 216, 220, 143, 128, 6, 191, 211, 195, 126, 33, 171, 41, 12, 66, 89, 143, 53, 124, 29, 71, 56, 98, 71, 167, 30, 144, 243, 19, 18, 179, 44, 103, 126, 149, 62, 246, 207, 141, 112, 194, 188, 144]).unwrap();
/// # let public_key_2 = PublicKey::from(&secret_2);
/// # let shared_secret = secret_1.as_diffie_hellman(&public_key_2).unwrap();
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
/// # use my_keyring_shared::crypt::CryptedMessage;
/// # let secret_1 = Secret::from_bytes(&[4, 36, 208, 209, 253, 39, 109, 9, 136, 133, 28, 92, 92, 85, 39, 187, 162, 244, 121, 149, 65, 178, 13, 45, 102, 116, 29, 119, 43, 134, 133, 48, 12, 113, 211, 217, 171, 72, 181, 1, 247, 71, 235, 9, 227, 186, 34, 54, 82, 153, 32, 57, 204, 178, 227, 212]).unwrap();
/// # let secret_2 = Secret::from_bytes(&[60, 126, 173, 166, 53, 200, 49, 76, 45, 1, 94, 173, 141, 43, 216, 220, 143, 128, 6, 191, 211, 195, 126, 33, 171, 41, 12, 66, 89, 143, 53, 124, 29, 71, 56, 98, 71, 167, 30, 144, 243, 19, 18, 179, 44, 103, 126, 149, 62, 246, 207, 141, 112, 194, 188, 144]).unwrap();
/// # let shared_secret = secret_1.as_diffie_hellman(&PublicKey::from(&secret_2)).unwrap();
/// # let encrypted_data = CryptedMessage {
/// #     salt: [46, 114, 42, 247, 114, 178, 92, 218, 139, 8, 250, 49, 230, 139, 253, 5],
/// #     data: vec![201, 229, 129, 4, 196, 107, 154, 88, 169, 218, 232, 211, 241, 102, 199, 41, 232, 165, 101, 149, 52, 85, 94, 167, 229, 130, 18, 217, 74, 110, 247, 101, 54, 255, 245, 28, 28, 134, 160, 67, 123, 159, 152, 201, 21, 222, 149, 211, 11, 207, 135, 55, 245, 135, 108, 61, 86, 162, 243, 108, 93, 143, 58, 203, 57, 9, 114, 182, 196, 218, 156, 101, 92, 226, 93, 117, 169, 73, 251, 250, 235, 226],
/// #     hmac: [124, 248, 104, 80, 231, 11, 47, 53, 74, 139, 12, 143, 210, 49, 157, 147, 208, 101, 51, 45, 167, 59, 27, 236, 188, 37, 82, 151, 176, 82, 36, 229, 222, 100, 231, 242, 87, 141, 67, 74, 109, 103, 146, 75, 97, 25, 90, 176, 55, 94, 134, 38, 221, 131, 191, 254, 167, 235, 12, 36, 207, 95, 127, 233]
/// # };
/// use my_keyring_shared::crypt::decrypt;
///
/// let decrypted_data = decrypt(shared_secret, encrypted_data, 20_000 );
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
    use x448::{PublicKey, Secret};

    use super::*;
    use crate::MyKeyringError;
    use test::Bencher;

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
