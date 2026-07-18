//! Hardware accelerated encoding and decoding of bytes or utf-8 using standard RFC 4648 base64
//! specifications
//!
//! The encoding and decoding of bytes or UTF-8 as base64 are according to the standard
//! [RFC 4648](https://datatracker.ietf.org/doc/html/rfc4648) specification.
//!
//! ## `no_std` Support
//!
//! `turbo_base64` crate is fully `#![no_std]` and does not require the Rust standard library.
//!
//! It can be used in embedded, kernel, bootloader, and other constrained environments.
//!
//! ## Benchmarks
//!
//! Observed measurements for encode,
//!
//! | Payload  | Avg Time  | Throughput |
//! |:---------|:----------|:-----------|
//! | 256 KiB  | 37.07 µs  | 6.59 GiB/s |
//! | 512 KiB  | 71.85 µs  | 6.80 GiB/s |
//! | 1 MiB    | 141.61 µs | 6.90 GiB/s |
//! | 2 MiB    | 285.02 µs | 6.85 GiB/s |
//!
//! Observed measurements for decode,
//!
//! | Payload | Avg Time | Throughput |
//! |:--------|:---------|:-----------|
//! | 4 KiB   | 3.29 µs  | 1.16 GiB/s |
//! | 8 KiB   | 6.41 µs  | 1.19 GiB/s |
//! | 16 KiB  | 13.34 µs | 1.14 GiB/s |
//! | 32 KiB  | 26.04 µs | 1.17 GiB/s |
//!
//! ## Example
//!
//! ```
//! use turbo_base64::{encode, decode, encoded_len, decoded_len};
//!
//! let data = b"Hello, Rust!";
//! let mut encoded = vec![0; encoded_len(data.len())];
//!
//! let enc_len = encode(data, &mut encoded).unwrap();
//! assert_eq!(&encoded[..enc_len], b"SGVsbG8sIFJ1c3Qh");
//!
//! let mut decoded = vec![0; decoded_len(encoded.len())];
//! let dec_len = decode(&encoded[..enc_len], &mut decoded).unwrap();
//! assert_eq!(&decoded[..dec_len], data);
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

#[cfg(any(test, doctest))]
extern crate std;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

const ALPHABETS: &[u8; 0x40] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

const DECODE_LUT: [u8; 0x100] = {
    let mut i = 0;
    let mut lut = [0xFF; 0x100];

    while i < 0x40 {
        lut[ALPHABETS[i] as usize] = i as u8;
        i += 1;
    }

    lut
};

// 0: Uninitialized, 1: Fallback, 2: SSSE3, 3: AVX2 4: AVX512
#[cfg(target_arch = "x86_64")]
static CPU_FEATURE: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);

#[inline(always)]
#[cfg(target_arch = "x86_64")]
fn get_cpu_feature() -> u8 {
    let feature = CPU_FEATURE.load(core::sync::atomic::Ordering::Relaxed);
    if feature != 0 {
        return feature;
    }

    let detected = unsafe { detect_features() };
    CPU_FEATURE.store(detected, core::sync::atomic::Ordering::Relaxed);
    detected
}

#[cold]
#[inline(never)]
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn detect_features() -> u8 {
    #[cfg(target_arch = "x86_64")]
    {
        #[cfg(feature = "nightly")]
        {
            let cpuid7 = core::arch::x86_64::__cpuid_count(7, 0);

            let avx512f_bw = (1 << 16) | (1 << 30);
            let vbmi_vbmi2 = (1 << 1) | (1 << 6);

            if (cpuid7.ebx & avx512f_bw) == avx512f_bw && (cpuid7.ecx & vbmi_vbmi2) == vbmi_vbmi2 {
                return 4;
            }
        }

        if (core::arch::x86_64::__cpuid_count(7, 0).ebx & (1 << 5)) != 0 {
            return 3;
        }

        if (core::arch::x86_64::__cpuid(1).ecx & (1 << 9)) != 0 {
            return 2;
        }
    }

    // NOTE: fallback
    1
}

/// Errors that can occur for `encode` or `decode`
///
/// ## Example
///
/// ```
/// use turbo_base64::{decode, Error};
/// let mut buf = [0u8; 8];
///
/// assert_eq!(decode(b"Z", &mut buf), Err(Error::InvalidLength));
/// assert_eq!(decode(b"Z=9v", &mut buf), Err(Error::InvalidPadding));
/// assert_eq!(decode(b"Zg =", &mut buf), Err(Error::InvalidByte(2, b' ')));
/// assert_eq!(decode(b"Zm9vYmFy", &mut []), Err(Error::BufferTooSmall));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Encountered a character outside the standard base64 alphabet
    ///
    /// ## Example
    ///
    /// ```
    /// use turbo_base64::{decode, Error};
    /// let mut buf = [0u8; 8];
    /// assert_eq!(decode(b"Zg =", &mut buf), Err(Error::InvalidByte(2, b' ')));
    /// ```
    InvalidByte(usize, u8),

    /// The length of the input buffer was not a multiple of `4`
    ///
    /// ## Example
    ///
    /// ```
    /// use turbo_base64::{decode, Error};
    /// let mut buf = [0u8; 8];
    /// assert_eq!(decode(b"Z", &mut buf), Err(Error::InvalidLength));
    /// ```
    InvalidLength,

    /// Padding `=` characters were misplaced or exceeded the maximum of `2`
    ///
    /// ## Example
    ///
    /// ```
    /// use turbo_base64::{decode, Error};
    /// let mut buf = [0u8; 8];
    /// assert_eq!(decode(b"Z=9v", &mut buf), Err(Error::InvalidPadding));
    /// ```
    InvalidPadding,

    /// The provided output buffer is too small to hold the decoded data
    ///
    /// ## Example
    ///
    /// ```
    /// use turbo_base64::{encode, Error};
    /// let mut buf = [0u8; 2];
    /// assert_eq!(encode(b"foobar", &mut buf), Err(Error::BufferTooSmall));
    /// ```
    BufferTooSmall,
}

/// Calculates the exact required output buffer length for encoding
///
/// ## Example
///
/// ```
/// use turbo_base64::encoded_len;
///
/// assert_eq!(encoded_len(5), 8);
/// assert_eq!(encoded_len(0), 0);
/// ```
#[inline(always)]
pub const fn encoded_len(input_len: usize) -> usize {
    (input_len + 2) / 3 * 4
}

