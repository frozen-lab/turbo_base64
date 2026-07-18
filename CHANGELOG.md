# Changelog

## [0.1.0](https://github.com/frozen-lab/turbo_base64/compare/v0.0.5...v0.1.0) - 2026-07-19
- Impl of `AVX2` ISA for `encode` function
- Impl of `AVX512` ISA for `encode` function
- Impl of `AVX2` ISA for `encode` function
- Impl of `AVX512` ISA for `decode` function
- Impl of `neon` ISA for `encode` function
- Impl of `neon` ISA for `decode` function

## [0.0.5](https://github.com/frozen-lab/turbo_base64/compare/v0.0.4...v0.0.5) - 2026-07-17
- Improved `encode` perf by 110% w/ use of `ssse4.2` SIMD ISA  

## [0.0.4](https://github.com/frozen-lab/turbo_base64/compare/v0.0.3...v0.0.4) - 2026-07-17
- Migration to be strictly `no_std` w/o requiring a global allocator
- Impl of `encoded_len` & `decoded_len`

## [0.0.3](https://github.com/frozen-lab/turbo_base64/compare/v0.0.2...v0.0.3) - 2026-07-16
- Improve throughput of `decode` by 175%

## [0.0.2](https://github.com/frozen-lab/turbo_base64/compare/v0.0.1...v0.0.2) - 2026-07-16
- Improve throughput of `encode` by 50%
- Impl of fuzz testing for bug hunting 

## [0.0.1](https://github.com/frozen-lab/turbo_base64/commits/v0.0.1) - 2026-07-15
- First release for crates.io
