use fnv::FnvHashMap;
use rand_core::OsRng;
use ulid::Ulid;
use x448::{PublicKey, Secret, SharedSecret};

/// The `PublicKey` pool holder and keep the `Secret`
pub struct KeyRing {
    /// My `Secret` key
    my_key: Secret,
    /// The `PublicKey` pool
    known: FnvHashMap<Ulid, PublicKey>,
}

impl KeyRing {
    /// Generate a new `KeyRing`
    pub fn new(secret: Secret, known: FnvHashMap<Ulid, PublicKey>) -> Self {
        Self {
            my_key: secret,
            known,
        }
    }

    /// Add a new known key to the `PublicKey` known pool
    pub fn add_known(&mut self, known_id: Ulid, key: PublicKey) {
        self.known.insert(known_id, key);
    }

    /// Remove the key from the pool
    pub fn del_known(&mut self, known_id: Ulid) {
        self.known.remove(&known_id);
    }

    /// Return the associated `PublicKey` based on the `key_id` from the pool
    pub fn get_shared_key(&self, key_id: Ulid) -> Option<SharedSecret> {
        if let Some(public_key) = self.known.get(&key_id) {
            self.my_key.as_diffie_hellman(public_key)
        } else {
            None
        }
    }

    /// Generate a `SharedSecret` based on an ephemeral private key that is not
    /// used outside of this function.
    ///
    /// It returns the associated `PublicKey` that must be used to compute the
    /// `shared_secret` on the other side so it must be send to the other side,
    /// and the `SharedSecret` computed from the `public_key` parameter and the ephemeral
    /// private key
    pub fn shared_with_ephemeral(&self, public_key: &PublicKey) -> (PublicKey, SharedSecret) {
        let ephemeral_secret = Secret::new(&mut OsRng);
        let ephemeral_public_key = PublicKey::from(&ephemeral_secret);
        (
            ephemeral_public_key,
            ephemeral_secret
                .to_diffie_hellman(&public_key)
                .expect("shared secret"),
        )
    }

    /// Generate a `SharedSecret` from the `public_key` and the local `PrivateKey`
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
        let alice_secret = Secret::new(&mut OsRng);
        let alice_public = PublicKey::from(&alice_secret);

        // Ephemeral
        let alice_keyring = KeyRing::new(alice_secret, Default::default());
        let (public_ephemeral, shared_ephemeral) =
            alice_keyring.shared_with_ephemeral(&alice_public);

        // From ephemeral public key
        let shared_keyring = alice_keyring.shared_from_ephemeral(public_ephemeral);

        assert_eq!(shared_ephemeral.as_bytes(), shared_keyring.as_bytes());
    }
}
