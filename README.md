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
| 4 KiB        | 2.89 µs      | 1.31 GiB/s   |
| 64 KiB       | 45.20 µs     | 1.35 GiB/s   |
| 128 KiB      | 93.78 µs     | 1.05 GiB/s   |
| 256 KiB      | 183.64 µs    | 1.33 GiB/s   |

Observed measurements for decode,

| Payload      | Avg Time     | Throughput   |
|:-------------|:-------------|:-------------|
| 4 KiB        | 4.48 µs      | 872.55 MiB/s |
| 64 KiB       | 70.75 µs     | 883.38 MiB/s |
| 128 KiB      | 142.14 µs    | 879.43 MiB/s |
| 256 KiB      | 292.10 µs    | 855.86 MiB/s |