/// Calculates the maximum expected output buffer length for decoding
///
/// ## Example
///
/// ```
/// use turbo_base64::decoded_len;
///
/// assert_eq!(decoded_len(8), 6);
/// assert_eq!(decoded_len(0), 0);
/// ```
#[inline(always)]
pub const fn decoded_len(input_len: usize) -> usize {
    (input_len / 4) * 3
}

/// Encodes a slice of bytes into standard RFC 4648 base64
///
/// ## Example
///
/// ```
/// use turbo_base64::{encode, encoded_len};
///
/// const INPUT: &[u8] = b"fooba";
/// let mut buf = [0u8; encoded_len(INPUT.len())];
///
/// let len = encode(INPUT, &mut buf).unwrap();
/// assert_eq!(&buf[..len], b"Zm9vYmE=");
///
/// let len_empty = encode(b"", &mut buf).unwrap();
/// assert_eq!(len_empty, 0);
/// ```
#[inline(always)]
#[allow(unused_mut)]
pub fn encode(buffer: &[u8], output: &mut [u8]) -> Result<usize, Error> {
    let encoded_len = (buffer.len() + 2) / 3 * 4;

    if output.len() < encoded_len {
        return Err(Error::BufferTooSmall);
    }

    let mut in_idx = 0;
    let mut out_idx = 0;

    #[cfg(target_arch = "x86_64")]
    unsafe {
        match get_cpu_feature() {
            #[cfg(feature = "nightly")]
            4 => {
                let (proc_in, proc_out) = encode_chunk_avx512(buffer, output);
                in_idx += proc_in;
                out_idx += proc_out;
            }
            3 => {
                let (proc_in, proc_out) = encode_chunk_avx2(buffer, output);
                in_idx += proc_in;
                out_idx += proc_out;
            }
            2 => {
                let (proc_in, proc_out) = encode_chunk_ssse3(buffer, output);
                in_idx += proc_in;
                out_idx += proc_out;
            }
            _ => {}
        }
    }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        let (proc_in, proc_out) = encode_chunk_neon(buffer, output);
        in_idx += proc_in;
        out_idx += proc_out;
    }

    let chunks = buffer[in_idx..].chunks_exact(0x0C);
    let remaining_bytes = chunks.remainder();

    for chunk in chunks {
        let n0 = (chunk[0] as u32) << 0x10 | (chunk[1] as u32) << 8 | chunk[2] as u32;
        let n1 = (chunk[3] as u32) << 0x10 | (chunk[4] as u32) << 8 | chunk[5] as u32;
        let n2 = (chunk[6] as u32) << 0x10 | (chunk[7] as u32) << 8 | chunk[8] as u32;
        let n3 = (chunk[9] as u32) << 0x10 | (chunk[0x0A] as u32) << 8 | chunk[0x0B] as u32;

        output[out_idx..out_idx + 16].copy_from_slice(&[
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

        out_idx += 0x10;
    }

    let sub_chunks = remaining_bytes.chunks_exact(3);
    let remainder = sub_chunks.remainder();

    for chunk in sub_chunks {
        let n = (chunk[0] as u32) << 0x10 | (chunk[1] as u32) << 8 | chunk[2] as u32;

        output[out_idx..out_idx + 4].copy_from_slice(&[
            ALPHABETS[((n >> 0x12) & 0x3F) as usize],
            ALPHABETS[((n >> 0x0C) & 0x3F) as usize],
            ALPHABETS[((n >> 6) & 0x3F) as usize],
            ALPHABETS[(n & 0x3F) as usize],
        ]);

        out_idx += 4;
    }

    match remainder.len() {
        0 => {}
        1 => {
            let n = (remainder[0] as u32) << 0x10;
            output[out_idx..out_idx + 4].copy_from_slice(&[
                ALPHABETS[((n >> 0x12) & 0x3F) as usize],
                ALPHABETS[((n >> 0x0C) & 0x3F) as usize],
                b'=',
                b'=',
            ]);
        }
        2 => {
            let n = (remainder[0] as u32) << 0x10 | (remainder[1] as u32) << 8;
            output[out_idx..out_idx + 4].copy_from_slice(&[
                ALPHABETS[((n >> 0x12) & 0x3F) as usize],
                ALPHABETS[((n >> 0x0C) & 0x3F) as usize],
                ALPHABETS[((n >> 6) & 0x3F) as usize],
                b'=',
            ]);
        }
        _ => unreachable!(),
    }

    Ok(encoded_len)
}

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_op_in_unsafe_fn)]
#[target_feature(enable = "ssse3")]
unsafe fn encode_chunk_ssse3(buffer: &[u8], output: &mut [u8]) -> (usize, usize) {
    let mut in_idx = 0;
    let mut out_idx = 0;

    let shuf = _mm_setr_epi8(1, 0, 2, 1, 4, 3, 5, 4, 7, 6, 8, 7, 0x0A, 9, 0x0B, 0x0A);

    let mask_t0 = _mm_set1_epi32(0x0FC0FC00_u32 as i32);
    let mask_t1 = _mm_set1_epi32(0x04000040_u32 as i32);
    let mask_t2 = _mm_set1_epi32(0x003F03F0_u32 as i32);
    let mask_t3 = _mm_set1_epi32(0x01000010_u32 as i32);

    let cmp_25 = _mm_set1_epi8(0x19);
    let cmp_51 = _mm_set1_epi8(0x33);
    let cmp_61 = _mm_set1_epi8(0x3D);
    let cmp_62 = _mm_set1_epi8(0x3E);

    let add_41 = _mm_set1_epi8(0x41);
    let add_25 = _mm_set1_epi8(6);
    let add_51 = _mm_set1_epi8(-0x4B);
    let add_61 = _mm_set1_epi8(-0x0F);
    let add_62 = _mm_set1_epi8(3);

    while in_idx + 0x1C <= buffer.len() {
        let in1 = _mm_loadu_si128(buffer.as_ptr().add(in_idx) as *const __m128i);
        let in2 = _mm_loadu_si128(buffer.as_ptr().add(in_idx + 0x0C) as *const __m128i);

        let v1 = _mm_shuffle_epi8(in1, shuf);
        let v2 = _mm_shuffle_epi8(in2, shuf);

        let t0_1 = _mm_and_si128(v1, mask_t0);
        let t0_2 = _mm_and_si128(v2, mask_t0);
        let t1_1 = _mm_mulhi_epu16(t0_1, mask_t1);
        let t1_2 = _mm_mulhi_epu16(t0_2, mask_t1);

        let t2_1 = _mm_and_si128(v1, mask_t2);
        let t2_2 = _mm_and_si128(v2, mask_t2);
        let t3_1 = _mm_mullo_epi16(t2_1, mask_t3);
        let t3_2 = _mm_mullo_epi16(t2_2, mask_t3);

        let i1 = _mm_or_si128(t1_1, t3_1);
        let i2 = _mm_or_si128(t1_2, t3_2);

        let m25_1 = _mm_cmpgt_epi8(i1, cmp_25);
        let m25_2 = _mm_cmpgt_epi8(i2, cmp_25);
        let m51_1 = _mm_cmpgt_epi8(i1, cmp_51);
        let m51_2 = _mm_cmpgt_epi8(i2, cmp_51);
        let m61_1 = _mm_cmpgt_epi8(i1, cmp_61);
        let m61_2 = _mm_cmpgt_epi8(i2, cmp_61);
        let m62_1 = _mm_cmpgt_epi8(i1, cmp_62);
        let m62_2 = _mm_cmpgt_epi8(i2, cmp_62);

        let mut a1 = _mm_add_epi8(i1, add_41);
        let mut a2 = _mm_add_epi8(i2, add_41);

        a1 = _mm_add_epi8(a1, _mm_and_si128(m25_1, add_25));
        a2 = _mm_add_epi8(a2, _mm_and_si128(m25_2, add_25));
        a1 = _mm_add_epi8(a1, _mm_and_si128(m51_1, add_51));
        a2 = _mm_add_epi8(a2, _mm_and_si128(m51_2, add_51));
        a1 = _mm_add_epi8(a1, _mm_and_si128(m61_1, add_61));
        a2 = _mm_add_epi8(a2, _mm_and_si128(m61_2, add_61));
        a1 = _mm_add_epi8(a1, _mm_and_si128(m62_1, add_62));
        a2 = _mm_add_epi8(a2, _mm_and_si128(m62_2, add_62));

        _mm_storeu_si128(output.as_mut_ptr().add(out_idx) as *mut __m128i, a1);
        _mm_storeu_si128(output.as_mut_ptr().add(out_idx + 0x10) as *mut __m128i, a2);

        in_idx += 0x18;
        out_idx += 0x20;
    }

    if in_idx + 0x10 <= buffer.len() {
        let in_data = _mm_loadu_si128(buffer.as_ptr().add(in_idx) as *const __m128i);
        let v = _mm_shuffle_epi8(in_data, shuf);

        let t0 = _mm_and_si128(v, mask_t0);
        let t1 = _mm_mulhi_epu16(t0, mask_t1);
        let t2 = _mm_and_si128(v, mask_t2);
        let t3 = _mm_mullo_epi16(t2, mask_t3);

        let i = _mm_or_si128(t1, t3);

        let m25 = _mm_cmpgt_epi8(i, cmp_25);
        let m51 = _mm_cmpgt_epi8(i, cmp_51);
        let m61 = _mm_cmpgt_epi8(i, cmp_61);
        let m62 = _mm_cmpgt_epi8(i, cmp_62);

        let mut a = _mm_add_epi8(i, add_41);
        a = _mm_add_epi8(a, _mm_and_si128(m25, add_25));
        a = _mm_add_epi8(a, _mm_and_si128(m51, add_51));
        a = _mm_add_epi8(a, _mm_and_si128(m61, add_61));
        a = _mm_add_epi8(a, _mm_and_si128(m62, add_62));

        _mm_storeu_si128(output.as_mut_ptr().add(out_idx) as *mut __m128i, a);

        in_idx += 0x0C;
        out_idx += 0x10;
    }

    (in_idx, out_idx)
}

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_op_in_unsafe_fn)]
#[target_feature(enable = "avx2")]
unsafe fn encode_chunk_avx2(buffer: &[u8], output: &mut [u8]) -> (usize, usize) {
    let mut in_idx = 0;
    let mut out_idx = 0;

    let perm_idx = _mm256_setr_epi32(0, 1, 2, 0, 3, 4, 5, 0);

    let shuf = _mm256_setr_epi8(
        1, 0, 2, 1, 4, 3, 5, 4, 7, 6, 8, 7, 0x0A, 9, 0x0B, 0x0A, 1, 0, 2, 1, 4, 3, 5, 4, 7, 6, 8,
        7, 0x0A, 9, 0x0B, 0x0A,
    );

    let mask_t0 = _mm256_set1_epi32(0x0FC0FC00_u32 as i32);
    let mask_t1 = _mm256_set1_epi32(0x04000040_u32 as i32);
    let mask_t2 = _mm256_set1_epi32(0x003F03F0_u32 as i32);
    let mask_t3 = _mm256_set1_epi32(0x01000010_u32 as i32);

    let cmp_25 = _mm256_set1_epi8(0x19);
    let cmp_51 = _mm256_set1_epi8(0x33);
    let cmp_61 = _mm256_set1_epi8(0x3D);
    let cmp_62 = _mm256_set1_epi8(0x3E);

    let add_41 = _mm256_set1_epi8(0x41);
    let add_25 = _mm256_set1_epi8(6);
    let add_51 = _mm256_set1_epi8(-0x4B);
    let add_61 = _mm256_set1_epi8(-0x0F);
    let add_62 = _mm256_set1_epi8(3);

    while in_idx + 0x38 <= buffer.len() {
        let in1 = _mm256_loadu_si256(buffer.as_ptr().add(in_idx) as *const __m256i);
        let in2 = _mm256_loadu_si256(buffer.as_ptr().add(in_idx + 0x18) as *const __m256i);

        let p1 = _mm256_permutevar8x32_epi32(in1, perm_idx);
        let p2 = _mm256_permutevar8x32_epi32(in2, perm_idx);

        let v1 = _mm256_shuffle_epi8(p1, shuf);
        let v2 = _mm256_shuffle_epi8(p2, shuf);

        let t0_1 = _mm256_and_si256(v1, mask_t0);
        let t0_2 = _mm256_and_si256(v2, mask_t0);
        let t1_1 = _mm256_mulhi_epu16(t0_1, mask_t1);
        let t1_2 = _mm256_mulhi_epu16(t0_2, mask_t1);

        let t2_1 = _mm256_and_si256(v1, mask_t2);
        let t2_2 = _mm256_and_si256(v2, mask_t2);
        let t3_1 = _mm256_mullo_epi16(t2_1, mask_t3);
        let t3_2 = _mm256_mullo_epi16(t2_2, mask_t3);

        let i1 = _mm256_or_si256(t1_1, t3_1);
        let i2 = _mm256_or_si256(t1_2, t3_2);

        let m25_1 = _mm256_cmpgt_epi8(i1, cmp_25);
        let m25_2 = _mm256_cmpgt_epi8(i2, cmp_25);
        let m51_1 = _mm256_cmpgt_epi8(i1, cmp_51);
        let m51_2 = _mm256_cmpgt_epi8(i2, cmp_51);
        let m61_1 = _mm256_cmpgt_epi8(i1, cmp_61);
        let m61_2 = _mm256_cmpgt_epi8(i2, cmp_61);
        let m62_1 = _mm256_cmpgt_epi8(i1, cmp_62);
        let m62_2 = _mm256_cmpgt_epi8(i2, cmp_62);

        let mut a1 = _mm256_add_epi8(i1, add_41);
        let mut a2 = _mm256_add_epi8(i2, add_41);

        a1 = _mm256_add_epi8(a1, _mm256_and_si256(m25_1, add_25));
        a2 = _mm256_add_epi8(a2, _mm256_and_si256(m25_2, add_25));
        a1 = _mm256_add_epi8(a1, _mm256_and_si256(m51_1, add_51));
        a2 = _mm256_add_epi8(a2, _mm256_and_si256(m51_2, add_51));
        a1 = _mm256_add_epi8(a1, _mm256_and_si256(m61_1, add_61));
        a2 = _mm256_add_epi8(a2, _mm256_and_si256(m61_2, add_61));
        a1 = _mm256_add_epi8(a1, _mm256_and_si256(m62_1, add_62));
        a2 = _mm256_add_epi8(a2, _mm256_and_si256(m62_2, add_62));

        _mm256_storeu_si256(output.as_mut_ptr().add(out_idx) as *mut __m256i, a1);
        _mm256_storeu_si256(output.as_mut_ptr().add(out_idx + 0x20) as *mut __m256i, a2);

        in_idx += 0x30;
        out_idx += 0x40;
    }

    if in_idx + 0x20 <= buffer.len() {
        let in_data = _mm256_loadu_si256(buffer.as_ptr().add(in_idx) as *const __m256i);
        let p = _mm256_permutevar8x32_epi32(in_data, perm_idx);
        let v = _mm256_shuffle_epi8(p, shuf);

        let t0 = _mm256_and_si256(v, mask_t0);
        let t1 = _mm256_mulhi_epu16(t0, mask_t1);
        let t2 = _mm256_and_si256(v, mask_t2);
        let t3 = _mm256_mullo_epi16(t2, mask_t3);

        let i = _mm256_or_si256(t1, t3);

        let m25 = _mm256_cmpgt_epi8(i, cmp_25);
        let m51 = _mm256_cmpgt_epi8(i, cmp_51);
        let m61 = _mm256_cmpgt_epi8(i, cmp_61);
        let m62 = _mm256_cmpgt_epi8(i, cmp_62);

        let mut a = _mm256_add_epi8(i, add_41);
        a = _mm256_add_epi8(a, _mm256_and_si256(m25, add_25));
        a = _mm256_add_epi8(a, _mm256_and_si256(m51, add_51));
        a = _mm256_add_epi8(a, _mm256_and_si256(m61, add_61));
        a = _mm256_add_epi8(a, _mm256_and_si256(m62, add_62));

        _mm256_storeu_si256(output.as_mut_ptr().add(out_idx) as *mut __m256i, a);

        in_idx += 0x18;
        out_idx += 0x20;
    }

    (in_idx, out_idx)
}

