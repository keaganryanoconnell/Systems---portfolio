use std::net::UdpSocket;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use telemetry_aggregator::{
    GorillaCompressor, IngestStats, LogBuffer, PacketRing,
    parse_coap_payload,
};

fn main() {
    println!("=== Telemetry Edge Aggregator ===");
    println!("Zero-copy packet ring · Gorilla compression · Bounded memory");
    println!();

    let bind_addr = "0.0.0.0:5683";
    let data_dir = PathBuf::from("./telemetry_data");
    let memory_cap: usize = 256 * 1024 * 1024;

    let ring = Arc::new(PacketRing::new());
    let mut compressor = GorillaCompressor::new();
    let mut buffer = LogBuffer::new(memory_cap, &data_dir);
    let mut stats = IngestStats::new();

    println!("Packet ring: {} frames × {} bytes = {}KB",
        ring.frame_count(), ring.frame_size(), ring.total_size() / 1024);
    println!("Memory cap: {}MB", memory_cap / 1024 / 1024);
    println!("Listening on UDP {}", bind_addr);
    println!();

    let ring_clone = ring.clone();
    let _recv_thread = thread::spawn(move || {
        let socket = UdpSocket::bind(bind_addr).expect("bind failed");
        socket.set_read_timeout(Some(Duration::from_secs(1))).ok();

        let mut buf = vec![0u8; 65536];
        loop {
            match socket.recv_from(&mut buf) {
                Ok((len, _src)) => {
                    if ring_clone.write_frame(&buf[..len]).is_err() {
                        eprintln!("[WARN] packet ring full, dropping packet");
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(e) => eprintln!("[ERROR] recv error: {}", e),
            }
        }
    });

    let report_interval = Duration::from_secs(5);
    let mut last_report = Instant::now();

    loop {
        if ring.has_available() {
            if let Some((data, idx)) = ring.read_frame() {
                stats.packets_received += 1;
                stats.bytes_ingested += data.len() as u64;

                if let Ok(points) = parse_coap_payload(data) {
                    stats.points_processed += points.len() as u64;

                    if let Some(compressed) = compressor.ingest(&points) {
                        stats.bytes_compressed += compressed.len() as u64;
                        stats.blocks_written += 1;
                        buffer.write(&compressed).ok();
                    }
                }

                ring.mark_consumed(idx);
            }
        } else {
            if let Some(flushed) = compressor.flush() {
                stats.bytes_compressed += flushed.len() as u64;
                stats.blocks_written += 1;
                buffer.write(&flushed).ok();
            }
        }

        if last_report.elapsed() >= report_interval {
            println!(
                "Packets: {} | Points: {} | Ratio: {:.1}:1 | Memory: {}MB | Segments: {}",
                stats.packets_received,
                stats.points_processed,
                stats.compression_ratio(),
                buffer.memory_used() / 1024 / 1024,
                buffer.segment_count(),
            );
            last_report = Instant::now();
        }

        thread::sleep(Duration::from_millis(1));
    }
}
