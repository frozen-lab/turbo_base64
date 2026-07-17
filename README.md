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
turbo_base64 = { version = "0.0.4" }

```

> [!NOTE]
> Current version of `turbo_base64` requires Rust 1.86 or later.

## Benchmarks

Observed measurements for encode,

| Payload | Avg Time | Throughput |
|:--------|:---------|:-----------|
| 4 KiB   | 2.63 µs  | 1.45 GiB/s |
| 8 KiB   | 5.34 µs  | 1.43 GiB/s |
| 16 KiB  | 10.46 µs | 1.46 GiB/s |
| 32 KiB  | 21.56 µs | 1.42 GiB/s |

Observed measurements for decode,

| Payload | Avg Time | Throughput |
|:--------|:---------|:-----------|
| 4 KiB   | 3.29 µs  | 1.16 GiB/s |
| 8 KiB   | 6.41 µs  | 1.19 GiB/s |
| 16 KiB  | 13.34 µs | 1.14 GiB/s |
| 32 KiB  | 26.04 µs | 1.17 GiB/s |
