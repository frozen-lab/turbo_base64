# TurboBase64

Hardware accelerated encoding and decoding of bytes or utf-8 using standard RFC 4648 base64 spec

## Benchmarks

Observed measurements for encode,

| Payload      | Avg Time     | Throughput   |
|:-------------|:-------------|:-------------|
| 32 B         | 100.93 ns    | 302.36 MiB/s |
| 1 KiB        | 1.30 µs      | 749.08 MiB/s |
| 4 KiB        | 4.21 µs      | 926.41 MiB/s |
| 64 KiB       | 61.47 µs     | 1.0167 GiB/s |

Observed measurements for decode,

| Payload      | Avg Time     | Throughput   |
|:-------------|:-------------|:-------------|
| 32 B         | 102.13 ns    | 298.81 MiB/s |
| 1 KiB        | 2.75 µs      | 354.17 MiB/s |
| 4 KiB        | 11.66 µs     | 335.02 MiB/s |
| 64 KiB       | 184.81 µs    | 338.18 MiB/s |
