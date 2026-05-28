//! Criterion Benchmark: SPSC Queue Throughput & Latency
//!
//! Measures the lock-free SPSC queue's single-threaded and cross-thread
//! throughput characteristics under sustained push/pop pressure.

use core_sys::spsc::SpscQueue;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;
use std::thread;

// ── Single-Threaded Throughput ────────────────────────────────────────────────

fn bench_spsc_single_thread_push_pop(c: &mut Criterion) {
    let mut group = c.benchmark_group("spsc_single_thread");
    group.throughput(Throughput::Elements(1024));

    group.bench_function("push_pop_1024_elements", |b| {
        b.iter(|| {
            let queue: SpscQueue<u64, 2048> = SpscQueue::new();
            for i in 0u64..1024 {
                let _ = queue.push(black_box(i));
            }
            for _ in 0u64..1024 {
                let _ = black_box(queue.pop());
            }
        });
    });

    group.finish();
}

// ── Cross-Thread Round-Trip Latency ──────────────────────────────────────────

fn bench_spsc_cross_thread(c: &mut Criterion) {
    let mut group = c.benchmark_group("spsc_cross_thread");

    for batch_size in [64u64, 256, 1024].iter() {
        group.throughput(Throughput::Elements(*batch_size));
        group.bench_with_input(
            BenchmarkId::new("producer_consumer", batch_size),
            batch_size,
            |b, &n| {
                b.iter(|| {
                    // Safety: SpscQueue requires exactly one producer and one consumer.
                    // We use Arc to share the queue pointer across threads.
                    let queue: Arc<SpscQueue<u64, 2048>> = Arc::new(SpscQueue::new());
                    let q_producer = queue.clone();
                    let q_consumer = queue.clone();

                    let producer = thread::spawn(move || {
                        for i in 0..n {
                            // Busy-retry until the slot is available
                            while q_producer.push(black_box(i)).is_err() {
                                std::hint::spin_loop();
                            }
                        }
                    });

                    let consumer = thread::spawn(move || {
                        let mut received = 0u64;
                        while received < n {
                            if black_box(q_consumer.pop()).is_some() {
                                received += 1;
                            } else {
                                std::hint::spin_loop();
                            }
                        }
                    });

                    producer.join().unwrap();
                    consumer.join().unwrap();
                });
            },
        );
    }

    group.finish();
}

// ── Contention: Alternating Push/Pop Under Back-Pressure ─────────────────────

fn bench_spsc_bounded_back_pressure(c: &mut Criterion) {
    let mut group = c.benchmark_group("spsc_back_pressure");
    group.throughput(Throughput::Elements(512));

    group.bench_function("half_capacity_drain", |b| {
        b.iter(|| {
            // Fill half the queue, then drain completely, repeat
            let queue: SpscQueue<u32, 1024> = SpscQueue::new();
            for i in 0..512u32 {
                let _ = queue.push(black_box(i));
            }
            let mut total = 0u64;
            while let Some(v) = queue.pop() {
                total += black_box(v) as u64;
            }
            black_box(total)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_spsc_single_thread_push_pop,
    bench_spsc_cross_thread,
    bench_spsc_bounded_back_pressure,
);
criterion_main!(benches);
