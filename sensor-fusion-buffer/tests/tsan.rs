use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use sensor_fusion_buffer::{MpmcRingBuffer, SensorFrame};

const RING_SIZE: usize = 1 << 10;
const FRAMES_PER_PRODUCER: usize = 10_000;

#[test]
fn test_tsan_3_producers_1_consumer_no_data_races() {
    let buffer = Arc::new(MpmcRingBuffer::<SensorFrame, RING_SIZE>::new());
    let running = Arc::new(AtomicBool::new(true));

    let mut producer_handles = Vec::new();

    for pid in 0..3 {
        let buf = buffer.clone();
        let run = running.clone();
        let h = thread::spawn(move || {
            for seq in 0..FRAMES_PER_PRODUCER {
                let frame = SensorFrame::new_imu(
                    pid,
                    (seq as u64) * 1_000_000,
                    seq as u64,
                    0.01, 0.02, 0.98, 0.1, 0.05, 0.03,
                );

                let mut attempts = 0;
                loop {
                    match buf.try_write(frame) {
                        Ok(_) => break,
                        Err(_) => {
                            attempts += 1;
                            if attempts > 100 {
                                thread::sleep(Duration::from_micros(1));
                                attempts = 0;
                            }
                        }
                    }
                }
            }
        });
        producer_handles.push(h);
    }

    let consumer_buf = buffer.clone();
    let consumer_run = running.clone();
    let consumer_handle = thread::spawn(move || {
        let mut total = 0usize;
        let mut batch = Vec::with_capacity(64);

        while consumer_run.load(Ordering::Acquire) {
            batch.clear();
            let count = consumer_buf.try_read_batch(&mut batch, 64).unwrap_or(0);
            total += count;

            if total >= FRAMES_PER_PRODUCER * 3 {
                consumer_run.store(false, Ordering::Release);
                break;
            }

            if count == 0 {
                thread::yield_now();
            }
        }

        assert!(total >= FRAMES_PER_PRODUCER * 3,
            "Consumer should have read all {} frames, got {}",
            FRAMES_PER_PRODUCER * 3, total);
    });

    for h in producer_handles {
        h.join().unwrap();
    }
    consumer_handle.join().unwrap();
}
