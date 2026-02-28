//! Shared protocol definitions for the RP2040 rotary encoder project.
//!
//! Contains data structures and serialization logic used by both the firmware and host client.

#![cfg_attr(not(feature = "std"), no_std)]
pub mod types;
pub mod uart_protocol;

pub use types::*;
pub use uart_protocol::*;

pub const PACKET_SIZE: usize = 64;
pub const MAX_ENCODERS: usize = 8;

pub const PROTOCOL_VERSION: u8 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_serialization() {
        let original = SensorDataPacket {
            seq: 42,
            encoders: [1, -2, 3, -4, 5, -6, 7, -8],
        };
        let packet = Packet::SensorData(original);

        let serialized = serialize_packet(&packet);
        assert_eq!(serialized.as_str(), "$42:1,-2,3,-4,5,-6,7,-8*18\n");
    }
}
