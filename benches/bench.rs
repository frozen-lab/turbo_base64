//! Benchmarks to measure througputs
//! Run using: `taskset -c 3,4,5,6 cargo bench --bench bench --profile release`

use core::hint::black_box;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use turbo_base64::{decode, encode};

pub fn bench_base64(c: &mut Criterion) {
    let sizes = [0x1000, 0x10_000, 0x20_000, 0x40_000];
    let mut group = c.benchmark_group("turbo_base64_throughput");

    for size in sizes {
        let plain_data = vec![0u8; size];
        let encoded_data = encode(&plain_data);

        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new("encode", size), &plain_data, |b, data| {
            b.iter(|| encode(black_box(data)))
        });

        group.bench_with_input(BenchmarkId::new("decode", size), &encoded_data, |b, data| {
            b.iter(|| decode(black_box(data)))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_base64);
criterion_main!(benches);
