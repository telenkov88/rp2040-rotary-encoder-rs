//! Client library for reading the RP2040 rotary encoder states over UART.
//!
//! Provides a real-time, thread-safe view into the most recent count of all 8 axes.

use std::io::{BufRead, BufReader};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EncoderError {
    #[error("Failed to connect to serial port: {0}")]
    SerialPortError(#[from] serialport::Error),
    #[error("Failed to read from serial port: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse encoder output string")]
    ParseError,
}

/// A client for continuous background reading of the RP2040 8-axis encoder states.
pub struct EncoderClient {
    /// The current encoder counts across all eight axes.
    counts: Arc<RwLock<[i32; 8]>>,
    /// The current sequence number received from the device counter.
    sequence: Arc<RwLock<u32>>,
    #[allow(dead_code)]
    worker_handle: JoinHandle<()>,
}

impl EncoderClient {
    /// Starts retrieving encoder positions from the target serial device at 115,200 baud rate.
    pub fn spawn(port_name: &str) -> std::result::Result<Self, EncoderError> {
        let mut port = serialport::new(port_name, 115_200)
            .timeout(Duration::from_millis(100))
            .open()
            .or_else(|e| {
                if cfg!(target_os = "macos") {
                    // Fallback to baud rate 0 on macOS for virtual ports (e.g. socat pseudo-terminals)
                    serialport::new(port_name, 0)
                        .timeout(Duration::from_millis(100))
                        .open()
                } else {
                    Err(e)
                }
            })?;

        // For USB CDC ACM devices (like the RP2040), DTR must be asserted for the host
        // to receive any data stream.
        port.write_data_terminal_ready(true).ok();

        let counts = Arc::new(RwLock::new([0; 8]));
        let sequence = Arc::new(RwLock::new(0));

        let counts_clone = Arc::clone(&counts);
        let sequence_clone = Arc::clone(&sequence);

        let worker_handle = thread::spawn(move || {
            let mut reader = BufReader::new(port);
            let mut line = String::new();

            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(bytes_read) if bytes_read > 0 => {
                        let trimmed = line.trim_end();
                        if let Some((seq, parsed_encoders)) = parse_line(trimmed) {
                            if let Ok(mut c) = counts_clone.write() {
                                *c = parsed_encoders;
                            }
                            if let Ok(mut s) = sequence_clone.write() {
                                *s = seq;
                            }
                        } else {
                            eprintln!("Failed to parse UART text: '{}'", trimmed);
                        }
                    }
                    Ok(_) => {
                        eprintln!("UART EOF / disconnected.");
                        break;
                    }
                    Err(e) => {
                        if e.kind() != std::io::ErrorKind::TimedOut {
                            eprintln!("Encoder client reader error: {}", e);
                            break;
                        }
                    }
                }
            }
        });

        Ok(Self {
            counts,
            sequence,
            worker_handle,
        })
    }

    /// Gets a thread-safe atomic view of the latest polled 8 encoder orientations.
    pub fn get_counts(&self) -> [i32; 8] {
        if let Ok(c) = self.counts.read() {
            *c
        } else {
            [0; 8]
        }
    }

    /// Gets a thread-safe atomic view of the latest emitted packet sequence number.
    pub fn get_sequence(&self) -> u32 {
        if let Ok(s) = self.sequence.read() {
            *s
        } else {
            0
        }
    }
}

/// Helper method to cleanly extract the string format "$Seq:E0,E1,E2,E3,E4,E5,E6,E7*XX"
fn parse_line(line: &str) -> Option<(u32, [i32; 8])> {
    let start_idx = line.find('$')?;
    let slice = &line[start_idx + 1..];

    let star_idx = slice.find('*')?;
    let payload = &slice[..star_idx];

    let checksum_hex = slice.get(star_idx + 1..star_idx + 3)?;
    let expected_checksum = u8::from_str_radix(checksum_hex, 16).ok()?;

    let computed_checksum = payload.bytes().fold(0, |acc, b| acc ^ b);
    if computed_checksum != expected_checksum {
        return None;
    }

    let mut parts = payload.split(':');
    let seq_str = parts.next()?;
    let counts_str = parts.next()?;

    let seq: u32 = seq_str.parse().ok()?;

    let mut encoders = [0i32; 8];
    for (i, val_str) in counts_str.split(',').enumerate() {
        if i >= 8 {
            break;
        }
        encoders[i] = val_str.parse().ok()?;
    }

    Some((seq, encoders))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_line() {
        let line = "$123:1,-2,3,-4,5,-6,7,-8*2E";
        let (seq, counts) = parse_line(line).unwrap();
        assert_eq!(seq, 123);
        assert_eq!(counts, [1, -2, 3, -4, 5, -6, 7, -8]);
    }

    #[test]
    fn test_parse_corrupt_line() {
        assert!(parse_line("bad_data").is_none());
        assert!(parse_line("$123:0,1,2,abc,4,5,6,7*XX").is_none()); // Bad hex checksum
        assert!(parse_line("$123:0,1,2,3,4,5,6,7*00").is_none()); // Wrong checksum
        assert!(parse_line("123:0,1,2,3,4,5,6,7").is_none()); // Missing $ and *
    }
}
