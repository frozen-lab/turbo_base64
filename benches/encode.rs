//! Benchmarks to measure througputs
//! Run using: `taskset -c 3 cargo +nightly bench --bench encode --profile release --features=nightly`

use core::hint::black_box;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use turbo_base64::{encode, encoded_len};

const KB: usize = 0x400;
const MB: usize = KB * KB;

pub fn bench_base64(c: &mut Criterion) {
    let sizes = [
        0x200,
        1 * KB,
        0x40 * KB,
        0x200 * KB,
        0x400 * KB,
        1 * MB,
        0x40 * MB,
        0x200 * MB,
        0x400 * MB,
    ];
    let mut group = c.benchmark_group("turbo_base64_encode_throughput");

    for size in sizes {
        let plain_data = vec![0u8; size];
        let mut encoded = vec![0u8; encoded_len(size)];

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::new("encode", size), &plain_data, |b, data| {
            b.iter(|| encode(black_box(data), black_box(&mut encoded)))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_base64);
criterion_main!(benches);
