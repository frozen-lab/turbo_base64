[![Latest Version](https://img.shields.io/crates/v/turbo_base64.svg)](https://crates.io/crates/turbo_base64)
[![License](https://img.shields.io/github/license/frozen-lab/turbo_base64?logo=open-source-initiative&logoColor=white)](https://github.com/frozen-lab/turbo_base64/blob/master/LICENSE)
[![Tests](https://github.com/frozen-lab/turbo_base64/actions/workflows/tests.yaml/badge.svg)](https://github.com/frozen-lab/turbo_base64/actions/workflows/tests.yaml)

# TurboBase64

Hardware accelerated encoding and decoding of bytes or utf-8 using standard RFC 4648 base64 spec

## Usage

Add following to your `Cargo.toml`,

```toml
[dependencies]
turbo_base64 = { version = "0.0.2" }

```

> [!NOTE]
> Current version of `turbo_base64` requires Rust 1.86 or later.

## Benchmarks

Observed measurements for encode,

| Payload      | Avg Time     | Throughput   |
|:-------------|:-------------|:-------------|
| 4 KiB        | 2.82 µs      | 1.35 GiB/s   |
| 64 KiB       | 44.95 µs     | 1.36 GiB/s   |
| 128 KiB      | 99.08 µs     | 1.23 GiB/s   |
| 256 KiB      | 195.10 µs    | 1.25 GiB/s   |

Observed measurements for decode,

| Payload      | Avg Time     | Throughput   |
|:-------------|:-------------|:-------------|
| 32 B         | 102.13 ns    | 298.81 MiB/s |
| 1 KiB        | 2.75 µs      | 354.17 MiB/s |
| 4 KiB        | 11.66 µs     | 335.02 MiB/s |
| 64 KiB       | 184.81 µs    | 338.18 MiB/s |
