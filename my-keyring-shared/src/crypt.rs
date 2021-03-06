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

type Salt = [u8; 16];
type Key = [u8; 32];
type Iv = [u8; 24];
type Derived = [u8; 88];

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
) -> Result<Vec<u8>, ()> {
    let (_salt, keys) = derive_keys(shared.as_bytes(), Some(encrypted.salt), iterations);
    let (key_chacha, iv_chacha, key_hmac) = split_keys(&keys);

    // Check HMAC
    let mut mac = <Hmac<Sha512>>::new_varkey(&key_hmac).expect("HMAC");
    mac.update(&encrypted.data);
    // result of the HMAC
    if mac.verify(&encrypted.hmac).is_err() {
        return Err(());
    }

    // Create cipher instance
    let mut cipher = XChaCha20::new_var(&key_chacha, &iv_chacha).expect("xchacha20");
    // apply keystream (encrypt)
    cipher.apply_keystream(&mut encrypted.data);

    Ok(encrypted.data)
}

fn split_keys(keys: &[u8]) -> (Key, Iv, Key) {
    let mut key_1 = [0; 32];
    key_1.copy_from_slice(&keys[..32]);

    let mut iv = [0; 24];
    iv.copy_from_slice(&keys[32..56]);

    let mut key_2 = [0; 32];
    key_2.copy_from_slice(&keys[56..88]);

    (key_1, iv, key_2)
}

fn derive_keys(shared: &[u8], nonce: Option<Salt>, iterations: u32) -> (Salt, Derived) {
    let mut hex = [0; 88];
    // Generate a salt if none
    let nonce = match nonce {
        Some(nonce) => nonce,
        None => {
            // Compute a random salt
            let mut nonce = [0; 16];
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

    use crate::keys::KeyRing;

    use super::*;

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

        let encrypted = crypt(shared_1, clear_ref.as_slice(), 50_000);
        let clear = decrypt(shared_2, encrypted, 50_000);
        assert!(clear.is_ok());
        assert_eq!(clear_ref.as_slice(), &clear.unwrap());
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

        let encrypted = crypt(shared_1, clear_ref.as_slice(), 50_000);
        let clear = decrypt(shared_2, encrypted, 50_000);
        assert!(clear.is_err());
    }
}
