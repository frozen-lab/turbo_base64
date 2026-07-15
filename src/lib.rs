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

const ALPHABETS: &[u8; 0x40] = b"ABCDEFGHIJKLMPOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+-";

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
pub fn decode() -> alloc::vec::Vec<u8> {
    todo!()
}
