use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use sensor_fusion_buffer::{
    CpuAffinity, FusionBufferError, MpmcRingBuffer,
    SensorFrame, SensorType,
};

const RING_SIZE: usize = 1 << 16;
const TOTAL_FRAMES: u64 = 30_000;

struct ProducerStats {
    frames_written: AtomicU64,
    total_latency_ns: AtomicU64,
    min_latency_ns: AtomicU64,
    max_latency_ns: AtomicU64,
}

impl ProducerStats {
    fn new() -> Self {
        Self {
            frames_written: AtomicU64::new(0),
            total_latency_ns: AtomicU64::new(0),
            min_latency_ns: AtomicU64::new(u64::MAX),
            max_latency_ns: AtomicU64::new(0),
        }
    }
}

fn producer(
    buffer: Arc<MpmcRingBuffer<SensorFrame, RING_SIZE>>,
    stats: Arc<ProducerStats>,
    id: u32,
    sensor_type: SensorType,
    interval_us: u64,
    running: Arc<AtomicBool>,
) {
    let mut seq: u64 = 0;
    let base_ts = Instant::now();

    while running.load(Ordering::Acquire) {
        let t0 = Instant::now();
        let now_ns = base_ts.elapsed().as_nanos() as u64;

        let frame = match sensor_type {
            SensorType::LiDAR => SensorFrame::new_lidar(id, now_ns, seq, 128000, 50.0 + (seq as f32 * 0.001)),
            SensorType::Camera => SensorFrame::new_camera(id, now_ns, seq, 1920, 1080, 15000),
            SensorType::IMU => SensorFrame::new_imu(id, now_ns, seq, 0.01, 0.02, 0.98, 0.1, 0.05, 0.03),
            _ => SensorFrame::new_imu(id, now_ns, seq, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0),
        };

        let elapsed = t0.elapsed().as_nanos() as u64;
        stats.total_latency_ns.fetch_add(elapsed, Ordering::Relaxed);
        stats.frames_written.fetch_add(1, Ordering::Relaxed);

        let mut min = stats.min_latency_ns.load(Ordering::Relaxed);
        while elapsed < min {
            let _ = stats.min_latency_ns.compare_exchange(min, elapsed, Ordering::Relaxed, Ordering::Relaxed);
            min = stats.min_latency_ns.load(Ordering::Relaxed);
        }

        let mut max = stats.max_latency_ns.load(Ordering::Relaxed);
        while elapsed > max {
            let _ = stats.max_latency_ns.compare_exchange(max, elapsed, Ordering::Relaxed, Ordering::Relaxed);
            max = stats.max_latency_ns.load(Ordering::Relaxed);
        }

        match buffer.try_write(frame) {
            Ok(_) => seq += 1,
            Err(FusionBufferError::BufferFull) => {
                eprintln!("[Producer {}] Buffer full at seq {}", id, seq);
            }
            Err(e) => {
                eprintln!("[Producer {}] Error: {}", id, e);
            }
        }

        if seq >= TOTAL_FRAMES / 3 {
            break;
        }

        thread::sleep(Duration::from_micros(interval_us));
    }
}

fn main() {
    println!("=== Sensor Fusion MPMC Ring Buffer ===");
    println!("Lock-free CAS producers · Deterministic overwrite · CPU affinity");
    println!();
    println!("Configuration: {} slots, 3 producers, 1 consumer", RING_SIZE);
    println!("Target: {} total frames", TOTAL_FRAMES);

    let cores = CpuAffinity::available_cores();
    println!("Available CPU cores: {}", cores);

    let buffer = Arc::new(MpmcRingBuffer::<SensorFrame, RING_SIZE>::new());
    let running = Arc::new(AtomicBool::new(true));

    let producer_stats: [Arc<ProducerStats>; 3] = [
        Arc::new(ProducerStats::new()),
        Arc::new(ProducerStats::new()),
        Arc::new(ProducerStats::new()),
    ];

    let sensors = [
        (SensorType::LiDAR, 10_000u64, "LiDAR ~100Hz"),
        (SensorType::Camera, 33_000u64, "Camera ~30Hz"),
        (SensorType::IMU, 1_000u64, "IMU ~1KHz"),
    ];

    let mut handles = Vec::new();
    for i in 0..3 {
        let buf = buffer.clone();
        let stats = producer_stats[i].clone();
        let run = running.clone();
        let (s_type, interval, name) = sensors[i];
        let id = i as u32;

        let h = thread::Builder::new()
            .name(format!("producer-{}", name))
            .spawn(move || {
                println!("[Producer {}] {} started (interval={}us)", id, name, interval);
                producer(buf, stats, id, s_type, interval, run);
                println!("[Producer {}] {} finished", id, name);
            })
            .unwrap();
        handles.push(h);
    }

    let consumer_buf = buffer.clone();
    let consumer_running = running.clone();
    let consumer_handle = thread::Builder::new()
        .name("consumer-fusion".into())
        .spawn(move || {
            if cores > 1 {
                let target_core = (cores - 1).min(3);
                match CpuAffinity::pin_current(target_core) {
                    Ok(()) => println!("[Consumer] Pinned to core {}", target_core),
                    Err(e) => eprintln!("[Consumer] Affinity failed: {}", e),
                }
            }

            let mut merged: Vec<SensorFrame> = Vec::with_capacity(64);
            let mut total = 0u64;

            while consumer_running.load(Ordering::Acquire) {
                merged.clear();
                let count = consumer_buf.try_read_batch(&mut merged, 64).unwrap_or(0);
                total += count as u64;

                if count > 0 && total.is_multiple_of(10000) {
                    println!("[Consumer] Read {} frames total, batch of {}", total, count);
                }

                if total >= TOTAL_FRAMES {
                    consumer_running.store(false, Ordering::Release);
                }

                if count == 0 {
                    thread::sleep(Duration::from_micros(10));
                }
            }

            println!("[Consumer] Total frames consumed: {}", total);
        })
        .unwrap();

    for h in handles {
        h.join().unwrap();
    }
    consumer_handle.join().unwrap();

    println!();
    println!("=== RESULTS ===");
    for (i, s) in producer_stats.iter().enumerate() {
        let written = s.frames_written.load(Ordering::Relaxed);
        let total_lat = s.total_latency_ns.load(Ordering::Relaxed);
        let min_lat = s.min_latency_ns.load(Ordering::Relaxed);
        let max_lat = s.max_latency_ns.load(Ordering::Relaxed);
        let avg_lat = total_lat.checked_div(written).unwrap_or(0);

        println!(
            "Producer {}: {} frames | avg={}ns | min={}ns | max={}ns",
            i, written, avg_lat, min_lat, max_lat
        );
    }

    println!("Simulation complete: full pipeline tested");
}
