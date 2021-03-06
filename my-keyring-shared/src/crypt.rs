use crate::errors::MyKeyringError;
use c2_chacha::{
    stream_cipher::{NewStreamCipher, SyncStreamCipher},
    XChaCha20,
};
use hmac::{Hmac, Mac, NewMac};
use rand_core::{OsRng, RngCore};
use sha2::Sha512;
use x448::SharedSecret;

pub struct CryptedMessage {
    pub salt: Salt,
    pub data: Vec<u8>,
    pub hmac: Vec<u8>,
}

pub const SALT_LENGTH: usize = 16;
pub const KEY_LENGTH: usize = 32;
pub const IV_LENGTH: usize = 24;
pub const DERIVED_LENGTH: usize = KEY_LENGTH * 2 + IV_LENGTH;

type Salt = [u8; SALT_LENGTH];
type Key = [u8; KEY_LENGTH];
type Iv = [u8; IV_LENGTH];
type Derived = [u8; DERIVED_LENGTH];

pub fn crypt(shared: SharedSecret, data: &[u8], iterations: u32) -> CryptedMessage {
    let (salt, keys) = derive_keys(shared.as_bytes(), None, iterations);

    let mut result = CryptedMessage {
        salt,
        data: data[..].to_vec(),
        hmac: Vec::new(),
    };

    let (key_chacha, iv_chacha, key_hmac) = split_keys(&keys);

    // Create cipher instance
    let mut cipher = XChaCha20::new_var(&key_chacha, &iv_chacha).expect("xchacha20");
    // apply keystream (encrypt)
    cipher.apply_keystream(&mut result.data);

    // compute HMAC
    let mut mac = <Hmac<Sha512>>::new_varkey(&key_hmac).expect("HMAC");
    mac.update(&result.data);
    // result of the HMAC
    let res = mac.finalize();
    result.hmac = res.into_bytes().to_vec();

    result
}

pub fn decrypt(
    shared: SharedSecret,
    mut encrypted: CryptedMessage,
    iterations: u32,
) -> crate::Result<Vec<u8>> {
    let (_salt, keys) = derive_keys(shared.as_bytes(), Some(encrypted.salt), iterations);
    let (key_chacha, iv_chacha, key_hmac) = split_keys(&keys);

    // Initialise HMAC
    let mut mac = <Hmac<Sha512>>::new_varkey(&key_hmac).expect("HMAC");
    mac.update(&encrypted.data);
    // Check the HMAC
    mac.verify(&encrypted.hmac)
        .map_err(|_| MyKeyringError::IncorrectHmac)?;

    // Create cipher instance
    let mut cipher = XChaCha20::new_var(&key_chacha, &iv_chacha).expect("xchacha20");
    // apply keystream (encrypt)
    cipher.apply_keystream(&mut encrypted.data);

    Ok(encrypted.data)
}

fn split_keys(keys: &[u8]) -> (Key, Iv, Key) {
    let mut boundary = (0, KEY_LENGTH);
    let mut key_1 = [0; KEY_LENGTH];
    key_1.copy_from_slice(&keys[(boundary.0)..(boundary.1)]);

    boundary = (boundary.1, boundary.1 + IV_LENGTH);
    let mut iv = [0; IV_LENGTH];
    iv.copy_from_slice(&keys[(boundary.0)..(boundary.1)]);

    boundary = (boundary.1, boundary.1 + KEY_LENGTH);
    let mut key_2 = [0; KEY_LENGTH];
    key_2.copy_from_slice(&keys[(boundary.0)..(boundary.1)]);

    (key_1, iv, key_2)
}

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

    const ITERATIONS: u32 = 50_000;

    #[test]
    fn crypt_then_decrypt() {
        let secret_1 = Secret::new(&mut OsRng);
        let public_key_1 = PublicKey::from(&secret_1);

        let secret_2 = Secret::new(&mut OsRng);
        let public_key_2 = PublicKey::from(&secret_2);

        let shared_1 = secret_1
            .as_diffie_hellman(&public_key_2)
            .expect("shared secret");
        let shared_2 = secret_2
            .as_diffie_hellman(&public_key_1)
            .expect("shared secret");

        let clear_ref = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec et ultricies augue. Etiam ultrices massa diam, id laoreet neque lobortis.";

        let encrypted = crypt(shared_1, &clear_ref[..], ITERATIONS);
        let clear = decrypt(shared_2, encrypted, ITERATIONS);
        assert!(clear.is_ok());
        assert_eq!(&clear_ref[..], &clear.unwrap());
    }

    #[test]
    fn crypt_then_wrong_decrypt() {
        let secret_1 = Secret::new(&mut OsRng);
        let public_key_1 = PublicKey::from(&secret_1);

        let secret_2 = Secret::new(&mut OsRng);
        let public_key_2 = PublicKey::from(&secret_2);
        let secret_3 = Secret::new(&mut OsRng);

        let shared_1 = secret_1
            .as_diffie_hellman(&public_key_2)
            .expect("shared secret");
        let shared_2 = secret_3
            .as_diffie_hellman(&public_key_1)
            .expect("shared secret");

        let clear_ref = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec et ultricies augue. Etiam ultrices massa diam, id laoreet neque lobortis.";

        let encrypted = crypt(shared_1, &clear_ref[..], ITERATIONS);
        let clear = decrypt(shared_2, encrypted, ITERATIONS);
        assert!(clear.is_err());
    }
}
