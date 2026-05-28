//! ANSI Terminal Renderer
//!
//! Provides a lightweight, cross-platform terminal UI engine using raw ANSI
//! escape sequences. No external crates required. Renders the live dashboard
//! for the Platform Nodes resource monitor.

use crate::telemetry::TelemetrySnapshot;
use std::io::{self, Write};

// ── ANSI Escape Code Constants ───────────────────────────────────────────────

const ESC: &str = "\x1b[";

/// Foreground color codes (standard 16-color palette).
/// All constants are declared for the public API surface; callers may use any subset.
#[allow(dead_code)]
pub mod fg {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const BLACK: &str = "\x1b[30m";
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";
    pub const BRIGHT_WHITE: &str = "\x1b[97m";
    pub const BRIGHT_CYAN: &str = "\x1b[96m";
    pub const BRIGHT_GREEN: &str = "\x1b[92m";
    pub const BRIGHT_RED: &str = "\x1b[91m";
    pub const BRIGHT_YELLOW: &str = "\x1b[93m";
}

/// Background color codes.
#[allow(dead_code)]
pub mod bg {
    pub const RESET: &str = "\x1b[49m";
    pub const BLUE: &str = "\x1b[44m";
    pub const DARK_GRAY: &str = "\x1b[100m";
    pub const BRIGHT_BLACK: &str = "\x1b[40m";
}

// ── Terminal Control ──────────────────────────────────────────────────────────

/// Moves the cursor to `(row, col)` (1-indexed).
#[allow(dead_code)]
pub fn goto(row: u16, col: u16) -> String {
    format!("{}{};{}H", ESC, row, col)
}

/// Clears the entire terminal screen and moves cursor to top-left.
pub fn clear_screen() -> String {
    format!("{}2J{}H", ESC, ESC)
}

/// Hides the terminal cursor (reduces flicker during re-renders).
pub fn hide_cursor() -> String {
    format!("{}?25l", ESC)
}

/// Shows the terminal cursor again.
pub fn show_cursor() -> String {
    format!("{}?25h", ESC)
}

/// Clears from the cursor to end of the current line.
#[allow(dead_code)]
pub fn clear_eol() -> String {
    format!("{}K", ESC)
}

// ── Layout Helpers ────────────────────────────────────────────────────────────

/// Draws a horizontal box-drawing border of given width.
pub fn hline(width: usize, ch: char) -> String {
    std::iter::repeat_n(ch, width).collect()
}

/// Creates a labeled section header.
#[allow(dead_code)]
pub fn section_header(label: &str, width: usize) -> String {
    let _bar = hline(width, '─');
    let title = format!(" {} ", label);
    let pad = if width > title.len() + 2 {
        hline(width - title.len() - 2, '─')
    } else {
        String::new()
    };
    format!("├{}{}{}┤", title, pad, "")
        .chars()
        .take(width)
        .collect()
}

/// Right-pads a string to the given width with spaces.
pub fn pad_right(s: &str, width: usize) -> String {
    if s.len() >= width {
        s[..width].to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - s.len()))
    }
}

/// Renders a simple ASCII progress bar like `[████████░░░░] 65%`.
pub fn progress_bar(value: u64, max: u64, width: usize) -> String {
    let ratio = if max == 0 {
        0.0
    } else {
        (value as f64 / max as f64).min(1.0)
    };
    let filled = (ratio * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    let bar: String = "█".repeat(filled);
    let void: String = "░".repeat(empty);
    let pct = (ratio * 100.0) as u64;
    format!("[{}{}] {:>3}%", bar, void, pct)
}

// ── Dashboard Renderer ────────────────────────────────────────────────────────

const PANEL_WIDTH: usize = 60;
const PANEL_INNER: usize = PANEL_WIDTH - 4; // inside borders

/// Renders a single row inside a panel.
#[allow(dead_code)]
fn panel_row(content: &str) -> String {
    format!(
        "│ {}{} │",
        content,
        " ".repeat(PANEL_INNER.saturating_sub(content.len()))
    )
}

/// Strips ANSI codes for length calculation.
fn visible_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for ch in s.chars() {
        if ch == '\x1b' {
            in_escape = true;
        } else if in_escape && ch == 'm' {
            in_escape = false;
        } else if !in_escape {
            len += 1;
        }
    }
    len
}

