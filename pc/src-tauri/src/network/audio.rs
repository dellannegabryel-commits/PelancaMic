//! UDP Audio Receiver

use std::collections::VecDeque;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use crate::protocol::*;
use crate::protocol::frame::{AudioFrame, AudioFrameHeader};

const MAX_BUFFER_FRAMES: usize = 16;
const TARGET_BUFFER_FRAMES: usize = 5;
const MIN_BUFFER_FRAMES: usize = 3;
const MAX_BUFFER_FRAMES_ADAPTIVE: usize = 8;

pub struct AudioReceiver {
    socket: UdpSocket,
    buffer: Arc<Mutex<VecDeque<AudioFrame>>>,
    last_sequence: Arc<Mutex<Option<u32>>>,
    stats: Arc<Mutex<AudioStats>>,
    target_buffer_frames: Arc<Mutex<usize>>,
}

#[derive(Debug, Default, Clone)]
pub struct AudioStats {
    pub packets_received: u64,
    pub packets_lost: u64,
    pub packets_out_of_order: u64,
    pub last_jitter_ms: f32,
    pub buffer_health_percent: f32,
}

impl AudioReceiver {
    pub fn new(bind_addr: SocketAddr) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(bind_addr)?;
        socket.set_read_timeout(Some(Duration::from_millis(10)))?;
        socket.set_nonblocking(true)?;

        Ok(Self {
            socket,
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_BUFFER_FRAMES))),
            last_sequence: Arc::new(Mutex::new(None)),
            stats: Arc::new(Mutex::new(AudioStats::default())),
            target_buffer_frames: Arc::new(Mutex::new(TARGET_BUFFER_FRAMES)),
        })
    }

    pub fn start_receive_loop(self: Arc<Self>) {
        thread::spawn(move || {
            let mut buf = [0u8; FRAME_SIZE];
            loop {
                match self.socket.recv_from(&mut buf) {
                    Ok((len, src)) => {
                        if len >= HEADER_SIZE {
                            self.process_packet(&buf[..len], src);
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_micros(100));
                    }
                    Err(e) => {
                        tracing::error!("UDP recv error: {}", e);
                        thread::sleep(Duration::from_millis(1));
                    }
                }
            }
        });
    }

    fn process_packet(&self, data: &[u8], src: SocketAddr) {
        if let Some(frame) = AudioFrame::decode(data) {
            let mut stats = self.stats.lock().unwrap();
            stats.packets_received += 1;

            let mut last_seq = self.last_sequence.lock().unwrap();
            if let Some(last) = *last_seq {
                let expected = last.wrapping_add(1);
                if frame.header.sequence != expected {
                    if frame.header.sequence > expected {
                        stats.packets_lost += (frame.header.sequence - expected) as u64;
                    } else {
                        stats.packets_out_of_order += 1;
                    }
                }
            }
            *last_seq = Some(frame.header.sequence);

            let mut buffer = self.buffer.lock().unwrap();
            if buffer.len() < MAX_BUFFER_FRAMES {
                buffer.push_back(frame);
            } else {
                stats.packets_lost += 1;
            }

            self.adapt_buffer_target(buffer.len());
            self.update_stats(&mut stats, buffer.len());
        }
    }

    fn adapt_buffer_target(&self, current_len: usize) {
        let mut target = self.target_buffer_frames.lock().unwrap();
        if current_len < MIN_BUFFER_FRAMES {
            *target = (*target + 1).min(MAX_BUFFER_FRAMES_ADAPTIVE);
        } else if current_len > MAX_BUFFER_FRAMES_ADAPTIVE {
            *target = (*target.saturating_sub(1)).max(MIN_BUFFER_FRAMES);
        }
    }

    fn update_stats(&self, stats: &mut AudioStats, buffer_len: usize) {
        let target = *self.target_buffer_frames.lock().unwrap();
        stats.buffer_health_percent = (buffer_len as f32 / target as f32 * 100.0).min(100.0);
    }

    pub fn pop_frame(&self) -> Option<AudioFrame> {
        self.buffer.lock().unwrap().pop_front()
    }

    pub fn get_stats(&self) -> AudioStats {
        self.stats.lock().unwrap().clone()
    }

    pub fn buffer_len(&self) -> usize {
        self.buffer.lock().unwrap().len()
    }

    pub fn target_buffer_frames(&self) -> usize {
        *self.target_buffer_frames.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddr};

    #[test]
    fn test_receiver_creation() {
        let addr: SocketAddr = (Ipv4Addr::UNSPECIFIED, 0).into();
        let receiver = AudioReceiver::new(addr);
        assert!(receiver.is_ok());
    }
}