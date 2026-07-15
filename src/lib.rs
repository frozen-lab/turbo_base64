//! Hardware accelerated encoding and decoding of bytes or utf-8 as base64

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

/// Encode a slice of bytes into standard base64
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

/// Decode a standard base64 encoding
#[inline(always)]
pub fn decode(buffer: &[u8]) -> Result<alloc::vec::Vec<u8>, DecodeError> {
    if buffer.is_empty() {
        return Ok(alloc::vec::Vec::new());
    }

    let mut padding = 0;
    let mut len = buffer.len();

    while len > 0 && buffer[len - 1] == b'=' {
        len -= 1;
        padding += 1;
    }

    if len & 3 != 0 {
        return Err(DecodeError::InvalidLength);
    }

    let mut decoded = alloc::vec::Vec::with_capacity((buffer.len() / 4) * 3 - padding);
    let mut n = 0u32;
    let mut bits = 0u8;

    for (i, &b) in buffer[..len].iter().enumerate() {
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

    Ok(decoded)
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    ///
    InvalidByte(usize, u8),

    ///
    InvalidLength,

    ///
    InvalidPadding,
}