#[allow(unsafe_op_in_unsafe_fn)]
#[cfg(all(target_arch = "x86_64", feature = "nightly"))]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi,avx512vbmi2")]
unsafe fn encode_chunk_avx512(buffer: &[u8], output: &mut [u8]) -> (usize, usize) {
    let mut in_idx = 0;
    let mut out_idx = 0;

    let alphabet = _mm512_loadu_si512(ALPHABETS.as_ptr() as *const __m512i);
    let expand_bswap = _mm512_set_epi8(
        42, 43, 44, 45, 46, 47, 0, 0, 36, 37, 38, 39, 40, 41, 0, 0, 30, 31, 32, 33, 34, 35, 0, 0,
        24, 25, 26, 27, 28, 29, 0, 0, 18, 19, 20, 21, 22, 23, 0, 0, 12, 13, 14, 15, 16, 17, 0, 0,
        6, 7, 8, 9, 10, 11, 0, 0, 0, 1, 2, 3, 4, 5, 0, 0,
    );

    let shift_ctrl = _mm512_set1_epi64(0x10161C22282E343A_u64 as i64);
    let mask_3f = _mm512_set1_epi8(0x3F);

    while in_idx + 0x30 <= buffer.len() {
        let in_data = if in_idx + 0x40 <= buffer.len() {
            _mm512_loadu_si512(buffer.as_ptr().add(in_idx) as *const __m512i)
        } else {
            _mm512_maskz_loadu_epi8(0xFFFFFFFFFFFF, buffer.as_ptr().add(in_idx) as *const i8)
        };

        let expanded = _mm512_permutexvar_epi8(expand_bswap, in_data);

        let shifted = _mm512_multishift_epi64_epi8(shift_ctrl, expanded);
        let indices = _mm512_and_si512(shifted, mask_3f);

        let encoded = _mm512_permutexvar_epi8(indices, alphabet);

        _mm512_storeu_si512(output.as_mut_ptr().add(out_idx) as *mut __m512i, encoded);

        in_idx += 0x30;
        out_idx += 0x40;
    }

    (in_idx, out_idx)
}

