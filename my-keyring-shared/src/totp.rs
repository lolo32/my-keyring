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
    pub fn new(secret: &[u8], digits: u8, period: Option<u32>) -> Self {
        Self {
            secret: secret.to_vec(),
            digits,
            period: period.unwrap_or(30).max(1),
        }
    }

    #[must_use]
    pub fn totp(&self) -> String {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let counter = timestamp / u64::from(self.period);

        // Create the HMAC
        let mut mac = <Hmac<Sha1>>::new_varkey(&self.secret).expect("Hmac creation failed");
        // Do the hashing
        mac.update(&counter.to_be_bytes());
        // Return the result
        let digest = mac.finalize().into_bytes().to_vec();

        // Truncate
        let off = (match digest.last() {
            Some(byte) => byte,
            None => unreachable!(),
        } & 0xf) as usize;
        let binary = (u64::from(digest[off]) & 0x7f) << 24
            | (u64::from(digest[off + 1]) & 0xff) << 16
            | (u64::from(digest[off + 2]) & 0xff) << 8
            | u64::from(digest[off + 3]) & 0xff;
        let binary = binary % (10_u64.pow(self.digits.into()));

        // Prepend with additional 0 to have digits length Token and convert it to String
        format!("{:0>1$}", binary, usize::from(self.digits))
    }
}
