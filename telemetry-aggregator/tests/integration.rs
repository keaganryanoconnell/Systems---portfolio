use std::net::UdpSocket;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use telemetry_aggregator::{
    parse_coap_payload, GorillaCompressor, IngestStats, LogBuffer, PacketRing,
};

#[test]
fn test_100k_packets_compression() {
    let data_dir = tempfile::tempdir().unwrap();
    let ring = Arc::new(PacketRing::new());
    let mut compressor = GorillaCompressor::new();
    let mut buffer = LogBuffer::new(256 * 1024 * 1024, data_dir.path());
    let mut stats = IngestStats::new();

    let send_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let recv_socket = UdpSocket::bind("127.0.0.1:19999").unwrap();
    recv_socket
        .set_read_timeout(Some(Duration::from_millis(10)))
        .ok();

    let dest_addr = "127.0.0.1:19999";
    send_socket.connect(dest_addr).unwrap();

    let ring_clone = ring.clone();
    let recv_handle = thread::spawn(move || {
        let mut buf = [0u8; 2048];
        loop {
            match recv_socket.recv_from(&mut buf) {
                Ok((len, _)) => {
                    ring_clone.write_frame(&buf[..len]).ok();
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }
    });

    let total = 100_000;
    for i in 0..total {
        let ts = 1_717_012_345_678_901u64 + (i as u64 * 1_000_000);
        let val = 234.567 + (i as f64 * 0.001);

        let payload = format!("{:032x},{},{},{},{}", 0xDEAD_BEEF_CAFE_u128, ts, 1, val, 2);
        send_socket.send(payload.as_bytes()).unwrap();

        if i % 1000 == 0 {
            thread::sleep(Duration::from_micros(10));
        }
    }

    recv_handle.join().ok();

    let mut processed = 0;
    while ring.has_available() {
        if let Some((data, idx)) = ring.read_frame() {
            stats.packets_received += 1;
            stats.bytes_ingested += data.len() as u64;

            if let Ok(points) = parse_coap_payload(data) {
                stats.points_processed += points.len() as u64;
                processed += points.len();

                if let Some(compressed) = compressor.ingest(&points) {
                    stats.bytes_compressed += compressed.len() as u64;
                    stats.blocks_written += 1;
                    buffer.write(&compressed).unwrap();
                }
            }

            ring.mark_consumed(idx);
        }
    }

    if let Some(flushed) = compressor.flush() {
        stats.bytes_compressed += flushed.len() as u64;
        buffer.write(&flushed).ok();
    }

    assert!(processed > 0, "Should have processed at least some points");
    println!("Processed {} points", processed);
    println!("Packets received: {}", stats.packets_received);
    println!("Bytes ingested: {}", stats.bytes_ingested);
    println!("Bytes compressed: {}", stats.bytes_compressed);

    if stats.bytes_compressed > 0 {
        let ratio = stats.compression_ratio();
        println!("Compression ratio: {:.1}:1", ratio);
        assert!(ratio > 5.0, "Expected >5:1 compression, got {:.1}:1", ratio);
    }

    assert!(
        buffer.memory_used() < 256 * 1024 * 1024,
        "Memory usage {} exceeds 256MB cap",
        buffer.memory_used()
    );
}
