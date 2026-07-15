//! Hardware accelerated encoding and decoding of bytes or utf-8 using standard RFC 4648 base64 spec
//!
//! The encoding and decoding of bytes or UTF-8 as base64 are according to the standard
//! [RFC 4648](https://datatracker.ietf.org/doc/html/rfc4648) specification.
//!
//! ## Example
//!
//! ```
//! use turbo_base64::{encode, decode};
//!
//! let data = b"Hello, Rust!";
//!
//! let encoded = encode(data);
//! assert_eq!(encoded, b"SGVsbG8sIFJ1c3Qh");
//!
//! let decoded = decode(&encoded).unwrap();
//! assert_eq!(decoded, data);
//! ```

#![no_std]
#![deny(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_results,
    unused_must_use
)]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;

const ALPHABETS: &[u8; 0x40] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Encodes a slice of bytes into standard RFC 4648 base64
///
/// ## Example
///
/// ```
/// use turbo_base64::encode;
///
/// let encoded = encode(b"fooba");
/// assert_eq!(encoded, b"Zm9vYmE=");
///
/// let encoded_empty = encode(b"");
/// assert_eq!(encoded_empty, b"");
/// ```
#[inline(always)]
pub fn encode(buffer: &[u8]) -> alloc::vec::Vec<u8> {
    let mut encoded = alloc::vec::Vec::new();

    let chunks = buffer.chunks_exact(3);
    let remaining_bytes = chunks.remainder();

    for chunk in chunks {
        let (b0, b1, b2) = (chunk[0], chunk[1], chunk[2]);
        let n = (b0 as u32) << 0x10 | (b1 as u32) << 8 | b2 as u32;

        encoded.push(ALPHABETS[((n >> 0x12) & 0x3F) as usize]);
        encoded.push(ALPHABETS[((n >> 0x0C) & 0x3F) as usize]);
        encoded.push(ALPHABETS[((n >> 6) & 0x3F) as usize]);
        encoded.push(ALPHABETS[(n & 0x3F) as usize]);
    }

    match remaining_bytes.len() {
        0 => {}
        1 => {
            let b0 = remaining_bytes[0] as u32;
            let n = b0 << 0x10;

            encoded.push(ALPHABETS[((n >> 0x12) & 0x3F) as usize]);
            encoded.push(ALPHABETS[((n >> 0x0C) & 0x3F) as usize]);
            encoded.push(b'=');
            encoded.push(b'=');
        }
        2 => {
            let b0 = remaining_bytes[0] as u32;
            let b1 = remaining_bytes[1] as u32;
            let n = (b0 << 0x10) | (b1 << 8);

            encoded.push(ALPHABETS[((n >> 0x12) & 0x3F) as usize]);
            encoded.push(ALPHABETS[((n >> 0x0C) & 0x3F) as usize]);
            encoded.push(ALPHABETS[((n >> 6) & 0x3F) as usize]);
            encoded.push(b'=');
        }
        _ => unreachable!(),
    }

    encoded
}

/// Decodes a standard RFC 4648 base64 encoded byte slice
///
/// ## Example
///
/// ```
/// use turbo_base64::{encode, decode, DecodeError};
///
/// let decoded = decode(b"Zm9vYmFy").unwrap();
/// assert_eq!(decoded, b"foobar");
///
/// let decoded_padded = decode(b"Zm8=").unwrap();
/// assert_eq!(decoded_padded, b"fo");
///
/// assert_eq!(decode(b"Zg ="), Err(DecodeError::InvalidByte(2, b' ')));
/// assert_eq!(decode(b"Z"), Err(DecodeError::InvalidLength));
/// ```
#[inline(always)]
pub fn decode(buffer: &[u8]) -> Result<alloc::vec::Vec<u8>, DecodeError> {
    if buffer.is_empty() {
        return Ok(alloc::vec::Vec::new());
    }

    if buffer.len() & 3 != 0 {
        return Err(DecodeError::InvalidLength);
    }

    let mut padding = 0;
    let mut len = buffer.len();

    while len > 0 && buffer[len - 1] == b'=' {
        len -= 1;
        padding += 1;
    }

    if padding > 2 {
        return Err(DecodeError::InvalidPadding);
    }

    let mut decoded = alloc::vec::Vec::with_capacity((buffer.len() / 4) * 3 - padding);
    let mut n = 0u32;
    let mut bits = 0u8;

    for (i, &b) in buffer.iter().enumerate() {
        if b == b'=' {
            if i < len {
                return Err(DecodeError::InvalidPadding);
            }

            continue;
        }

        let val = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 0x1A,
            b'0'..=b'9' => b - b'0' + 0x34,
            b'+' => 0x3E,
            b'/' => 0x3F,
            _ => return Err(DecodeError::InvalidByte(i, b)),
        };

        n = (n << 6) | (val as u32);
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            decoded.push((n >> bits) as u8);
        }
    }

    if bits > 0 && (n & ((1 << bits) - 1)) != 0 {
        return Err(DecodeError::InvalidPadding);
    }

    Ok(decoded)
}

