// shared/src/types.rs

/// Maximum number of rotary encoders supported by the system.
pub const MAX_ENCODERS: usize = 8;

/// Maximum size in bytes for a serialized packet string payload.
pub const BUFFER_SIZE: usize = 128;

/// Represents an active reading of all encoder values.
#[derive(Debug, Clone, PartialEq)]
pub struct SensorDataPacket {
    /// A monotonically increasing sequence number for this packet.
    pub seq: u32,
    /// The accumulated array of 8 encoder values.
    pub encoders: [i32; MAX_ENCODERS],
}

/// Command to reset zero or more encoders on the device.
#[derive(Debug, Clone, PartialEq)]
pub struct ResetCommand {
    /// The target encoder ID (0-7), or 255 to mean "all".
    pub encoder_id: u8,
}

/// The top-level protocol message.
#[derive(Debug, Clone, PartialEq)]
pub enum Packet {
    /// Standard periodic broadcasting of sensor counts.
    SensorData(SensorDataPacket),
    /// Command requesting to reset active counters.
    Reset(ResetCommand),
    /// Diagnostic ping.
    Ping { timestamp: u32 },
    /// Diagnostic pong.
    Pong { timestamp: u32 },
}

impl SensorDataPacket {
    pub fn new(seq: u32, encoders: [i32; MAX_ENCODERS]) -> Self {
        Self { seq, encoders }
    }

    pub fn total_movement(&self) -> i32 {
        self.encoders.iter().map(|&x| x.abs()).sum()
    }

    pub fn has_movement(&self, previous: &SensorDataPacket) -> bool {
        self.encoders
            .iter()
            .zip(previous.encoders.iter())
            .any(|(curr, prev)| curr != prev)
    }
}

impl ResetCommand {
    pub fn single(encoder_id: u8) -> Self {
        Self { encoder_id }
    }

    pub fn all() -> Self {
        Self { encoder_id: 255 }
    }

    pub fn resets_all(&self) -> bool {
        self.encoder_id == 255
    }
}
