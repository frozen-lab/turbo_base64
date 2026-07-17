[![docs.rs](https://img.shields.io/docsrs/turbo_base64?logo=rust)](https://docs.rs/turbo_base64)
[![Latest Version](https://img.shields.io/crates/v/turbo_base64.svg?logo=rust)](https://crates.io/crates/turbo_base64)
[![License](https://img.shields.io/github/license/frozen-lab/turbo_base64?logo=open-source-initiative&logoColor=white)](https://github.com/frozen-lab/turbo_base64/blob/master/LICENSE)
[![Tests](https://github.com/frozen-lab/turbo_base64/actions/workflows/tests.yaml/badge.svg)](https://github.com/frozen-lab/turbo_base64/actions/workflows/tests.yaml)

# TurboBase64

Hardware accelerated encoding and decoding of bytes or utf-8 using standard RFC 4648 base64 spec

## `no_std` Support

`turbo_base64` crate is fully `#![no_std]` and does not require the Rust standard library.

It can be used in embedded, kernel, bootloader, and other constrained environments.

## Usage

Add following to your `Cargo.toml`,

```toml
[dependencies]
turbo_base64 = { version = "0.0.5" }

```

> [!NOTE]
> Current version of `turbo_base64` requires Rust 1.86 or later.

## Benchmarks

Observed measurements for encode,

| Payload  | Avg Time  | Throughput |
|:---------|:----------|:-----------|
| 256 KiB  | 37.07 µs  | 6.59 GiB/s |
| 512 KiB  | 71.85 µs  | 6.80 GiB/s |
| 1 MiB    | 141.61 µs | 6.90 GiB/s |
| 2 MiB    | 285.02 µs | 6.85 GiB/s |

Observed measurements for decode,

| Payload | Avg Time | Throughput |
|:--------|:---------|:-----------|
| 4 KiB   | 3.29 µs  | 1.16 GiB/s |
| 8 KiB   | 6.41 µs  | 1.19 GiB/s |
| 16 KiB  | 13.34 µs | 1.14 GiB/s |
| 32 KiB  | 26.04 µs | 1.17 GiB/s |
