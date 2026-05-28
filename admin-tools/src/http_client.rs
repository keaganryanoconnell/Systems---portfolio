//! Zero-Dependency HTTP/1.1 Telemetry Client
//!
//! Fetches the JSON telemetry payload from the platform-nodes HTTP proxy
//! using a raw `TcpStream`. No external crates required.

use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// Result of a telemetry fetch: either the raw response body or an I/O error.
pub enum FetchResult {
    /// Successfully retrieved response body.
    Ok(String),
    /// Connection failed or timed out.
    Unavailable(String),
}

impl FetchResult {
    /// Returns the inner message for logging or display purposes.
    pub fn message(&self) -> &str {
        match self {
            FetchResult::Ok(s) | FetchResult::Unavailable(s) => s.as_str(),
        }
    }
}

/// Connects to `host:port`, sends a minimal HTTP GET, and returns the response body.
pub fn fetch_telemetry(host: &str, port: u16, path: &str) -> FetchResult {
    let addr = format!("{}:{}", host, port);

    let stream = match TcpStream::connect(&addr) {
        Ok(s) => s,
        Err(e) => return FetchResult::Unavailable(format!("Connect failed: {}", e)),
    };

    if let Err(e) = stream.set_read_timeout(Some(Duration::from_secs(1))) {
        return FetchResult::Unavailable(format!("Timeout set failed: {}", e));
    }

    let mut stream = stream;
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        path, host
    );

    if let Err(e) = stream.write_all(request.as_bytes()) {
        return FetchResult::Unavailable(format!("Write failed: {}", e));
    }

    let mut raw = String::new();
    if let Err(e) = stream.read_to_string(&mut raw) {
        // A timeout on read is treated as unavailable
        if e.kind() != io::ErrorKind::TimedOut && e.kind() != io::ErrorKind::WouldBlock {
            return FetchResult::Unavailable(format!("Read failed: {}", e));
        }
    }

    // Split HTTP headers from body at the blank line separator
    if let Some(body_start) = raw.find("\r\n\r\n") {
        FetchResult::Ok(raw[body_start + 4..].trim().to_string())
    } else {
        FetchResult::Unavailable("Malformed HTTP response: no header separator".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn test_fetch_parses_body_correctly() {
        // Spin up a minimal mock HTTP server on a random port
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 512];
                let _ = stream.read(&mut buf);
                let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 30\r\nConnection: close\r\n\r\n{\"status\":\"ACTIVE\",\"peers\":2}";
                let _ = stream.write_all(response.as_bytes());
            }
        });

        // Give the thread time to bind
        std::thread::sleep(std::time::Duration::from_millis(20));

        let result = fetch_telemetry("127.0.0.1", port, "/telemetry");
        match result {
            FetchResult::Ok(body) => {
                assert!(body.contains("ACTIVE"));
                assert!(body.contains("peers"));
            }
            FetchResult::Unavailable(reason) => panic!("Expected Ok, got unavailable: {}", reason),
        }
    }

    #[test]
    fn test_fetch_returns_unavailable_when_no_server() {
        // Port 1 is virtually guaranteed to be refused
        let result = fetch_telemetry("127.0.0.1", 1, "/telemetry");
        assert!(matches!(result, FetchResult::Unavailable(_)));
    }
}
