//! Lock-Free, Zero-Allocation Telemetry Logger Engine
//!
//! Exposes stack-allocated formatting APIs and a background worker thread
//! to drain the telemetry events queue and stream structured JSON to stdout.

use crate::spsc::SpscQueue;
use std::fmt::{Arguments, Write};
use std::io::Write as IoWrite;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Capacity of the static lock-free telemetry queue
const QUEUE_CAPACITY: usize = 1024;

/// Represents a single telemetry log record in the circular queue.
pub struct LogEntry {
    /// Timestamp in nanoseconds since UNIX epoch
    pub timestamp_ns: u64,
    /// Static log level string (e.g. "INFO")
    pub level: &'static str,
    /// Subsystem target path
    pub target: &'static str,
    /// Formatted message byte length
    pub msg_len: usize,
    /// Stack-allocated buffer for the formatted log text
    pub msg_buf: [u8; 128],
}

/// Global atomic flag to manage background daemon lifecycle
static DAEMON_RUNNING: AtomicBool = AtomicBool::new(false);

/// Global static telemetry queue
static TELEMETRY_QUEUE: OnceLock<SpscQueue<LogEntry, QUEUE_CAPACITY>> = OnceLock::new();

/// Global static daemon thread handle
static DAEMON_INITIALIZED: OnceLock<()> = OnceLock::new();

/// Retrives a reference to the global static SpscQueue.
fn get_queue() -> &'static SpscQueue<LogEntry, QUEUE_CAPACITY> {
    TELEMETRY_QUEUE.get_or_init(SpscQueue::new)
}

/// Helper struct that implements `std::fmt::Write` to format string data
/// directly into a stack-allocated byte slice without allocations.
struct BufferWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> Write for BufferWriter<'a> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = self.buf.len() - self.pos;

        if bytes.len() > remaining {
            // Copy as much as fits and signal truncation error
            let copy_len = remaining;
            self.buf[self.pos..self.pos + copy_len].copy_from_slice(&bytes[..copy_len]);
            self.pos += copy_len;
            return Err(std::fmt::Error);
        }

        self.buf[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
        self.pos += bytes.len();
        Ok(())
    }
}

/// Initializes the background telemetry daemon.
///
/// Spawns a dedicated worker thread to drain logging buffers.
/// Returns true if initialized, false if the daemon was already running.
pub fn init_telemetry_daemon() -> bool {
    let mut initialized = false;

    DAEMON_INITIALIZED.get_or_init(|| {
        DAEMON_RUNNING.store(true, Ordering::Release);

        thread::spawn(move || {
            let queue = get_queue();
            let mut stdout = std::io::stdout();

            while DAEMON_RUNNING.load(Ordering::Acquire) || !queue.is_empty() {
                let mut processed_any = false;

                while let Some(entry) = queue.pop() {
                    processed_any = true;

                    if let Ok(msg_str) = std::str::from_utf8(&entry.msg_buf[..entry.msg_len]) {
                        // Avoid allocations: print directly to raw output stream as structured JSON
                        let mut json_buf = [0u8; 256];
                        let mut pos = 0;
                        {
                            let mut writer = BufferWriter { buf: &mut json_buf, pos: 0 };

                            // Safely serialize structured JSON manually
                            let write_res = writeln!(
                                &mut writer,
                                "{{\"timestamp_ns\":{},\"level\":\"{}\",\"target\":\"{}\",\"message\":\"{}\"}}",
                                entry.timestamp_ns, entry.level, entry.target, msg_str
                            );

                            if write_res.is_ok() || writer.pos > 0 {
                                pos = writer.pos;
                            }
                        }

                        if pos > 0 {
                            let _ = stdout.write_all(&json_buf[..pos]);
                        }
                    }
                }

                if !processed_any {
                    // Back off when the queue is empty to reduce CPU usage
                    thread::sleep(Duration::from_millis(2));
                }
            }

            let _ = stdout.flush();
        });

        initialized = true;
    });

    initialized
}

/// Stops the background telemetry daemon cleanly.
pub fn stop_telemetry_daemon() {
    DAEMON_RUNNING.store(false, Ordering::Release);
}

/// Formats and pushes a log record onto the lock-free SpscQueue.
///
/// This function is safe to call from the performance hot path. It formats
/// strings directly onto the stack and copies bytes atomically.
pub fn log_raw(level: &'static str, target: &'static str, args: Arguments) {
    let queue = get_queue();

    // Allocate raw byte buffer on the stack
    let mut buf = [0u8; 128];
    let mut writer = BufferWriter {
        buf: &mut buf,
        pos: 0,
    };

    // Format directly into stack memory (handles truncation cleanly)
    let _ = std::fmt::write(&mut writer, args);

    let now_ns = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos() as u64,
        Err(_) => 0,
    };

    let entry = LogEntry {
        timestamp_ns: now_ns,
        level,
        target,
        msg_len: writer.pos,
        msg_buf: buf,
    };

    // Attempt non-blocking push. If queue is full, discard to protect performance invariants.
    let _ = queue.push(entry);
}
