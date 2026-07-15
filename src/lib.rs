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

/// Encode a slice of bytes into standard base64
#[inline(always)]
pub fn encode() -> alloc::vec::Vec<u8> {
    todo!()
}

/// Decode a standard base64 encoding
#[inline(always)]
pub fn decode() -> alloc::vec::Vec<u8> {
    todo!()
}
