//! Audio frame parsing and serialization

use bytes::{Buf, BufMut, BytesMut};
use crate::protocol::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioFrameHeader {
    pub magic: u16,
    pub version: u8,
    pub flags: u8,
    pub sequence: u32,
    pub timestamp: u32,
    pub payload_length: u16,
    pub channels: u8,
    pub codec: u8,
}

impl AudioFrameHeader {
    pub const SIZE: usize = HEADER_SIZE;

    pub fn new(sequence: u32, timestamp: u32, flags: u8) -> Self {
        Self {
            magic: MAGIC,
            version: VERSION,
            flags,
            sequence,
            timestamp,
            payload_length: PAYLOAD_SIZE as u16,
            channels: CHANNELS,
            codec: codec::PCM,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == MAGIC && self.version == VERSION
    }

    pub fn is_muted(&self) -> bool {
        (self.flags & flags::MUTED) != 0
    }

    pub fn is_silence(&self) -> bool {
        (self.flags & flags::SILENCE) != 0
    }

    pub fn encode(&self, buf: &mut BytesMut) {
        buf.put_u16(self.magic);
        buf.put_u8(self.version);
        buf.put_u8(self.flags);
        buf.put_u32(self.sequence);
        buf.put_u32(self.timestamp);
        buf.put_u16(self.payload_length);
        buf.put_u8(self.channels);
        buf.put_u8(self.codec);
    }

    pub fn decode(buf: &mut BytesMut) -> Option<Self> {
        if buf.remaining() < HEADER_SIZE {
            return None;
        }
        let magic = buf.get_u16();
        let version = buf.get_u8();
        let flags = buf.get_u8();
        let sequence = buf.get_u32();
        let timestamp = buf.get_u32();
        let payload_length = buf.get_u16();
        let channels = buf.get_u8();
        let codec = buf.get_u8();

        let header = Self {
            magic,
            version,
            flags,
            sequence,
            timestamp,
            payload_length,
            channels,
            codec,
        };

        if header.is_valid() {
            Some(header)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct AudioFrame {
    pub header: AudioFrameHeader,
    pub payload: Vec<u8>,
}

impl AudioFrame {
    pub fn new(sequence: u32, timestamp: u32, payload: Vec<u8>, flags: u8) -> Self {
        let header = AudioFrameHeader::new(sequence, timestamp, flags);
        Self { header, payload }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(FRAME_SIZE);
        self.header.encode(&mut buf);
        buf.extend_from_slice(&self.payload);
        buf.to_vec()
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < HEADER_SIZE {
            return None;
        }
        let mut buf = BytesMut::from(data);
        let header = AudioFrameHeader::decode(&mut buf)?;
        let payload = buf.to_vec();
        Some(Self { header, payload })
    }

    pub fn payload_as_i16(&self) -> Vec<i16> {
        self.payload
            .chunks_exact(2)
            .map(|chunk| i16::from_be_bytes([chunk[0], chunk[1]]))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_encode_decode() {
        let header = AudioFrameHeader::new(42, 12345, 0);
        let mut buf = BytesMut::new();
        header.encode(&mut buf);
        assert_eq!(buf.len(), HEADER_SIZE);

        let decoded = AudioFrameHeader::decode(&mut buf).unwrap();
        assert_eq!(header, decoded);
        assert!(decoded.is_valid());
        assert!(!decoded.is_muted());
        assert!(!decoded.is_silence());
    }

    #[test]
    fn test_header_flags() {
        let mut header = AudioFrameHeader::new(1, 0, flags::MUTED);
        assert!(header.is_muted());
        assert!(!header.is_silence());

        header.flags = flags::SILENCE;
        assert!(!header.is_muted());
        assert!(header.is_silence());

        header.flags = flags::MUTED | flags::SILENCE;
        assert!(header.is_muted());
        assert!(header.is_silence());
    }

    #[test]
    fn test_frame_encode_decode() {
        let payload = vec![0u8; PAYLOAD_SIZE];
        let frame = AudioFrame::new(100, 5000, payload.clone(), 0);
        let encoded = frame.encode();
        assert_eq!(encoded.len(), FRAME_SIZE);

        let decoded = AudioFrame::decode(&encoded).unwrap();
        assert_eq!(decoded.header.sequence, 100);
        assert_eq!(decoded.header.timestamp, 5000);
        assert_eq!(decoded.payload, payload);
    }

    #[test]
    fn test_invalid_magic() {
        let mut buf = BytesMut::new();
        buf.put_u16(0xFFFF); // Invalid magic
        buf.put_u8(VERSION);
        buf.put_u8(0);
        buf.put_u32(1);
        buf.put_u32(0);
        buf.put_u16(PAYLOAD_SIZE as u16);
        buf.put_u8(CHANNELS);
        buf.put_u8(codec::PCM);
        buf.extend_from_slice(&vec![0u8; PAYLOAD_SIZE]);

        let result = AudioFrame::decode(&buf);
        assert!(result.is_none());
    }
}