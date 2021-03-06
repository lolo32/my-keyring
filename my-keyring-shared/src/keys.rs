use fnv::FnvHashMap;
use rand_core::OsRng;
use ulid::Ulid;
use x448::{PublicKey, Secret, SharedSecret};

pub struct KeyRing {
    my_key: Secret,
    known: FnvHashMap<Ulid, PublicKey>,
}

impl KeyRing {
    pub fn new(secret: Secret, known: FnvHashMap<Ulid, PublicKey>) -> Self {
        Self {
            my_key: secret,
            known,
        }
    }

    pub fn add_known(&mut self, known_id: Ulid, key: PublicKey) {
        self.known.insert(known_id, key);
    }

    pub fn del_known(&mut self, known_id: Ulid) {
        self.known.remove(&known_id);
    }

    pub fn get_shared_key(&self, key_id: Ulid) -> Option<SharedSecret> {
        if let Some(public_key) = self.known.get(&key_id) {
            self.my_key.as_diffie_hellman(public_key)
        } else {
            None
        }
    }

    pub fn shared_with_ephemeral(&self, public_key: PublicKey) -> (PublicKey, SharedSecret) {
        let ephemeral_secret = Secret::new(&mut OsRng);
        let ephemeral_public_key = PublicKey::from(&ephemeral_secret);
        (
            ephemeral_public_key,
            ephemeral_secret
                .to_diffie_hellman(&public_key)
                .expect("shared secret"),
        )
    }

    pub fn shared_from_ephemeral(&self, public_key: PublicKey) -> SharedSecret {
        self.my_key
            .as_diffie_hellman(&public_key)
            .expect("shared secret")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_ephemeral() {
        let secret = Secret::new(&mut OsRng);
        let public_key = PublicKey::from(&secret);

        // Ephemeral
        let keyring = KeyRing::new(secret, Default::default());
        let (public_ephemeral, shared_ephemeral) = keyring.shared_with_ephemeral(public_key);

        // From ephemeral public key
        let shared_keyring = keyring.shared_from_ephemeral(public_ephemeral);

        assert_eq!(shared_ephemeral.as_bytes(), shared_keyring.as_bytes());
    }
}
