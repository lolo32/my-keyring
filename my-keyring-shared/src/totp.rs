use std::time::SystemTime;

use base32::{decode, Alphabet};

use crate::algo::Algorithm;
use crate::errors::MyKeyringError;

/// Decode a base32 encoded string, removing padding and optional `-` or `spaces`
///
/// # Examples
///
/// All these examples return the same result
/// ```
/// use my_keyring_shared::totp::decode_base32;
/// # fn main() -> my_keyring_shared::Result<()> {
/// // Literally
/// let literally = decode_base32("JBSWY3DPEB3W64TMMQQQ")?;
/// // With padding
/// let padding = decode_base32("JBSWY3DPEB3W64TMMQQQ==")?;
/// // With extra characters for readability
/// let extra_chars = decode_base32("JBSW Y3DP-EB3W 64TM-MQQQ")?;
///
/// assert_eq!(extra_chars, b"Hello world!");
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn decode_base32(input: &str) -> crate::Result<Vec<u8>> {
    let encoded = input
        .trim_end_matches(|c| c == '=')
        .replace("-", "")
        .replace(" ", "");

    decode(Alphabet::RFC4648 { padding: false }, &encoded).ok_or(MyKeyringError::InvalidBase32)
}

/// Generate a Totp, based either on the current timestamp, or arbitrary value
///
/// # Examples
///
/// ```
/// use my_keyring_shared::{Algorithm, totp::Totp};
/// let totp = Totp::new("JBSWY3DPEB3W64TMMQQQ", 6, 30, Algorithm::Sha1);
///
/// // Current timestamp
/// println!("{}", totp.totp());
/// // Arbitral timestamp, 1234567890 here
/// println!("{}", totp.totp_from_timestamp(1_234_567_890));
/// ```
#[derive(Debug)]
pub struct Totp {
    /// Secret to use
    secret: String,
    /// Number of digits, 6 (default) or 8
    digits: u8,
    /// Period of validity of the token (30 secs by default)
    period: u32,
    /// Algorithm to use during Totp generation
    algorithm: Algorithm,
}

impl Totp {
    /// Initialise a new Totp
    ///
    /// # Defaults valued
    ///
    /// The `secret` is indicated from the website,
    /// `digits` is the desired length, defaulting to 6,
    /// `period` is the window timestamp validity, defaulting to 30 seconds,
    /// `algorithm` is the algorithm used to generate, defaulting to Sha-1.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_keyring_shared::{Algorithm, totp::Totp};
    /// # fn main() -> my_keyring_shared::Result<()> {
    /// // Specifying only the secret
    /// let totp = Totp::new("JBSWY3DPEB3W64TMMQQQ", None, None, None);
    ///
    /// // Specifying the other parameters
    /// let totp = Totp::new("JBSWY3DPEB3W64TMMQQQ", 8, Some(30), Some(Algorithm::Sha1));
    ///
    /// # println!("{}", totp.totp());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Return [`MyKeyringError::InvalidBase32`] error if the provided `secret` is not
    /// a valid base32 encoded string
    pub fn new(
        secret: &str,
        digits: impl Into<Option<u8>>,
        period: impl Into<Option<u32>>,
        algorithm: impl Into<Option<Algorithm>>,
    ) -> crate::Result<Self> {
        // Do this to check base32 string
        let _ = decode_base32(secret)?;

        Ok(Self {
            secret: secret.to_owned(),
            digits: digits.into().unwrap_or(6).max(1),
            period: period.into().unwrap_or(30).max(1),
            algorithm: algorithm.into().unwrap_or(Algorithm::Sha1),
        })
    }

    /// Retrieve the Totp value based on current timestamp
    ///
    /// # Examples
    ///
    /// ```
    /// use my_keyring_shared::totp::Totp;
    /// let totp = Totp::new("JBSWY3DPEB3W64TMMQQQ", None, None, None);
    ///
    /// println!("{}", totp.totp());
    /// // Will print something like: 037194
    /// ```
    #[inline]
    pub fn totp(&self) -> String {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.totp_from_timestamp(timestamp)
    }

