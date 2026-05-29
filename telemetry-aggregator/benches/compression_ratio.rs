use criterion::{black_box, criterion_group, criterion_main, Criterion};

use telemetry_aggregator::{GorillaCompressor, SensorPoint, parse_coap_payload};

fn generate_block(count: usize) -> Vec<SensorPoint> {
    (0..count)
        .map(|i| SensorPoint::new(
             0xDEAD_BEEF_CAFE_u128,
            1_717_012_345_678_901 + (i as u64 * 1_000_000),
            (i % 4) as u8,
            234.567 + (i as f64 * 0.001),
        ))
        .collect()
}

fn bench_compress_128_point_block(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression");
    let points = generate_block(128);

    group.bench_function("compress_128_points", |b| {
        b.iter(|| {
            let mut compressor = GorillaCompressor::new();
            let result = compressor.ingest(black_box(&points));
            black_box(result);
        });
    });

    group.bench_function("decompress_128_points", |b| {
        let mut compressor = GorillaCompressor::new();
        let compressed = compressor.ingest(&points).unwrap();

        b.iter(|| {
            let result = GorillaCompressor::decompress_block(black_box(&compressed), 128);
            let _ = black_box(result);
        });
    });

    group.finish();
}

fn bench_parse_packets(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");
    let sample = "DEADBEEFCAFE0000,1717012345678901,1,234.567,2\n"
        .repeat(100)
        .into_bytes();

    group.bench_function("parse_100_packets", |b| {
        b.iter(|| {
            let result = parse_coap_payload(black_box(&sample));
            let _ = black_box(result);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_compress_128_point_block, bench_parse_packets);
criterion_main!(benches);
