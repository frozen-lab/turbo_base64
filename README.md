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

| Payload Size | x86_64 Throughput (GiB/s) | AArch64 Throughput (GiB/s) |
|-------------:|--------------------------:|---------------------------:|
|        512 B |                    15.190 |                      6.370 |
|        1 KiB |                    29.129 |                      6.691 |
|       64 KiB |                    26.756 |                      6.869 |
|      512 KiB |                    27.414 |                      6.715 |
|        1 MiB |                    31.901 |                      6.659 |
|       64 MiB |                    15.423 |                      6.642 |
|      128 MiB |                    10.645 |                      6.612 |
|      256 MiB |                     8.552 |                      6.623 |
|      512 MiB |                     7.994 |                      6.600 |
|        1 GiB |                     7.763 |                      6.571 |

Observed measurements for decode,

| Payload Size | x86_64 Throughput (GiB/s) | AArch64 Throughput (GiB/s) |
|-------------:|--------------------------:|---------------------------:|
|        512 B |                     6.415 |                      3.691 |
|        1 KiB |                     8.307 |                      3.804 |
|       64 KiB |                     8.612 |                      3.892 |
|      512 KiB |                     8.805 |                      3.857 |
|        1 MiB |                     8.644 |                      3.850 |
|       64 MiB |                     5.229 |                      3.883 |
|      128 MiB |                     4.521 |                      3.884 |
|      256 MiB |                     4.327 |                      3.886 |
|      512 MiB |                     4.266 |                      3.884 |
|        1 GiB |                     4.243 |                      3.887 |