/// Renders a full-screen dashboard given a telemetry snapshot (or None when offline).
/// Writes directly to `stdout`.
pub fn render(snapshot: &Option<TelemetrySnapshot>, tick: u64) {
    let mut out = String::with_capacity(2048);

    // Clear and position cursor at top-left
    out.push_str(&clear_screen());

    let top_border = format!("╔{}╗", hline(PANEL_WIDTH - 2, '═'));
    let bot_border = format!("╚{}╝", hline(PANEL_WIDTH - 2, '═'));
    let mid_border = format!("├{}┤", hline(PANEL_WIDTH - 2, '─'));

    // ── Header ─────────────────────────────────────────────────────────────
    out.push_str(&format!("{}{}{}", fg::BRIGHT_CYAN, fg::BOLD, top_border));
    out.push_str("\r\n");

    let title_str = "  Platform Nodes  ·  Resource Monitor  ";
    let title_padded = pad_right(title_str, PANEL_WIDTH - 2);
    out.push_str(&format!("║{}║\r\n", title_padded));

    let version_str = format!("  v0.1.0  ·  tick #{:<6}", tick);
    let version_padded = pad_right(&version_str, PANEL_WIDTH - 2);
    out.push_str(&format!("{}║{}{}║\r\n", fg::DIM, version_padded, fg::RESET));

    out.push_str(&format!(
        "{}{}{}\r\n",
        fg::BRIGHT_CYAN,
        fg::BOLD,
        mid_border
    ));

    // ── Connection Status ──────────────────────────────────────────────────
    let conn_label = "  CONNECTION";
    let conn_padded = pad_right(conn_label, PANEL_WIDTH - 2);
    out.push_str(&format!("{}║{}║\r\n", fg::DIM, conn_padded));
    out.push_str(fg::RESET);

    let (status_color, status_text, status_icon) = match snapshot {
        Some(s) if s.status == "ACTIVE" => (fg::BRIGHT_GREEN, "ACTIVE", "●"),
        Some(_) => (fg::BRIGHT_YELLOW, "DEGRADED", "◕"),
        None => (fg::BRIGHT_RED, "OFFLINE", "○"),
    };

    let conn_row = format!(
        "{}{}  {} {}  platform-nodes @ 127.0.0.1:8080{}",
        fg::BOLD,
        status_color,
        status_icon,
        status_text,
        fg::RESET
    );
    let raw_row = format!(
        "  {} {}  platform-nodes @ 127.0.0.1:8080",
        status_icon, status_text
    );
    let padding = " ".repeat(PANEL_INNER.saturating_sub(visible_len(&raw_row)));
    out.push_str(&format!(
        "{}│ {}{}│\r\n",
        fg::BRIGHT_CYAN,
        conn_row,
        padding
    ));

    out.push_str(&format!(
        "{}{}{}\r\n",
        fg::BRIGHT_CYAN,
        mid_border,
        fg::RESET
    ));

    // ── SWIM Gossip Panel ──────────────────────────────────────────────────
    let swim_label = "  SWIM GOSSIP CONSENSUS";
    let swim_padded = pad_right(swim_label, PANEL_WIDTH - 2);
    out.push_str(&format!(
        "{}{}║{}║\r\n",
        fg::BRIGHT_CYAN,
        fg::DIM,
        swim_padded
    ));
    out.push_str(fg::RESET);

    match snapshot {
        Some(snap) => {
            let peers_row = format!(
                "{}{}  Cluster Peers{}   {}{}{}{}",
                fg::BOLD,
                fg::WHITE,
                fg::RESET,
                fg::BRIGHT_CYAN,
                fg::BOLD,
                snap.swim_peers,
                fg::RESET
            );
            let peers_raw = format!("  Cluster Peers   {}", snap.swim_peers);
            let peers_pad = " ".repeat(PANEL_INNER.saturating_sub(visible_len(&peers_raw)));
            out.push_str(&format!(
                "{}│ {}{}│\r\n",
                fg::BRIGHT_CYAN,
                peers_row,
                peers_pad
            ));

            let liveness = if snap.swim_peers > 0 {
                "Cluster is healthy"
            } else {
                "No peers registered"
            };
            let liveness_color = if snap.swim_peers > 0 {
                fg::BRIGHT_GREEN
            } else {
                fg::BRIGHT_YELLOW
            };
            let live_row = format!("{}{}  {}{}", fg::BOLD, liveness_color, liveness, fg::RESET);
            let live_raw = format!("  {}", liveness);
            let live_pad = " ".repeat(PANEL_INNER.saturating_sub(visible_len(&live_raw)));
            out.push_str(&format!(
                "{}│ {}{}│\r\n",
                fg::BRIGHT_CYAN,
                live_row,
                live_pad
            ));
        }
        None => {
            out.push_str(&format!(
                "{}│{}{}│\r\n",
                fg::BRIGHT_CYAN,
                pad_right("  --", PANEL_WIDTH - 2),
                ""
            ));
            out.push_str(&format!(
                "{}│{}│\r\n",
                fg::BRIGHT_CYAN,
                pad_right("", PANEL_WIDTH - 2)
            ));
        }
    }

    out.push_str(&format!(
        "{}{}{}\r\n",
        fg::BRIGHT_CYAN,
        mid_border,
        fg::RESET
    ));

    // ── LSM Storage Panel ──────────────────────────────────────────────────
    let lsm_label = "  LSM STORAGE ENGINE";
    let lsm_padded = pad_right(lsm_label, PANEL_WIDTH - 2);
    out.push_str(&format!(
        "{}{}║{}║\r\n",
        fg::BRIGHT_CYAN,
        fg::DIM,
        lsm_padded
    ));
    out.push_str(fg::RESET);

    match snapshot {
        Some(snap) => {
            let sst_row = format!(
                "{}{}  SSTables on disk{}  {}{}{}{}",
                fg::BOLD,
                fg::WHITE,
                fg::RESET,
                fg::BRIGHT_CYAN,
                fg::BOLD,
                snap.lsm_sstables,
                fg::RESET
            );
            let sst_raw = format!("  SSTables on disk  {}", snap.lsm_sstables);
            let sst_pad = " ".repeat(PANEL_INNER.saturating_sub(visible_len(&sst_raw)));
            out.push_str(&format!("{}│ {}{}│\r\n", fg::BRIGHT_CYAN, sst_row, sst_pad));

            // Utilisation bar: compaction triggers at 4 files
            let bar = progress_bar(snap.lsm_sstables, 4, 24);
            let bar_row = format!(
                "{}  Compaction load {}{}{}",
                fg::DIM,
                fg::BRIGHT_YELLOW,
                bar,
                fg::RESET
            );
            let bar_raw = format!("  Compaction load {}", bar);
            let bar_pad = " ".repeat(PANEL_INNER.saturating_sub(visible_len(&bar_raw)));
            out.push_str(&format!("{}│ {}{}│\r\n", fg::BRIGHT_CYAN, bar_row, bar_pad));
        }
        None => {
            out.push_str(&format!(
                "{}│{}│\r\n",
                fg::BRIGHT_CYAN,
                pad_right("  --", PANEL_WIDTH - 2)
            ));
            out.push_str(&format!(
                "{}│{}│\r\n",
                fg::BRIGHT_CYAN,
                pad_right("", PANEL_WIDTH - 2)
            ));
        }
    }

    // ── Footer ─────────────────────────────────────────────────────────────
    out.push_str(&format!(
        "{}{}{}\r\n",
        fg::BRIGHT_CYAN,
        bot_border,
        fg::RESET
    ));

    let footer = "  [q] Quit  ·  [r] Force Refresh  ·  Polling every 1s";
    let footer_padded = pad_right(footer, PANEL_WIDTH);
    out.push_str(&format!(
        "{}{}  {}{}\r\n",
        fg::DIM,
        fg::WHITE,
        footer_padded,
        fg::RESET
    ));

    // Flush all at once to minimize flicker
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = handle.write_all(out.as_bytes());
    let _ = handle.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_zero() {
        let bar = progress_bar(0, 4, 20);
        assert!(bar.contains("░"));
        assert!(bar.contains("0%") || bar.contains("  0%"));
    }

    #[test]
    fn test_progress_bar_full() {
        let bar = progress_bar(4, 4, 20);
        assert!(bar.contains("100%"));
    }

    #[test]
    fn test_progress_bar_partial() {
        let bar = progress_bar(2, 4, 20);
        assert!(bar.contains("50%"));
    }

    #[test]
    fn test_pad_right_pads() {
        let s = pad_right("hi", 10);
        assert_eq!(s.len(), 10);
        assert!(s.starts_with("hi"));
    }

    #[test]
    fn test_pad_right_truncates() {
        let s = pad_right("hello world!", 5);
        assert_eq!(s, "hello");
    }
}
