use crate::types::{BUFFER_SIZE, Packet};
use core::fmt::Write;
use heapless::String;

/// Computes an XOR checksum of the ASCII payload string.
pub fn compute_checksum(payload: &str) -> u8 {
    payload.bytes().fold(0, |acc, b| acc ^ b)
}

/// Serializes a Packet enum into a heapless NMEA-framed string.
pub fn serialize_packet(packet: &Packet) -> String<BUFFER_SIZE> {
    let mut payload: String<BUFFER_SIZE> = String::new();
    match packet {
        Packet::SensorData(data) => {
            let _ = write!(
                &mut payload,
                "{}:{},{},{},{},{},{},{},{}",
                data.seq,
                data.encoders[0],
                data.encoders[1],
                data.encoders[2],
                data.encoders[3],
                data.encoders[4],
                data.encoders[5],
                data.encoders[6],
                data.encoders[7],
            );
        }
        Packet::Reset(cmd) => {
            let _ = write!(&mut payload, "RST:{}", cmd.encoder_id);
        }
        Packet::Ping { timestamp } => {
            let _ = write!(&mut payload, "PING:{}", timestamp);
        }
        Packet::Pong { timestamp } => {
            let _ = write!(&mut payload, "PONG:{}", timestamp);
        }
    }

    let checksum = compute_checksum(&payload);

    let mut buf: String<BUFFER_SIZE> = String::new();
    let _ = writeln!(&mut buf, "${}*{:02X}", payload, checksum);
    buf
}

/// Utility to quickly mint a new SensorData packet.
pub fn create_sensor_packet(seq: u32, encoders: [i32; 8]) -> Packet {
    use crate::types::SensorDataPacket;
    Packet::SensorData(SensorDataPacket::new(seq, encoders))
}

/// Utility to quickly mint a new ResetCommand packet.
pub fn create_reset_packet(encoder_id: u8) -> Packet {
    use crate::types::ResetCommand;
    Packet::Reset(ResetCommand { encoder_id })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn test_compute_checksum() {
        assert_eq!(compute_checksum("123:1,-2,3,-4,5,-6,7,-8"), 0x2E);
        assert_eq!(compute_checksum("RST:3"), 0x5C);
    }

    #[test]
    fn test_serialize_sensor_data() {
        let original = SensorDataPacket::new(123, [1, -2, 3, -4, 5, -6, 7, -8]);
        let packet = Packet::SensorData(original);

        let serialized = serialize_packet(&packet);
        assert_eq!(serialized.as_str(), "$123:1,-2,3,-4,5,-6,7,-8*2E\n");
    }

    #[test]
    fn test_serialize_reset_command() {
        let packet = Packet::Reset(ResetCommand::single(3));

        let serialized = serialize_packet(&packet);
        assert_eq!(serialized.as_str(), "$RST:3*5C\n");
    }
}