#[cfg(target_arch = "aarch64")]
#[allow(unsafe_op_in_unsafe_fn)]
#[target_feature(enable = "neon")]
unsafe fn encode_chunk_neon(buffer: &[u8], output: &mut [u8]) -> (usize, usize) {
    use core::arch::aarch64::*;

    let mut in_idx = 0;
    let mut out_idx = 0;

    let shuf = vld1q_u8([0, 0, 1, 2, 3, 3, 4, 5, 6, 6, 7, 8, 9, 9, 10, 11].as_ptr());
    let shift_l = vld1q_s8([-2i8, 4, 2, 0, -2, 4, 2, 0, -2, 4, 2, 0, -2, 4, 2, 0].as_ptr());

    let shift_r =
        vld1q_s8([-8i8, -4, -6, -8, -8, -4, -6, -8, -8, -4, -6, -8, -8, -4, -6, -8].as_ptr());

    let mask_3f = vdupq_n_u8(0x3F);

    let alpha = uint8x16x4_t(
        vld1q_u8(b"ABCDEFGHIJKLMNOP".as_ptr()),
        vld1q_u8(b"QRSTUVWXYZabcdef".as_ptr()),
        vld1q_u8(b"ghijklmnopqrstuv".as_ptr()),
        vld1q_u8(b"wxyz0123456789+/".as_ptr()),
    );

    while in_idx + 16 <= buffer.len() {
        let input = vld1q_u8(buffer.as_ptr().add(in_idx));

        let v = vqtbl1q_u8(input, shuf);
        let v_next = vextq_u8(v, v, 1);

        let part_l = vshlq_u8(v, shift_l);
        let part_r = vshlq_u8(v_next, shift_r);

        let indices = vandq_u8(vorrq_u8(part_l, part_r), mask_3f);
        let encoded = vqtbl4q_u8(alpha, indices);

        vst1q_u8(output.as_mut_ptr().add(out_idx), encoded);

        in_idx += 12;
        out_idx += 16;
    }

    (in_idx, out_idx)
}