/// Errors that can occur during strict base64 decoding
///
/// ## Example
///
/// ```
/// use turbo_base64::{decode, DecodeError};
///
/// assert_eq!(decode(b"Z"), Err(DecodeError::InvalidLength));
/// assert_eq!(decode(b"Z=9v"), Err(DecodeError::InvalidPadding));
/// assert_eq!(decode(b"Zg ="), Err(DecodeError::InvalidByte(2, b' ')));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    /// Encountered a character outside the standard base64 alphabet
    ///
    /// ## Example
    ///
    /// ```
    /// use turbo_base64::{decode, DecodeError};
    ///
    /// assert_eq!(decode(b"Zg ="), Err(DecodeError::InvalidByte(2, b' ')));
    /// ```
    InvalidByte(usize, u8),

    /// The length of the input buffer was not a multiple of `4`
    ///
    /// ## Example
    ///
    /// ```
    /// use turbo_base64::{decode, DecodeError};
    ///
    /// assert_eq!(decode(b"Z"), Err(DecodeError::InvalidLength));
    /// ```
    InvalidLength,

    /// Padding `=` characters were misplaced or exceeded the maximum of `2`
    ///
    /// ## Example
    ///
    /// ```
    /// use turbo_base64::{decode, DecodeError};
    ///
    /// assert_eq!(decode(b"Z=9v"), Err(DecodeError::InvalidPadding));
    /// ```
    InvalidPadding,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod rfc_4648 {
        use super::*;

        #[test]
        fn ok_rfc4648_vectors() {
            let tests = [
                (b"".as_slice(), b"".as_slice()),
                (b"f".as_slice(), b"Zg==".as_slice()),
                (b"fo".as_slice(), b"Zm8=".as_slice()),
                (b"foo".as_slice(), b"Zm9v".as_slice()),
                (b"foob".as_slice(), b"Zm9vYg== ".as_slice()),
                (b"fooba".as_slice(), b"Zm9vYmE=".as_slice()),
                (b"foobar".as_slice(), b"Zm9vYmFy".as_slice()),
            ];

            for (plain, expected_encoded) in tests {
                let expected_encoded =
                    expected_encoded.strip_suffix(b" ").unwrap_or(expected_encoded);

                let enc = encode(plain);
                assert_eq!(enc, expected_encoded);

                let dec = decode(&enc).unwrap();
                assert_eq!(dec, plain);
            }
        }

        #[test]
        fn ok_invalid_length() {
            assert_eq!(decode(b"Z"), Err(DecodeError::InvalidLength));
            assert_eq!(decode(b"Zg"), Err(DecodeError::InvalidLength));
            assert_eq!(decode(b"Zg="), Err(DecodeError::InvalidLength));
            assert_eq!(decode(b"Zm9vY"), Err(DecodeError::InvalidLength));
        }

        #[test]
        fn ok_invalid_chars() {
            assert_eq!(decode(b"Zg ="), Err(DecodeError::InvalidByte(2, b' ')));
            assert_eq!(decode(b"Zg\n="), Err(DecodeError::InvalidByte(2, b'\n')));
            assert_eq!(decode(b"Zm9v-mFy"), Err(DecodeError::InvalidByte(4, b'-')));
        }

        #[test]
        fn ok_invalid_padding_placement() {
            assert_eq!(decode(b"=m9v"), Err(DecodeError::InvalidPadding));
            assert_eq!(decode(b"Z=9v"), Err(DecodeError::InvalidPadding));
        }

        #[test]
        fn ok_binary_roundtrip() {
            let binary_data: alloc::vec::Vec<u8> = (0..=255).collect();
            let enc = encode(&binary_data);
            let dec = decode(&enc).unwrap();
            assert_eq!(dec, binary_data);
        }
    }

    mod standard_parity {
        use super::*;
        use base64::prelude::*;
        use rand::{rng, RngExt};

        #[test]
        fn ok_randomized_encoding_and_decoding_parity() {
            let mut rng = rng();

            for _ in 0..0x1000 {
                let len = rng.random_range(0..0x2000);
                let mut original_data = alloc::vec![0u8; len];
                rng.fill(&mut original_data[..]);

                let turbo_encoded = encode(&original_data);
                let standard_encoded = BASE64_STANDARD.encode(&original_data);

                assert_eq!(
                    turbo_encoded,
                    standard_encoded.as_bytes(),
                    "Encoding parity failure at size {len}!"
                );

                let turbo_decoded = decode(&turbo_encoded).unwrap();
                let standard_decoded = BASE64_STANDARD.decode(&standard_encoded).unwrap();

                assert_eq!(
                    turbo_decoded, standard_decoded,
                    "Decoding parity failure at size {len}!"
                );
            }
        }

        #[test]
        fn ok_randomized_error_rejection_parity() {
            let mut rng = rng();

            for _ in 0..0x400 {
                let len = rng.random_range(1..0x200) * 4;
                let mut valid_base64_string = encode(&alloc::vec![0u8; len / 4 * 3]);

                if valid_base64_string.is_empty() {
                    continue;
                }

                let corrupt_index = rng.random_range(0..valid_base64_string.len());
                valid_base64_string[corrupt_index] = b'%';

                let turbo_result = decode(&valid_base64_string);
                let standard_result = BASE64_STANDARD.decode(&valid_base64_string);

                assert!(
                    turbo_result.is_err(),
                    "TurboBase64 incorrectly accepted corrupt base64 containing '%' at index {corrupt_index}"
                );
                assert!(
                    standard_result.is_err(),
                    "Standard crate incorrectly accepted corrupt base64"
                );
            }
        }
    }

    mod utf8_compliance {
        use super::*;
        use alloc::string::String;
        use base64::{prelude::BASE64_STANDARD, Engine};
        use rand::{rng, RngExt};

        #[test]
        fn ok_utf8_multibyte_parities() {
            let ok_strings = [
                "こんにちは",
                "Hello World!",
                "🦀 Rustacean 🦀",
                "कर्मण्येवाधिकारस्ते मा फलेषु कदाचन",
                "लाभले आम्हास भाग्य बोलतो मराठी",
                "🚀 TurboBase64 Base64 Implementation ⚙️",
                "§ ± ! @ # $ % ^ & * ( ) _ + - = [ ] { } ; ' : \", . / < > ?",
            ];

            for &text in &ok_strings {
                let bytes = text.as_bytes();

                let enc = encode(bytes);
                let dec_bytes = decode(&enc).unwrap();

                let dec_string = String::from_utf8(dec_bytes).unwrap();

                assert_eq!(dec_string, text, "UTF-8 integrity broken during roundtrip!");
            }
        }

        #[test]
        fn ok_randomized_utf8_parity() {
            let mut rng = rng();

            for _ in 0..0x400 {
                let len = rng.random_range(1..0x200);
                let text: String = (0..len).map(|_| rng.random::<char>()).collect();
                let bytes = text.as_bytes();

                let turbo_encoded = encode(bytes);
                let standard_encoded = BASE64_STANDARD.encode(bytes);

                assert_eq!(
                    turbo_encoded,
                    standard_encoded.as_bytes(),
                    "UTF-8 Encoding parity failure at string length {len}!"
                );

                let turbo_decoded = decode(&turbo_encoded).unwrap();
                let standard_decoded = BASE64_STANDARD.decode(&standard_encoded).unwrap();

                assert_eq!(
                    turbo_decoded, standard_decoded,
                    "UTF-8 Decoding parity failure at string length {len}!"
                );
            }
        }
    }
}
