//! Adaptive Jitter Buffer

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use crate::protocol::frame::AudioFrame;

const MIN_BUFFER_FRAMES: usize = 3;
const MAX_BUFFER_FRAMES: usize = 8;
const TARGET_BUFFER_FRAMES: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq)]
enum BufferState {
    Filling,
    Stable,
    Draining,
    Underrun,
}

pub struct JitterBuffer {
    buffer: VecDeque<AudioFrame>,
    target_frames: usize,
    state: BufferState,
    last_pop: Option<Instant>,
    consecutive_underruns: u32,
    consecutive_overruns: u32,
}

impl JitterBuffer {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::with_capacity(MAX_BUFFER_FRAMES),
            target_frames: TARGET_BUFFER_FRAMES,
            state: BufferState::Filling,
            last_pop: None,
            consecutive_underruns: 0,
            consecutive_overruns: 0,
        }
    }

    pub fn push(&mut self, frame: AudioFrame) -> Result<(), JitterBufferError> {
        if self.buffer.len() >= MAX_BUFFER_FRAMES {
            self.consecutive_overruns += 1;
            self.consecutive_underruns = 0;
            return Err(JitterBufferError::BufferFull);
        }

        let insert_pos = self.buffer
            .binary_search_by_key(&frame.header.sequence, |f| f.header.sequence)
            .unwrap_or_else(|i| i);

        self.buffer.insert(insert_pos, frame);
        self.consecutive_overruns = 0;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<AudioFrame> {
        let now = Instant::now();

        if let Some(last) = self.last_pop {
            let elapsed = now.duration_since(last).as_millis() as u32;
            if elapsed > 20 {
                self.consecutive_underruns += 1;
            }
        }
        self.last_pop = Some(now);

        if self.buffer.is_empty() {
            self.consecutive_underruns += 1;
            self.update_state();
            return None;
        }

        self.consecutive_underruns = 0;
        let frame = self.buffer.pop_front();
        self.update_state();
        frame
    }

    fn update_state(&mut self) {
        let len = self.buffer.len();

        self.state = match (len, self.state) {
            (0, _) => BufferState::Underrun,
            (n, _) if n < MIN_BUFFER_FRAMES => BufferState::Filling,
            (n, _) if n > MAX_BUFFER_FRAMES => BufferState::Draining,
            _ => BufferState::Stable,
        };

        self.adapt_target();
    }

    fn adapt_target(&mut self) {
        match self.state {
            BufferState::Underrun => {
                if self.consecutive_underruns > 3 {
                    self.target_frames = (self.target_frames + 1).min(MAX_BUFFER_FRAMES);
                    self.consecutive_underruns = 0;
                }
            }
            BufferState::Draining => {
                if self.consecutive_overruns > 3 {
                    self.target_frames = self.target_frames.saturating_sub(1).max(MIN_BUFFER_FRAMES);
                    self.consecutive_overruns = 0;
                }
            }
            BufferState::Stable => {
                if self.buffer.len() > self.target_frames + 1 {
                    self.consecutive_overruns += 1;
                } else if self.buffer.len() < self.target_frames.saturating_sub(1) {
                    self.consecutive_underruns += 1;
                }
            }
            BufferState::Filling => {}
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn target_frames(&self) -> usize {
        self.target_frames
    }

    pub fn state(&self) -> BufferState {
        self.state
    }

    pub fn health_percent(&self) -> f32 {
        if self.target_frames == 0 {
            0.0
        } else {
            (self.buffer.len() as f32 / self.target_frames as f32 * 100.0).min(100.0)
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.state = BufferState::Filling;
        self.consecutive_underruns = 0;
        self.consecutive_overruns = 0;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JitterBufferError {
    #[error("Buffer full, dropping frame")]
    BufferFull,
}

impl Default for JitterBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::frame::AudioFrame;

    fn make_frame(seq: u32) -> AudioFrame {
        AudioFrame::new(seq, 0, vec![0u8; 960], 0)
    }

    #[test]
    fn test_basic_push_pop() {
        let mut jb = JitterBuffer::new();
        jb.push(make_frame(1)).unwrap();
        jb.push(make_frame(2)).unwrap();

        assert_eq!(jb.pop().unwrap().header.sequence, 1);
        assert_eq!(jb.pop().unwrap().header.sequence, 2);
        assert!(jb.pop().is_none());
    }

    #[test]
    fn test_out_of_order() {
        let mut jb = JitterBuffer::new();
        jb.push(make_frame(3)).unwrap();
        jb.push(make_frame(1)).unwrap();
        jb.push(make_frame(2)).unwrap();

        assert_eq!(jb.pop().unwrap().header.sequence, 1);
        assert_eq!(jb.pop().unwrap().header.sequence, 2);
        assert_eq!(jb.pop().unwrap().header.sequence, 3);
    }

    #[test]
    fn test_buffer_full() {
        let mut jb = JitterBuffer::new();
        for i in 0..MAX_BUFFER_FRAMES {
            jb.push(make_frame(i as u32)).unwrap();
        }
        assert!(jb.push(make_frame(MAX_BUFFER_FRAMES as u32)).is_err());
    }
}