use crate::types::{Packet, BUFFER_SIZE};
use core::fmt::Write;
use heapless::String;

pub fn serialize_packet(packet: &Packet) -> String<BUFFER_SIZE> {
    let mut buf = String::new();
    match packet {
        Packet::SensorData(data) => {
            let _ = write!(
                &mut buf,
                "{}:{},{},{},{},{},{},{},{}\n",
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
            let _ = write!(&mut buf, "RST:{}\n", cmd.encoder_id);
        }
        Packet::Ping { timestamp } => {
            let _ = write!(&mut buf, "PING:{}\n", timestamp);
        }
        Packet::Pong { timestamp } => {
            let _ = write!(&mut buf, "PONG:{}\n", timestamp);
        }
    }
    buf
}

pub fn create_sensor_packet(seq: u32, encoders: [i32; 8]) -> Packet {
    use crate::types::SensorDataPacket;
    Packet::SensorData(SensorDataPacket::new(seq, encoders))
}

pub fn create_reset_packet(encoder_id: u8) -> Packet {
    use crate::types::ResetCommand;
    Packet::Reset(ResetCommand { encoder_id })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn test_serialize_sensor_data() {
        let original = SensorDataPacket::new(123, [1, -2, 3, -4, 5, -6, 7, -8]);
        let packet = Packet::SensorData(original);

        let serialized = serialize_packet(&packet);
        assert_eq!(serialized.as_str(), "123:1,-2,3,-4,5,-6,7,-8\n");
    }

    #[test]
    fn test_serialize_reset_command() {
        let packet = Packet::Reset(ResetCommand::single(3));

        let serialized = serialize_packet(&packet);
        assert_eq!(serialized.as_str(), "RST:3\n");
    }
}
