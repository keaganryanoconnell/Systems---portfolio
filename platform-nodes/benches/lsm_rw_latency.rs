//! Criterion Benchmark: LSM Storage Engine Read/Write Latency & Throughput
//!
//! Exercises the MemTable insert and read paths, and the SSTable writer/reader
//! to measure storage engine performance characteristics under production-like
//! load profiles.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use platform_nodes::storage::{LsmConfig, LsmEngine};
use std::path::PathBuf;

/// Returns a temporary directory path unique to the current bench run.
/// Using the process PID keeps parallel bench invocations isolated.
fn bench_data_dir(suffix: &str) -> PathBuf {
    let dir = PathBuf::from(format!(
        "target/bench_lsm_{}_{}_{}",
        std::process::id(),
        suffix,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.subsec_nanos())
    ));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Cleans up the temp directory after a benchmark completes.
fn cleanup(dir: &PathBuf) {
    let _ = std::fs::remove_dir_all(dir);
}

// ── MemTable Insert Throughput ────────────────────────────────────────────────

fn bench_memtable_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("lsm_memtable_insert");

    for n_keys in [64usize, 256, 1024].iter() {
        group.throughput(Throughput::Elements(*n_keys as u64));
        group.bench_with_input(
            BenchmarkId::new("sequential_keys", n_keys),
            n_keys,
            |b, &n| {
                let dir = bench_data_dir("memtable_insert");
                let engine = LsmEngine::open(LsmConfig {
                    data_dir: dir.clone(),
                    flush_threshold_bytes: 128 * 1024 * 1024, // 128 MB — prevent flushing during bench
                    compaction_trigger_files: 999,
                })
                .expect("LSM engine open failed");

                b.iter(|| {
                    for i in 0..n {
                        let key = format!("bench-key-{:08}", i).into_bytes();
                        let val = format!("bench-val-{:08}", i).into_bytes();
                        let _ = engine.put(black_box(key), black_box(val));
                    }
                });
                cleanup(&dir);
            },
        );
    }

    group.finish();
}

// ── MemTable Read Latency ─────────────────────────────────────────────────────

fn bench_memtable_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("lsm_memtable_read");

    for n_keys in [64usize, 256, 1024].iter() {
        group.throughput(Throughput::Elements(*n_keys as u64));
        group.bench_with_input(
            BenchmarkId::new("sequential_reads", n_keys),
            n_keys,
            |b, &n| {
                let dir = bench_data_dir("memtable_read");
                let engine = LsmEngine::open(LsmConfig {
                    data_dir: dir.clone(),
                    flush_threshold_bytes: 128 * 1024 * 1024,
                    compaction_trigger_files: 999,
                })
                .expect("LSM engine open failed");

                // Pre-populate
                for i in 0..n {
                    let key = format!("bench-key-{:08}", i).into_bytes();
                    let val = format!("bench-val-{:08}", i).into_bytes();
                    let _ = engine.put(key, val);
                }

                b.iter(|| {
                    for i in 0..n {
                        let key = format!("bench-key-{:08}", i).into_bytes();
                        let _ = black_box(engine.get(black_box(&key)));
                    }
                });

                cleanup(&dir);
            },
        );
    }

    group.finish();
}

// ── Mixed Read/Write Workload (80/20 ratio) ───────────────────────────────────

fn bench_lsm_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("lsm_mixed_80_20");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("reads_80pct_writes_20pct", |b| {
        let dir = bench_data_dir("mixed");
        let engine = LsmEngine::open(LsmConfig {
            data_dir: dir.clone(),
            flush_threshold_bytes: 128 * 1024 * 1024,
            compaction_trigger_files: 999,
        })
        .expect("LSM engine open failed");

        // Seed 200 initial keys
        for i in 0..200usize {
            let _ = engine.put(
                format!("seed-{:08}", i).into_bytes(),
                format!("value-{:08}", i).into_bytes(),
            );
        }

        b.iter(|| {
            // 800 reads across existing keys
            for i in 0..800usize {
                let key = format!("seed-{:08}", i % 200).into_bytes();
                let _ = black_box(engine.get(black_box(&key)));
            }
            // 200 writes to new keys
            for i in 200..400usize {
                let _ = engine.put(
                    black_box(format!("new-{:08}", i).into_bytes()),
                    black_box(format!("val-{:08}", i).into_bytes()),
                );
            }
        });

        cleanup(&dir);
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_memtable_insert,
    bench_memtable_read,
    bench_lsm_mixed_workload,
);
criterion_main!(benches);