/// Decodes a standard RFC 4648 base64 encoded byte slice
///
/// ## Example
///
/// ```
/// use turbo_base64::{decode, decoded_len, Error};
///
/// const ENCODED: &[u8] = b"Zm9vYmFy";
///
/// let mut buf = [0u8; decoded_len(ENCODED.len())];
/// let len = decode(ENCODED, &mut buf).unwrap();
/// assert_eq!(&buf[..len], b"foobar");
///
/// let mut buf2 = [0u8; decoded_len(4)];
/// let len2 = decode(b"Zm8=", &mut buf2).unwrap();
/// assert_eq!(&buf2[..len2], b"fo");
///
/// assert_eq!(decode(b"Zg =", &mut buf), Err(Error::InvalidByte(2, b' ')));
/// assert_eq!(decode(b"Z", &mut buf), Err(Error::InvalidLength));
/// ```
#[inline(always)]
pub fn decode(buffer: &[u8], output: &mut [u8]) -> Result<usize, Error> {
    if buffer.is_empty() {
        return Ok(0);
    }

    if buffer.len() & 3 != 0 {
        return Err(Error::InvalidLength);
    }

    let mut padding = 0;
    let mut len = buffer.len();

    while len > 0 && buffer[len - 1] == b'=' {
        len -= 1;
        padding += 1;
    }

    if padding > 2 {
        return Err(Error::InvalidPadding);
    }

    let expected_len = (buffer.len() / 4) * 3 - padding;
    if output.len() < expected_len {
        return Err(Error::BufferTooSmall);
    }

    let mut offset = 0;
    let mut out_idx = 0;

    #[cfg(target_arch = "x86_64")]
    unsafe {
        match get_cpu_feature() {
            #[cfg(feature = "nightly")]
            4 => {
                let (proc_in, proc_out) = decode_chunk_avx512(&buffer[..len], output);
                offset += proc_in;
                out_idx += proc_out;
            }
            2 => {
                let (proc_in, proc_out) = decode_chunk_ssse3(&buffer[..len], output);
                offset += proc_in;
                out_idx += proc_out;
            }
            3 => {
                let (proc_in, proc_out) = decode_chunk_avx2(&buffer[..len], output);
                offset += proc_in;
                out_idx += proc_out;
            }
            _ => {}
        }
    }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        let (proc_in, proc_out) = decode_chunk_neon(&buffer[..len], output);
        offset += proc_in;
        out_idx += proc_out;
    }

    let chunks = buffer[offset..len].chunks_exact(0x10);
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
                        Error::InvalidPadding
                    } else {
                        Error::InvalidByte(offset + i, b)
                    });
                }
            }
        }

        let n0 = (v0 as u32) << 0x12 | (v1 as u32) << 0x0C | (v2 as u32) << 6 | (v3 as u32);
        let n1 = (v4 as u32) << 0x12 | (v5 as u32) << 0x0C | (v6 as u32) << 6 | (v7 as u32);
        let n2 = (v8 as u32) << 0x12 | (v9 as u32) << 0x0C | (v10 as u32) << 6 | (v11 as u32);
        let n3 = (v12 as u32) << 0x12 | (v13 as u32) << 0x0C | (v14 as u32) << 6 | (v15 as u32);

        output[out_idx..out_idx + 12].copy_from_slice(&[
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
        out_idx += 0x0C;
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
                        Error::InvalidPadding
                    } else {
                        Error::InvalidByte(offset + i, b)
                    });
                }
            }
        }

        let n = (v0 as u32) << 0x12 | (v1 as u32) << 0x0C | (v2 as u32) << 6 | (v3 as u32);
        output[out_idx..out_idx + 3].copy_from_slice(&[(n >> 0x10) as u8, (n >> 8) as u8, n as u8]);

        offset += 4;
        out_idx += 3;
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
                            Error::InvalidPadding
                        } else {
                            Error::InvalidByte(offset + i, b)
                        });
                    }
                }
            }

            if v1 & 0x0F != 0 {
                return Err(Error::InvalidPadding);
            }

            output[out_idx] = (v0 << 2) | (v1 >> 4);
        }
        3 => {
            let v0 = DECODE_LUT[final_rem[0] as usize];
            let v1 = DECODE_LUT[final_rem[1] as usize];
            let v2 = DECODE_LUT[final_rem[2] as usize];

            if (v0 | v1 | v2) & 0xC0 != 0 {
                for (i, &b) in final_rem.iter().enumerate() {
                    if DECODE_LUT[b as usize] == 0xFF {
                        return Err(if b == b'=' {
                            Error::InvalidPadding
                        } else {
                            Error::InvalidByte(offset + i, b)
                        });
                    }
                }
            }

            if v2 & 0x03 != 0 {
                return Err(Error::InvalidPadding);
            }

            let n = (v0 as u32) << 10 | (v1 as u32) << 4 | (v2 as u32) >> 2;
            output[out_idx..out_idx + 2].copy_from_slice(&[(n >> 8) as u8, n as u8]);
        }
        _ => unreachable!(),
    }

    Ok(expected_len)
}

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_op_in_unsafe_fn)]
#[target_feature(enable = "ssse3")]
unsafe fn decode_chunk_ssse3(buffer: &[u8], output: &mut [u8]) -> (usize, usize) {
    let mut in_idx = 0;
    let mut out_idx = 0;

    let lut_lo = _mm_setr_epi8(
        0x15, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x13, 0x1A, 0x1B, 0x1B, 0x1B,
        0x1A,
    );

    let lut_hi = _mm_setr_epi8(
        0x10, 0x10, 0x01, 0x02, 0x04, 0x08, 0x04, 0x08, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
        0x10,
    );

    let lut_roll = _mm_setr_epi8(0, 0x10, 0x13, 4, -65, -65, -71, -71, 0, 0, 0, 0, 0, 0, 0, 0);

    let pack_shuf = _mm_setr_epi8(2, 1, 0, 6, 5, 4, 0x0A, 9, 8, 0x0E, 0x0D, 0x0C, -1, -1, -1, -1);

    while in_idx + 0x10 <= buffer.len() {
        let in_val = _mm_loadu_si128(buffer.as_ptr().add(in_idx) as *const __m128i);

        let hi_nibbles = _mm_and_si128(_mm_srli_epi32(in_val, 4), _mm_set1_epi8(0x0F));
        let lo_nibbles = _mm_and_si128(in_val, _mm_set1_epi8(0x0F));

        let hi = _mm_shuffle_epi8(lut_hi, hi_nibbles);
        let lo = _mm_shuffle_epi8(lut_lo, lo_nibbles);

        if _mm_movemask_epi8(_mm_cmpeq_epi8(_mm_and_si128(hi, lo), _mm_setzero_si128())) != -1 {
            break;
        }

        let eq_2f = _mm_cmpeq_epi8(in_val, _mm_set1_epi8(0x2F));
        let shift = _mm_shuffle_epi8(lut_roll, _mm_add_epi8(hi_nibbles, eq_2f));
        let decoded = _mm_add_epi8(in_val, shift);

        let merged = _mm_maddubs_epi16(decoded, _mm_set1_epi32(0x01400140));
        let packed = _mm_madd_epi16(merged, _mm_set1_epi32(0x00011000));
        let shuf = _mm_shuffle_epi8(packed, pack_shuf);

        let shuf_ptr: *const __m128i = &shuf;
        core::ptr::copy_nonoverlapping(
            shuf_ptr as *const u8,
            output.as_mut_ptr().add(out_idx),
            0x0C,
        );

        in_idx += 0x10;
        out_idx += 0x0C;
    }

    (in_idx, out_idx)
}

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_op_in_unsafe_fn)]
#[target_feature(enable = "avx2")]
unsafe fn decode_chunk_avx2(buffer: &[u8], output: &mut [u8]) -> (usize, usize) {
    let mut in_idx = 0;
    let mut out_idx = 0;

    let lut_lo = _mm256_setr_epi8(
        0x15, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x13, 0x1A, 0x1B, 0x1B, 0x1B,
        0x1A, 0x15, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x13, 0x1A, 0x1B, 0x1B,
        0x1B, 0x1A,
    );

    let lut_hi = _mm256_setr_epi8(
        0x10, 0x10, 0x01, 0x02, 0x04, 0x08, 0x04, 0x08, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
        0x10, 0x10, 0x10, 0x01, 0x02, 0x04, 0x08, 0x04, 0x08, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
        0x10, 0x10,
    );

    let lut_roll = _mm256_setr_epi8(
        0, 0x10, 0x13, 4, -65, -65, -71, -71, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10, 0x13, 4, -65, -65,
        -71, -71, 0, 0, 0, 0, 0, 0, 0, 0,
    );

    let pack_shuf = _mm256_setr_epi8(
        2, 1, 0, 6, 5, 4, 0x0A, 9, 8, 0x0E, 0x0D, 0x0C, -1, -1, -1, -1, 2, 1, 0, 6, 5, 4, 0x0A, 9,
        8, 0x0E, 0x0D, 0x0C, -1, -1, -1, -1,
    );

    while in_idx + 0x20 <= buffer.len() {
        let in_val = _mm256_loadu_si256(buffer.as_ptr().add(in_idx) as *const __m256i);

        let hi_nibbles = _mm256_and_si256(_mm256_srli_epi32(in_val, 4), _mm256_set1_epi8(0x0F));
        let lo_nibbles = _mm256_and_si256(in_val, _mm256_set1_epi8(0x0F));

        let hi = _mm256_shuffle_epi8(lut_hi, hi_nibbles);
        let lo = _mm256_shuffle_epi8(lut_lo, lo_nibbles);

        if _mm256_movemask_epi8(_mm256_cmpeq_epi8(_mm256_and_si256(hi, lo), _mm256_setzero_si256()))
            != -1
        {
            break;
        }

        let eq_2f = _mm256_cmpeq_epi8(in_val, _mm256_set1_epi8(0x2F));
        let shift = _mm256_shuffle_epi8(lut_roll, _mm256_add_epi8(hi_nibbles, eq_2f));
        let decoded = _mm256_add_epi8(in_val, shift);

        let merged = _mm256_maddubs_epi16(decoded, _mm256_set1_epi32(0x01400140));
        let packed = _mm256_madd_epi16(merged, _mm256_set1_epi32(0x00011000));
        let shuf = _mm256_shuffle_epi8(packed, pack_shuf);

        let shuf_ptr: *const __m256i = &shuf;
        let shuf_u8 = shuf_ptr as *const u8;
        core::ptr::copy_nonoverlapping(shuf_u8, output.as_mut_ptr().add(out_idx), 0x0C);
        core::ptr::copy_nonoverlapping(
            shuf_u8.add(0x10),
            output.as_mut_ptr().add(out_idx + 0x0C),
            0x0C,
        );

        in_idx += 0x20;
        out_idx += 0x18;
    }

    (in_idx, out_idx)
}

