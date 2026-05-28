//! Admin Tools - Platform Nodes Resource Monitor
//!
//! A zero-dependency, cross-platform terminal dashboard that polls the
//! platform-nodes HTTP telemetry endpoint and renders live SWIM consensus
//! and LSM storage metrics using raw ANSI escape sequences.

mod http_client;
mod telemetry;
mod tui;

use http_client::{fetch_telemetry, FetchResult};
use telemetry::TelemetrySnapshot;

use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Global shutdown flag; set true to stop all threads cleanly.
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

fn main() {
    // Print startup banner and hide the cursor for clean rendering
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = write!(handle, "{}", tui::hide_cursor());
    let _ = handle.flush();
    drop(handle);

    // Register Ctrl-C handler: set the shutdown flag
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown_flag.clone();

    // Shared slot: latest telemetry snapshot and tick counter
    let snapshot: Arc<Mutex<Option<TelemetrySnapshot>>> = Arc::new(Mutex::new(None));
    let tick: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));

    let snapshot_poll = snapshot.clone();
    let tick_poll = tick.clone();

    // ── Background Poll Thread ─────────────────────────────────────────────
    let poll_handle = std::thread::spawn(move || {
        while !shutdown_clone.load(Ordering::Acquire) {
            let result = fetch_telemetry("127.0.0.1", 8080, "/telemetry");
            let parsed = match &result {
                FetchResult::Ok(body) => TelemetrySnapshot::parse(body),
                FetchResult::Unavailable(_) => {
                    // message() surfaces the error string for diagnostics
                    let _ = result.message();
                    None
                }
            };

            if let Ok(mut snap) = snapshot_poll.lock() {
                *snap = parsed;
            }
            if let Ok(mut t) = tick_poll.lock() {
                *t = t.wrapping_add(1);
            }

            // Sleep 1 second between polls, checking shutdown every 50ms
            let mut elapsed = 0u64;
            while elapsed < 1000 {
                std::thread::sleep(Duration::from_millis(50));
                elapsed += 50;
                if shutdown_clone.load(Ordering::Acquire) {
                    break;
                }
            }
        }
    });

    // ── Main Render Loop ───────────────────────────────────────────────────
    // We use non-blocking stdin reads to detect 'q' key press
    // without pulling in any terminal crate.
    //
    // On Windows & Unix, stdin in non-raw mode reads line-buffered,
    // so we spawn a reader thread and use a shared flag.
    let input_shutdown = shutdown_flag.clone();
    let _input_handle = std::thread::spawn(move || {
        let stdin = io::stdin();
        let mut buf = [0u8; 1];
        loop {
            if input_shutdown.load(Ordering::Acquire) {
                break;
            }
            if stdin.lock().read(&mut buf).is_ok()
                && (buf[0] == b'q' || buf[0] == b'Q' || buf[0] == 3)
            {
                input_shutdown.store(true, Ordering::Release);
                SHUTDOWN.store(true, Ordering::Release);
                break;
            }
        }
    });

    let mut last_tick = u64::MAX;

    loop {
        if shutdown_flag.load(Ordering::Acquire) || SHUTDOWN.load(Ordering::Acquire) {
            break;
        }

        let current_tick = tick.lock().map_or(0, |t| *t);
        if current_tick != last_tick {
            last_tick = current_tick;
            let snap = snapshot.lock().map_or(None, |s| s.clone());
            tui::render(&snap, current_tick);
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    // ── Cleanup ────────────────────────────────────────────────────────────
    // Signal poll thread to exit
    shutdown_flag.store(true, Ordering::Release);
    SHUTDOWN.store(true, Ordering::Release);

    // Restore terminal: show cursor, clear screen, reset colors
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = write!(
        handle,
        "{}{}{}Goodbye.\r\n",
        tui::show_cursor(),
        tui::clear_screen(),
        tui::fg::RESET
    );
    let _ = handle.flush();

    let _ = poll_handle.join();
}
