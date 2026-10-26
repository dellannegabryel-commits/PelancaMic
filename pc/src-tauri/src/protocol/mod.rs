//! Protocol constants shared between Android and PC

/// Magic bytes for PelancaMic audio frames (0x4D53 = "MS")
pub const MAGIC: u16 = 0x4D53;

/// Protocol version
pub const VERSION: u8 = 1;

/// Audio frame header size in bytes
pub const HEADER_SIZE: usize = 16;

/// Audio payload size in bytes (480 samples * 2 bytes per sample = 960 bytes)
pub const PAYLOAD_SIZE: usize = 960;

/// Total frame size (header + payload)
pub const FRAME_SIZE: usize = HEADER_SIZE + PAYLOAD_SIZE;

/// Sample rate in Hz
pub const SAMPLE_RATE: u32 = 48_000;

/// Number of channels (mono)
pub const CHANNELS: u8 = 1;

/// Bits per sample
pub const BITS_PER_SAMPLE: u16 = 16;

/// Frame duration in milliseconds
pub const FRAME_DURATION_MS: u32 = 10;

/// Samples per frame
pub const SAMPLES_PER_FRAME: usize = 480;

/// Default ports
pub const DISCOVERY_PORT: u16 = 41234;
pub const AUDIO_PORT: u16 = 41235;
pub const CONTROL_PORT: u16 = 41236;

/// Discovery beacon interval in milliseconds
pub const BEACON_INTERVAL_MS: u64 = 1000;

/// Flag bit positions
pub mod flags {
    pub const MUTED: u8 = 0x01;
    pub const SILENCE: u8 = 0x02;
}

/// Codec identifiers
pub mod codec {
    pub const PCM: u8 = 0;
}

/// Control message types
pub mod control {
    pub const HELLO: &str = "hello";
    pub const MUTE: &str = "mute";
    pub const VOLUME: &str = "volume";
    pub const BYE: &str = "bye";
    pub const PING: &str = "ping";
    pub const PONG: &str = "pong";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(MAGIC, 0x4D53);
        assert_eq!(VERSION, 1);
        assert_eq!(HEADER_SIZE, 16);
        assert_eq!(PAYLOAD_SIZE, 960);
        assert_eq!(FRAME_SIZE, 976);
        assert_eq!(SAMPLE_RATE, 48_000);
        assert_eq!(FRAME_DURATION_MS, 10);
        assert_eq!(SAMPLES_PER_FRAME, 480);
    }
}