#[allow(unsafe_op_in_unsafe_fn)]
#[cfg(all(target_arch = "x86_64", feature = "nightly"))]
#[target_feature(enable = "avx512f,avx512bw,avx512vbmi,avx512vbmi2")]
unsafe fn decode_chunk_avx512(buffer: &[u8], output: &mut [u8]) -> (usize, usize) {
    let mut in_idx = 0;
    let mut out_idx = 0;

    let lut_lo_128 = _mm_setr_epi8(
        0x15, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x13, 0x1A, 0x1B, 0x1B, 0x1B,
        0x1A,
    );
    let lut_lo = _mm512_broadcast_i32x4(lut_lo_128);

    let lut_hi_128 = _mm_setr_epi8(
        0x10, 0x10, 0x01, 0x02, 0x04, 0x08, 0x04, 0x08, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
        0x10,
    );
    let lut_hi = _mm512_broadcast_i32x4(lut_hi_128);

    let lut_roll_128 = _mm_setr_epi8(0, 0x10, 0x13, 4, -65, -65, -71, -71, 0, 0, 0, 0, 0, 0, 0, 0);
    let lut_roll = _mm512_broadcast_i32x4(lut_roll_128);

    let pack_shuf_128 =
        _mm_setr_epi8(2, 1, 0, 6, 5, 4, 0x0A, 9, 8, 0x0E, 0x0D, 0x0C, -1, -1, -1, -1);
    let pack_shuf = _mm512_broadcast_i32x4(pack_shuf_128);

    let mask_0f = _mm512_set1_epi8(0x0F);
    let mask_2f = _mm512_set1_epi8(0x2F);
    let madd_1 = _mm512_set1_epi32(0x01400140);
    let madd_2 = _mm512_set1_epi32(0x00011000);

    while in_idx + 0x40 <= buffer.len() {
        let in_val = _mm512_loadu_si512(buffer.as_ptr().add(in_idx) as *const __m512i);

        let hi_nibbles = _mm512_and_si512(_mm512_srli_epi32(in_val, 4), mask_0f);
        let lo_nibbles = _mm512_and_si512(in_val, mask_0f);

        let hi = _mm512_shuffle_epi8(lut_hi, hi_nibbles);
        let lo = _mm512_shuffle_epi8(lut_lo, lo_nibbles);

        if _mm512_cmpeq_epi8_mask(_mm512_and_si512(hi, lo), _mm512_setzero_si512()) != !0 {
            break;
        }

        let eq_2f_mask = _mm512_cmpeq_epi8_mask(in_val, mask_2f);
        let add_val = _mm512_maskz_set1_epi8(eq_2f_mask, -1);
        let shift = _mm512_shuffle_epi8(lut_roll, _mm512_add_epi8(hi_nibbles, add_val));
        let decoded = _mm512_add_epi8(in_val, shift);

        let merged = _mm512_maddubs_epi16(decoded, madd_1);
        let packed = _mm512_madd_epi16(merged, madd_2);
        let shuf = _mm512_shuffle_epi8(packed, pack_shuf);

        let shuf_ptr: *const __m512i = &shuf;
        let shuf_u8 = shuf_ptr as *const u8;
        core::ptr::copy_nonoverlapping(shuf_u8, output.as_mut_ptr().add(out_idx), 0x0C);
        core::ptr::copy_nonoverlapping(
            shuf_u8.add(0x10),
            output.as_mut_ptr().add(out_idx + 0x0C),
            0x0C,
        );
        core::ptr::copy_nonoverlapping(
            shuf_u8.add(0x20),
            output.as_mut_ptr().add(out_idx + 0x18),
            0x0C,
        );
        core::ptr::copy_nonoverlapping(
            shuf_u8.add(0x30),
            output.as_mut_ptr().add(out_idx + 0x24),
            0x0C,
        );

        in_idx += 0x40;
        out_idx += 0x30;
    }

    (in_idx, out_idx)
}

