use std::time::SystemTime;

use base32::{decode, Alphabet};
use hmac::{Hmac, Mac, NewMac};
use sha1::Sha1;
use sha2::Sha256;

use crate::errors::MyKeyringError;
use hmac::digest::generic_array::ArrayLength;
use hmac::digest::{BlockInput, FixedOutput, Reset, Update};

/// Decode a base32 encoded string, removing padding and optional `-` or `spaces`
#[inline]
pub fn decode_base32(input: &str) -> crate::Result<Vec<u8>> {
    let encoded = input
        .trim_end_matches(|c| c == '=')
        .replace("-", "")
        .replace(" ", "");

    decode(Alphabet::RFC4648 { padding: false }, &encoded).ok_or(MyKeyringError::InvalidBase32)
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Algorithm {
    Sha1,
    Sha256,
}

impl Algorithm {
    #[must_use]
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
        }
    }
}

#[derive(Debug)]
pub struct Totp {
    // Secret to use
    secret: Vec<u8>,
    // Number of digits, 6 (default) or 8
    digits: u8,
    // Period of validity of the token (30 secs by default)
    period: u32,
    algoritm: Algorithm,
}

impl Totp {
    /// Initialise a new Totp
    ///
    /// # Parameters with defaults valued
    ///
    /// The `secret` is the
    pub fn new(
        secret: &[u8],
        digits: impl Into<Option<u8>>,
        period: impl Into<Option<u32>>,
        algorithm: impl Into<Option<Algorithm>>,
    ) -> Self {
        let digits = digits.into();
        let period = period.into();
        let algorithm = algorithm.into();
        Self {
            secret: secret.to_vec(),
            digits: digits.unwrap_or(6).max(1),
            period: period.unwrap_or(30).max(1),
            algoritm: algorithm.unwrap_or(Algorithm::Sha1),
        }
    }

    /// Retrieve the Totp value based on current timestamp
    #[must_use]
    pub fn totp(&self) -> String {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.totp_from_timestamp(timestamp)
    }

    /// Retrieve the Totp, based on the `timestamp` parameter value
    pub fn totp_from_timestamp(&self, timestamp: u64) -> String {
        // Generate the counter based on the period window
        let counter = timestamp / u64::from(self.period);

        // Compute the Hmac
        let digest = self.algoritm.hmac(&self.secret, &counter.to_be_bytes());

        // Truncate
        let offset = (digest.last().expect("last array member") & 0xf) as usize;
        let binary = (u64::from(digest[offset]) & 0x7f) << 24
            | (u64::from(digest[offset + 1]) & 0xff) << 16
            | (u64::from(digest[offset + 2]) & 0xff) << 8
            | u64::from(digest[offset + 3]) & 0xff;
        let binary = binary % (10_u64.pow(self.digits.into()));

        // Prepend with additional 0 to have digits length Token and convert it to String
        format!("{:0>1$}", binary, usize::from(self.digits))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn base32_decoding() {
        let s = b"Hello world!";
        // Literally
        assert_eq!(decode_base32("JBSWY3DPEB3W64TMMQQQ").unwrap(), s);
        // With padding
        assert_eq!(decode_base32("JBSWY3DPEB3W64TMMQQQ==").unwrap(), s);
        // With extra characters for readability
        assert_eq!(decode_base32("JBSW Y3DP-EB3W 64TM-MQQQ").unwrap(), s);
    }

    #[bench]
    fn bench_totp_sha1_8chars(b: &mut Bencher) {
        let seed = b"12345678901234567890";
        let t = Totp::new(seed, 8, None, None);
        b.iter(move || t.totp())
    }

    #[bench]
    fn bench_totp_sha256_8chars(b: &mut Bencher) {
        let seed = b"12345678901234567890";
        let t = Totp::new(seed, 8, None, Algorithm::Sha256);
        b.iter(move || t.totp())
    }

    #[bench]
    fn bench_totp_sha1_6chars(b: &mut Bencher) {
        let seed = b"12345678901234567890";
        let t = Totp::new(seed, 6, None, None);
        b.iter(move || t.totp())
    }

    #[test]
    fn tests_vectors_rfc6238_sha1_8chars() {
        let seed = b"12345678901234567890";
        let t = Totp::new(seed, Some(8), None, None);
        assert_eq!(t.totp_from_timestamp(59), "94287082");
        assert_eq!(t.totp_from_timestamp(1_111_111_109), "07081804");
        assert_eq!(t.totp_from_timestamp(1_111_111_111), "14050471");
        assert_eq!(t.totp_from_timestamp(1_234_567_890), "89005924");
        assert_eq!(t.totp_from_timestamp(2_000_000_000), "69279037");
        assert_eq!(t.totp_from_timestamp(20_000_000_000), "65353130");
    }

    #[test]
    fn tests_vectors_rfc6238_sha256_8chars() {
        let seed = b"12345678901234567890123456789012";
        let t = Totp::new(seed, 8, None, Algorithm::Sha256);
        assert_eq!(t.totp_from_timestamp(59), "46119246");
        assert_eq!(t.totp_from_timestamp(1_111_111_109), "68084774");
        assert_eq!(t.totp_from_timestamp(1_111_111_111), "67062674");
        assert_eq!(t.totp_from_timestamp(1_234_567_890), "91819424");
        assert_eq!(t.totp_from_timestamp(2_000_000_000), "90698825");
        assert_eq!(t.totp_from_timestamp(20_000_000_000), "77737706");
    }

    #[test]
    fn tests_vectors_rfc4226_sha1_6chars() {
        // These are normally HTOP test vectors, but can be used if `period` is one second
        let seed = b"12345678901234567890";
        let t = Totp::new(seed, None, 1, Some(Algorithm::Sha1));
        assert_eq!(t.totp_from_timestamp(0), "755224");
        assert_eq!(t.totp_from_timestamp(1), "287082");
        assert_eq!(t.totp_from_timestamp(2), "359152");
        assert_eq!(t.totp_from_timestamp(3), "969429");
        assert_eq!(t.totp_from_timestamp(4), "338314");
        assert_eq!(t.totp_from_timestamp(5), "254676");
        assert_eq!(t.totp_from_timestamp(6), "287922");
        assert_eq!(t.totp_from_timestamp(7), "162583");
        assert_eq!(t.totp_from_timestamp(8), "399871");
        assert_eq!(t.totp_from_timestamp(9), "520489");
    }
}
