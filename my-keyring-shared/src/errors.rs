/// Error used in this library
#[derive(Debug, PartialEq)]
pub enum MyKeyringError {
    /// Diffie-Hellman error
    DHError,
    /// The HMAC verification failed
    IncorrectHmac,
    /// The provided base32's string is invalid
    InvalidBase32,
    /// The array of byte cannot be deserialized, it seems to be invalid
    InvalidCryptedMessage,
}
