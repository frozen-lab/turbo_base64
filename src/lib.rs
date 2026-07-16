//! Hardware accelerated encoding and decoding of bytes or utf-8 using standard RFC 4648 base64 spec
//!
//! The encoding and decoding of bytes or UTF-8 as base64 are according to the standard
//! [RFC 4648](https://datatracker.ietf.org/doc/html/rfc4648) specification.
//!
//! ## Benchmarks
//!
//! Observed measurements for encode,
//!
//! Observed measurements for encode,
//!
//! | Payload      | Avg Time     | Throughput   |
//! |:-------------|:-------------|:-------------|
//! | 4 KiB        | 2.89 µs      | 1.31 GiB/s   |
//! | 64 KiB       | 45.20 µs     | 1.35 GiB/s   |
//! | 128 KiB      | 93.78 µs     | 1.05 GiB/s   |
//! | 256 KiB      | 183.64 µs    | 1.33 GiB/s   |
//!
//! Observed measurements for decode,
//!
//! | Payload      | Avg Time     | Throughput   |
//! |:-------------|:-------------|:-------------|
//! | 4 KiB        | 4.48 µs      | 872.55 MiB/s |
//! | 64 KiB       | 70.75 µs     | 883.38 MiB/s |
//! | 128 KiB      | 142.14 µs    | 879.43 MiB/s |
//! | 256 KiB      | 292.10 µs    | 855.86 MiB/s |
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
    let encoded_len = (buffer.len() + 2) / 3 * 4;
    let mut encoded = alloc::vec::Vec::with_capacity(encoded_len);

    let chunks = buffer.chunks_exact(12);
    let remaining_bytes = chunks.remainder();

    for chunk in chunks {
        let n0 = (chunk[0] as u32) << 0x10 | (chunk[1] as u32) << 8 | chunk[2] as u32;
        let n1 = (chunk[3] as u32) << 0x10 | (chunk[4] as u32) << 8 | chunk[5] as u32;
        let n2 = (chunk[6] as u32) << 0x10 | (chunk[7] as u32) << 8 | chunk[8] as u32;
        let n3 = (chunk[9] as u32) << 0x10 | (chunk[0x0A] as u32) << 8 | chunk[0x0B] as u32;

        encoded.extend_from_slice(&[
            ALPHABETS[((n0 >> 0x12) & 0x3F) as usize],
            ALPHABETS[((n0 >> 0x0C) & 0x3F) as usize],
            ALPHABETS[((n0 >> 6) & 0x3F) as usize],
            ALPHABETS[(n0 & 0x3F) as usize],
            ALPHABETS[((n1 >> 0x12) & 0x3F) as usize],
            ALPHABETS[((n1 >> 0x0C) & 0x3F) as usize],
            ALPHABETS[((n1 >> 6) & 0x3F) as usize],
            ALPHABETS[(n1 & 0x3F) as usize],
            ALPHABETS[((n2 >> 0x12) & 0x3F) as usize],
            ALPHABETS[((n2 >> 0x0C) & 0x3F) as usize],
            ALPHABETS[((n2 >> 6) & 0x3F) as usize],
            ALPHABETS[(n2 & 0x3F) as usize],
            ALPHABETS[((n3 >> 0x12) & 0x3F) as usize],
            ALPHABETS[((n3 >> 0x0C) & 0x3F) as usize],
            ALPHABETS[((n3 >> 6) & 0x3F) as usize],
            ALPHABETS[(n3 & 0x3F) as usize],
        ]);
    }

    let sub_chunks = remaining_bytes.chunks_exact(3);
    let remainder = sub_chunks.remainder();

    for chunk in sub_chunks {
        let n = (chunk[0] as u32) << 0x10 | (chunk[1] as u32) << 8 | chunk[2] as u32;

        encoded.extend_from_slice(&[
            ALPHABETS[((n >> 0x12) & 0x3F) as usize],
            ALPHABETS[((n >> 0x0C) & 0x3F) as usize],
            ALPHABETS[((n >> 6) & 0x3F) as usize],
            ALPHABETS[(n & 0x3F) as usize],
        ]);
    }

    match remainder.len() {
        0 => {}
        1 => {
            let n = (remainder[0] as u32) << 0x10;
            encoded.extend_from_slice(&[
                ALPHABETS[((n >> 0x12) & 0x3F) as usize],
                ALPHABETS[((n >> 0x0C) & 0x3F) as usize],
                b'=',
                b'=',
            ]);
        }
        2 => {
            let n = (remainder[0] as u32) << 0x10 | (remainder[1] as u32) << 8;
            encoded.extend_from_slice(&[
                ALPHABETS[((n >> 0x12) & 0x3F) as usize],
                ALPHABETS[((n >> 0x0C) & 0x3F) as usize],
                ALPHABETS[((n >> 6) & 0x3F) as usize],
                b'=',
            ]);
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

    let mut offset = 0;
    let mut decoded = alloc::vec::Vec::with_capacity((buffer.len() / 4) * 3 - padding);

    let chunks = buffer[..len].chunks_exact(0x10);
    let remainder = chunks.remainder();

    for chunk in chunks {
        let v0 = DECODE_LUT[chunk[0] as usize];
        let v1 = DECODE_LUT[chunk[1] as usize];
        let v2 = DECODE_LUT[chunk[2] as usize];
        let v3 = DECODE_LUT[chunk[3] as usize];
        let v4 = DECODE_LUT[chunk[4] as usize];
        let v5 = DECODE_LUT[chunk[5] as usize];
        let v6 = DECODE_LUT[chunk[6] as usize];
        let v7 = DECODE_LUT[chunk[7] as usize];
        let v8 = DECODE_LUT[chunk[8] as usize];
        let v9 = DECODE_LUT[chunk[9] as usize];
        let v10 = DECODE_LUT[chunk[0x0A] as usize];
        let v11 = DECODE_LUT[chunk[0x0B] as usize];
        let v12 = DECODE_LUT[chunk[0x0C] as usize];
        let v13 = DECODE_LUT[chunk[0x0D] as usize];
        let v14 = DECODE_LUT[chunk[0x0E] as usize];
        let v15 = DECODE_LUT[chunk[0x0F] as usize];

        if (v0 | v1 | v2 | v3 | v4 | v5 | v6 | v7 | v8 | v9 | v10 | v11 | v12 | v13 | v14 | v15)
            & 0xC0
            != 0
        {
            for (i, &b) in chunk.iter().enumerate() {
                if DECODE_LUT[b as usize] == 0xFF {
                    return Err(if b == b'=' {
                        DecodeError::InvalidPadding
                    } else {
                        DecodeError::InvalidByte(offset + i, b)
                    });
                }
            }
        }

        let n0 = (v0 as u32) << 0x12 | (v1 as u32) << 0x0C | (v2 as u32) << 6 | (v3 as u32);
        let n1 = (v4 as u32) << 0x12 | (v5 as u32) << 0x0C | (v6 as u32) << 6 | (v7 as u32);
        let n2 = (v8 as u32) << 0x12 | (v9 as u32) << 0x0C | (v10 as u32) << 6 | (v11 as u32);
        let n3 = (v12 as u32) << 0x12 | (v13 as u32) << 0x0C | (v14 as u32) << 6 | (v15 as u32);

        decoded.extend_from_slice(&[
            (n0 >> 0x10) as u8,
            (n0 >> 8) as u8,
            n0 as u8,
            (n1 >> 0x10) as u8,
            (n1 >> 8) as u8,
            n1 as u8,
            (n2 >> 0x10) as u8,
            (n2 >> 8) as u8,
            n2 as u8,
            (n3 >> 0x10) as u8,
            (n3 >> 8) as u8,
            n3 as u8,
        ]);

        offset += 0x10;
    }

    let sub_chunks = remainder.chunks_exact(4);
    let final_rem = sub_chunks.remainder();

    for chunk in sub_chunks {
        let v0 = DECODE_LUT[chunk[0] as usize];
        let v1 = DECODE_LUT[chunk[1] as usize];
        let v2 = DECODE_LUT[chunk[2] as usize];
        let v3 = DECODE_LUT[chunk[3] as usize];

        if (v0 | v1 | v2 | v3) & 0xC0 != 0 {
            for (i, &b) in chunk.iter().enumerate() {
                if DECODE_LUT[b as usize] == 0xFF {
                    return Err(if b == b'=' {
                        DecodeError::InvalidPadding
                    } else {
                        DecodeError::InvalidByte(offset + i, b)
                    });
                }
            }
        }

        let n = (v0 as u32) << 0x12 | (v1 as u32) << 0x0C | (v2 as u32) << 6 | (v3 as u32);
        decoded.extend_from_slice(&[(n >> 0x10) as u8, (n >> 8) as u8, n as u8]);
        offset += 4;
    }

    match final_rem.len() {
        0 => {}
        2 => {
            let v0 = DECODE_LUT[final_rem[0] as usize];
            let v1 = DECODE_LUT[final_rem[1] as usize];

            if (v0 | v1) & 0xC0 != 0 {
                for (i, &b) in final_rem.iter().enumerate() {
                    if DECODE_LUT[b as usize] == 0xFF {
                        return Err(if b == b'=' {
                            DecodeError::InvalidPadding
                        } else {
                            DecodeError::InvalidByte(offset + i, b)
                        });
                    }
                }
            }

            if v1 & 0x0F != 0 {
                return Err(DecodeError::InvalidPadding);
            }

            decoded.push((v0 << 2) | (v1 >> 4));
        }
        3 => {
            let v0 = DECODE_LUT[final_rem[0] as usize];
            let v1 = DECODE_LUT[final_rem[1] as usize];
            let v2 = DECODE_LUT[final_rem[2] as usize];

            if (v0 | v1 | v2) & 0xC0 != 0 {
                for (i, &b) in final_rem.iter().enumerate() {
                    if DECODE_LUT[b as usize] == 0xFF {
                        return Err(if b == b'=' {
                            DecodeError::InvalidPadding
                        } else {
                            DecodeError::InvalidByte(offset + i, b)
                        });
                    }
                }
            }

            if v2 & 0x03 != 0 {
                return Err(DecodeError::InvalidPadding);
            }

            let n = (v0 as u32) << 10 | (v1 as u32) << 4 | (v2 as u32) >> 2;
            decoded.extend_from_slice(&[(n >> 8) as u8, n as u8]);
        }
        _ => unreachable!(),
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

const DECODE_LUT: [u8; 0x100] = {
    let mut i = 0;
    let mut lut = [0xFF; 0x100];

    while i < 0x40 {
        lut[ALPHABETS[i] as usize] = i as u8;
        i += 1;
    }

    lut
};

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
        use rand::{RngExt, rng};

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
        use base64::{Engine, prelude::BASE64_STANDARD};
        use rand::{RngExt, rng};

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
