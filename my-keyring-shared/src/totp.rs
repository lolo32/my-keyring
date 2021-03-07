use std::time::SystemTime;

use base32::{decode, Alphabet};
use hmac::{Hmac, Mac, NewMac};
use sha1::Sha1;

use crate::errors::MyKeyringError;

#[inline]
pub fn decode_base32(input: &str) -> crate::Result<Vec<u8>> {
    let encoded = input
        .trim_end_matches(|c| c == '=')
        .replace("-", "")
        .replace(" ", "");

    decode(Alphabet::RFC4648 { padding: false }, &encoded).ok_or(MyKeyringError::InvalidBase32)
}

#[derive(Debug)]
pub struct Totp {
    // Secret to use
    secret: Vec<u8>,
    // Number of digits, 6 (default) or 8
    digits: u8,
    // Period of validity of the token (30 secs by default)
    period: u32,
}

impl Totp {
    pub fn new(secret: &[u8], digits: Option<u8>, period: Option<u32>) -> Self {
        Self {
            secret: secret.to_vec(),
            digits: digits.unwrap_or(6).max(1),
            period: period.unwrap_or(30).max(1),
        }
    }

    #[must_use]
    pub fn totp(&self) -> String {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.totp_from_timestamp(timestamp)
    }

    pub fn totp_from_timestamp(&self, timestamp: u64) -> String {
        let counter = timestamp / u64::from(self.period);

        // Create the HMAC
        let mut mac = <Hmac<Sha1>>::new_varkey(&self.secret).expect("Hmac creation failed");
        // Do the hashing
        mac.update(&counter.to_be_bytes());
        // Return the result
        let digest = mac.finalize().into_bytes().to_vec();

        // Truncate
        let offset = (match digest.last() {
            Some(byte) => byte,
            None => unreachable!(),
        } & 0xf) as usize;
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

    #[test]
    fn base32_decoding() {
        let s = b"Hello world!";
        assert_eq!(decode_base32("JBSWY3DPEB3W64TMMQQQ").unwrap(), s);
        assert_eq!(decode_base32("JBSWY3DPEB3W64TMMQQQ==").unwrap(), s);
        assert_eq!(decode_base32("JBSW Y3DP-EB3W 64TM-MQQQ").unwrap(), s);
    }

    #[test]
    fn tests_vectors_rfc_sha1_8chars() {
        let seed = b"12345678901234567890";
        let t = Totp::new(seed, Some(8), None);
        assert_eq!(t.totp_from_timestamp(59), "94287082");
        assert_eq!(t.totp_from_timestamp(1_111_111_109), "07081804");
        assert_eq!(t.totp_from_timestamp(1_111_111_111), "14050471");
        assert_eq!(t.totp_from_timestamp(1_234_567_890), "89005924");
        assert_eq!(t.totp_from_timestamp(2_000_000_000), "69279037");
        assert_eq!(t.totp_from_timestamp(20_000_000_000), "65353130");
    }
}