#[cfg(target_arch = "aarch64")]
#[allow(unsafe_op_in_unsafe_fn)]
#[target_feature(enable = "neon")]
unsafe fn decode_chunk_neon(buffer: &[u8], output: &mut [u8]) -> (usize, usize) {
    use core::arch::aarch64::*;

    let mut in_idx = 0;
    let mut out_idx = 0;

    let lut_lo = vld1q_u8(
        [
            0x15, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x13, 0x1A, 0x1B, 0x1B,
            0x1B, 0x1A,
        ]
        .as_ptr(),
    );
    let lut_hi = vld1q_u8(
        [
            0x10, 0x10, 0x01, 0x02, 0x04, 0x08, 0x04, 0x08, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
            0x10, 0x10,
        ]
        .as_ptr(),
    );

    let lut_roll =
        vld1q_u8([0, 0x10, 0x13, 4, 191, 191, 185, 185, 0, 0, 0, 0, 0, 0, 0, 0].as_ptr());

    let shift_l = vld1q_s8([2, 4, 6, 0, 2, 4, 6, 0, 2, 4, 6, 0, 2, 4, 6, 0].as_ptr());
    let shift_r = vld1q_s8([-4i8, -2, 0, 0, -4, -2, 0, 0, -4, -2, 0, 0, -4, -2, 0, 0].as_ptr());
    let pack_shuf = vld1q_u8([0, 1, 2, 4, 5, 6, 8, 9, 10, 12, 13, 14, 255, 255, 255, 255].as_ptr());

    let mask_0f = vdupq_n_u8(0x0F);
    let mask_2f = vdupq_n_u8(0x2F);

    while in_idx + 16 <= buffer.len() {
        let in_val = vld1q_u8(buffer.as_ptr().add(in_idx));

        let hi_nibbles = vandq_u8(vshrq_n_u8(in_val, 4), mask_0f);
        let lo_nibbles = vandq_u8(in_val, mask_0f);

        let hi = vqtbl1q_u8(lut_hi, hi_nibbles);
        let lo = vqtbl1q_u8(lut_lo, lo_nibbles);

        let check = vandq_u8(hi, lo);
        if vmaxvq_u32(vreinterpretq_u32_u8(check)) != 0 {
            break;
        }

        let eq_2f = vceqq_u8(in_val, mask_2f);
        let idx = vaddq_u8(hi_nibbles, eq_2f);
        let shift = vqtbl1q_u8(lut_roll, idx);

        let decoded = vaddq_u8(in_val, shift);

        let p1 = vshlq_u8(decoded, shift_l);
        let p2 = vshlq_u8(vextq_u8(decoded, decoded, 1), shift_r);
        let merged = vorrq_u8(p1, p2);

        let packed = vqtbl1q_u8(merged, pack_shuf);

        let packed_ptr: *const uint8x16_t = &packed;
        core::ptr::copy_nonoverlapping(
            packed_ptr.cast::<u8>(),
            output.as_mut_ptr().add(out_idx),
            12,
        );

        in_idx += 16;
        out_idx += 12;
    }

    (in_idx, out_idx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{string::String, vec, vec::Vec};

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

                let mut enc_buf = vec![0u8; encoded_len(plain.len())];
                let enc_len = encode(plain, &mut enc_buf).unwrap();
                assert_eq!(&enc_buf[..enc_len], expected_encoded);

                let mut dec_buf = vec![0u8; decoded_len(enc_len)];
                let dec_len = decode(&enc_buf[..enc_len], &mut dec_buf).unwrap();
                assert_eq!(&dec_buf[..dec_len], plain);
            }
        }

        #[test]
        fn ok_invalid_length() {
            let mut buf = [0u8; 0x0A];

            assert_eq!(decode(b"Z", &mut buf), Err(Error::InvalidLength));
            assert_eq!(decode(b"Zg", &mut buf), Err(Error::InvalidLength));
            assert_eq!(decode(b"Zg=", &mut buf), Err(Error::InvalidLength));
            assert_eq!(decode(b"Zm9vY", &mut buf), Err(Error::InvalidLength));
        }

        #[test]
        fn ok_invalid_chars() {
            let mut buf = [0u8; 0x0A];

            assert_eq!(decode(b"Zg =", &mut buf), Err(Error::InvalidByte(2, b' ')));
            assert_eq!(decode(b"Zg\n=", &mut buf), Err(Error::InvalidByte(2, b'\n')));
            assert_eq!(decode(b"Zm9v-mFy", &mut buf), Err(Error::InvalidByte(4, b'-')));
        }

        #[test]
        fn ok_invalid_padding_placement() {
            let mut buf = [0u8; 0x0A];

            assert_eq!(decode(b"=m9v", &mut buf), Err(Error::InvalidPadding));
            assert_eq!(decode(b"Z=9v", &mut buf), Err(Error::InvalidPadding));
        }

        #[test]
        fn ok_binary_roundtrip() {
            let binary_data: Vec<u8> = (0..=0xFF).collect();

            let mut enc = vec![0u8; encoded_len(binary_data.len())];
            let enc_len = encode(&binary_data, &mut enc).unwrap();

            let mut dec = vec![0u8; decoded_len(enc_len)];
            let dec_len = decode(&enc[..enc_len], &mut dec).unwrap();
            assert_eq!(&dec[..dec_len], binary_data);
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
                let mut original_data = vec![0u8; len];
                rng.fill(&mut original_data[..]);

                let mut turbo_encoded = vec![0u8; encoded_len(original_data.len())];
                let enc_len = encode(&original_data, &mut turbo_encoded).unwrap();
                let standard_encoded = BASE64_STANDARD.encode(&original_data);

                assert_eq!(
                    &turbo_encoded[..enc_len],
                    standard_encoded.as_bytes(),
                    "Encoding parity failure at size {len}!"
                );

                let mut turbo_decoded = vec![0u8; decoded_len(enc_len)];
                let dec_len = decode(&turbo_encoded[..enc_len], &mut turbo_decoded).unwrap();
                let standard_decoded = BASE64_STANDARD.decode(&standard_encoded).unwrap();

                assert_eq!(
                    &turbo_decoded[..dec_len],
                    standard_decoded,
                    "Decoding parity failure at size {len}!"
                );
            }
        }

        #[test]
        fn ok_randomized_error_rejection_parity() {
            let mut rng = rng();
            let mut buf = vec![0u8; 0x2000];

            for _ in 0..0x400 {
                let len = rng.random_range(1..0x200) * 4;

                let temp_data = vec![0u8; decoded_len(len)];
                let mut valid_base64_string = vec![0u8; encoded_len(temp_data.len())];
                let enc_len = encode(&temp_data, &mut valid_base64_string).unwrap();
                valid_base64_string.truncate(enc_len);

                if valid_base64_string.is_empty() {
                    continue;
                }

                let corrupt_index = rng.random_range(0..valid_base64_string.len());
                valid_base64_string[corrupt_index] = b'%';

                let turbo_result = decode(&valid_base64_string, &mut buf);
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

                let mut enc = vec![0u8; encoded_len(bytes.len())];
                let enc_len = encode(bytes, &mut enc).unwrap();

                let mut dec_bytes = vec![0u8; decoded_len(enc_len)];
                let dec_len = decode(&enc[..enc_len], &mut dec_bytes).unwrap();

                let dec_string = String::from_utf8(dec_bytes[..dec_len].to_vec()).unwrap();

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

                let mut turbo_encoded = vec![0u8; encoded_len(bytes.len())];
                let enc_len = encode(bytes, &mut turbo_encoded).unwrap();
                let standard_encoded = BASE64_STANDARD.encode(bytes);

                assert_eq!(
                    &turbo_encoded[..enc_len],
                    standard_encoded.as_bytes(),
                    "UTF-8 Encoding parity failure at string length {len}!"
                );

                let mut turbo_decoded = vec![0u8; decoded_len(enc_len)];
                let dec_len = decode(&turbo_encoded[..enc_len], &mut turbo_decoded).unwrap();
                let standard_decoded = BASE64_STANDARD.decode(&standard_encoded).unwrap();

                assert_eq!(
                    &turbo_decoded[..dec_len],
                    standard_decoded,
                    "UTF-8 Decoding parity failure at string length {len}!"
                );
            }
        }
    }
}
