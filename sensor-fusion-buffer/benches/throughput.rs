use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sensor_fusion_buffer::{MpmcRingBuffer, SensorFrame};

const RING_SIZE: usize = 1 << 16;

fn bench_multi_producer_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("mpmc_throughput");
    group.sample_size(10);

    group.bench_function("3_producers_10k_frames_each", |b| {
        b.iter(|| {
            let buffer = Arc::new(MpmcRingBuffer::<SensorFrame, RING_SIZE>::new());
            let running = Arc::new(AtomicBool::new(true));
            let mut handles = Vec::new();

            for pid in 0..3 {
                let buf = buffer.clone();
                let _run = running.clone();
                handles.push(thread::spawn(move || {
                    for seq in 0..10_000u64 {
                        let frame = SensorFrame::new_imu(
                            pid,
                            seq * 1000,
                            seq,
                            0.0,
                            0.0,
                            1.0,
                            0.0,
                            0.0,
                            0.0,
                        );
                        loop {
                            match buf.try_write(black_box(frame)) {
                                Ok(_) => break,
                                Err(_) => std::hint::spin_loop(),
                            }
                        }
                    }
                }));
            }

            let consumer_buf = buffer.clone();
            let consumer_run = running.clone();
            let consumer = thread::spawn(move || {
                let mut batch = Vec::with_capacity(64);
                let mut total = 0usize;
                while total < 30_000 {
                    batch.clear();
                    if let Ok(n) = consumer_buf.try_read_batch(&mut batch, 64) {
                        total += n;
                    }
                    if total < 30_000 {
                        std::hint::spin_loop();
                    }
                }
                consumer_run.store(false, black_box(Ordering::Release));
            });

            for h in handles {
                h.join().unwrap();
            }
            consumer.join().unwrap();
        });
    });

    group.finish();
}

criterion_group!(benches, bench_multi_producer_throughput);
criterion_main!(benches);