    /// Retrieve the Totp, based on the `timestamp` parameter value
    ///
    /// # Examples
    ///
    /// ```
    /// use my_keyring_shared::totp::Totp;
    /// let totp = Totp::new("JFIFCUSTKRKVMV2IJFIFCUSTKRKVMV2I", None, None, None);
    ///
    /// println!("{}", totp.totp_from_timestamp(1_234_567_890));
    /// ```
    pub fn totp_from_timestamp(&self, timestamp: u64) -> String {
        // Generate the counter based on the period window
        let counter = timestamp / u64::from(self.period);

        // Compute the Hmac
        let digest = self.algorithm.hmac(
            &decode_base32(&self.secret).expect("Base32 decoded string"),
            &counter.to_be_bytes(),
        );

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

    #[test]
    fn base32_rfc4648_test_vector_decoding() {
        // Base32
        assert_eq!(decode_base32("MY======").unwrap(), b"f");
        assert_eq!(decode_base32("MZXQ====").unwrap(), b"fo");
        assert_eq!(decode_base32("MZXW6===").unwrap(), b"foo");
        assert_eq!(decode_base32("MZXW6YQ=").unwrap(), b"foob");
        assert_eq!(decode_base32("MZXW6YTB").unwrap(), b"fooba");
        assert_eq!(decode_base32("MZXW6YTBOI").unwrap(), b"foobar");
    }

    #[bench]
    fn bench_totp_sha1_8chars(b: &mut Bencher) {
        // 12345678901234567890
        let seed = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
        let t = Totp::new(seed, 8, None, None).unwrap();
        b.iter(move || t.totp())
    }

    #[bench]
    fn bench_totp_sha256_8chars(b: &mut Bencher) {
        // 12345678901234567890
        let seed = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
        let t = Totp::new(seed, 8, None, Algorithm::Sha256).unwrap();
        b.iter(move || t.totp())
    }

    #[bench]
    fn bench_totp_sha1_6chars(b: &mut Bencher) {
        // 12345678901234567890
        let seed = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
        let t = Totp::new(seed, 6, None, None).unwrap();
        b.iter(move || t.totp())
    }

    #[test]
    fn tests_vectors_rfc6238_sha1_8chars() -> crate::Result<()> {
        // 12345678901234567890
        let seed = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
        let t = Totp::new(seed, Some(8), None, None)?;
        assert_eq!(t.totp_from_timestamp(59), "94287082");
        assert_eq!(t.totp_from_timestamp(1_111_111_109), "07081804");
        assert_eq!(t.totp_from_timestamp(1_111_111_111), "14050471");
        assert_eq!(t.totp_from_timestamp(1_234_567_890), "89005924");
        assert_eq!(t.totp_from_timestamp(2_000_000_000), "69279037");
        assert_eq!(t.totp_from_timestamp(20_000_000_000), "65353130");

        Ok(())
    }

    #[test]
    fn tests_vectors_rfc6238_sha256_8chars() -> crate::Result<()> {
        // 12345678901234567890123456789012
        let seed = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZA";
        let t = Totp::new(seed, 8, None, Algorithm::Sha256)?;
        assert_eq!(t.totp_from_timestamp(59), "46119246");
        assert_eq!(t.totp_from_timestamp(1_111_111_109), "68084774");
        assert_eq!(t.totp_from_timestamp(1_111_111_111), "67062674");
        assert_eq!(t.totp_from_timestamp(1_234_567_890), "91819424");
        assert_eq!(t.totp_from_timestamp(2_000_000_000), "90698825");
        assert_eq!(t.totp_from_timestamp(20_000_000_000), "77737706");

        Ok(())
    }

    #[test]
    fn tests_vectors_rfc4226_sha1_6chars() -> crate::Result<()> {
        // These are normally HTOP test vectors, but can be used if `period` is one second

        // 12345678901234567890
        let seed = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
        let t = Totp::new(seed, None, 1, Some(Algorithm::Sha1))?;
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

        Ok(())
    }
}
