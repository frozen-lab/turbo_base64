//! Benchmarks to measure througputs
//! Run using: `taskset -c 3,4,5,6 cargo bench --bench decode --profile release`

use core::hint::black_box;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use turbo_base64::{decode, decoded_len, encode, encoded_len};

pub fn bench_base64(c: &mut Criterion) {
    let sizes = [0x1000, 0x2000, 0x4000, 0x8_000];
    let mut group = c.benchmark_group("turbo_base64_decode_throughput");

    for size in sizes {
        let plain_data = vec![0u8; size];

        let mut encoded = vec![0u8; encoded_len(size)];
        let len = encode(&plain_data, &mut encoded).unwrap();

        let mut decoded = vec![0u8; decoded_len(len)];

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::new("decode", size), &encoded, |b, data| {
            b.iter(|| decode(black_box(data), black_box(&mut decoded)))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_base64);
criterion_main!(benches);
