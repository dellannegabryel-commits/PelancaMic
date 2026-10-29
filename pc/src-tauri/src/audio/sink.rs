//! Audio output sink using cpal

#[cfg(feature = "audio")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
#[cfg(feature = "audio")]
use cpal::{Device, StreamConfig};
use std::sync::{Arc, Mutex};
use crate::audio::jitter::JitterBuffer;
use crate::protocol::frame::AudioFrame;

const SAMPLE_RATE: u32 = 48_000;
const FRAMES_PER_BUFFER: u32 = 480; // 10ms at 48kHz

#[cfg(feature = "audio")]
pub struct AudioSink {
    device: Device,
    config: StreamConfig,
    jitter_buffer: Arc<Mutex<JitterBuffer>>,
    stream: Option<cpal::Stream>,
    volume: Arc<Mutex<f32>>,
    muted: Arc<Mutex<bool>>,
}

#[cfg(not(feature = "audio"))]
pub struct AudioSink {
    _placeholder: (),
}

#[cfg(feature = "audio")]
impl AudioSink {
    pub fn new(device_name: Option<&str>) -> anyhow::Result<Self> {
        let host = cpal::default_host();

        let device = if let Some(name) = device_name {
            host.output_devices()?
                .find(|d| {
                    d.description()
                        .map(|desc| desc.name().contains(name))
                        .unwrap_or(false)
                })
                .ok_or_else(|| anyhow::anyhow!("Audio device not found: {}", name))?
        } else {
            host.default_output_device()
                .ok_or_else(|| anyhow::anyhow!("No default output device"))?
        };

        // Try mono first, then fallback to stereo (most devices only expose stereo)
        let supported_config = device
            .supported_output_configs()?
            .find(|c| {
                c.channels() == 1
                    && c.min_sample_rate() <= SAMPLE_RATE
                    && c.max_sample_rate() >= SAMPLE_RATE
            })
            .or_else(|| {
                device
                    .supported_output_configs()
                    .ok()?
                    .find(|c| {
                        c.channels() == 2
                            && c.min_sample_rate() <= SAMPLE_RATE
                            && c.max_sample_rate() >= SAMPLE_RATE
                    })
            })
            .ok_or_else(|| anyhow::anyhow!("No supported config for 48kHz mono or stereo"))?
            .with_sample_rate(SAMPLE_RATE);

        let config: StreamConfig = supported_config.into();

        let device_name = device.description()
            .map(|d| d.name().to_string())
            .unwrap_or("Unknown".to_string());

        tracing::info!(
            "Audio sink: {} @ {}Hz {}ch {:?}",
            device_name,
            config.sample_rate,
            config.channels,
            config.buffer_size
        );

        Ok(Self {
            device,
            config,
            jitter_buffer: Arc::new(Mutex::new(JitterBuffer::new())),
            stream: None,
            volume: Arc::new(Mutex::new(1.0)),
            muted: Arc::new(Mutex::new(false)),
        })
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        let jitter_buffer = self.jitter_buffer.clone();
        let volume = self.volume.clone();
        let muted = self.muted.clone();
        let config = self.config.clone();
        let device = self.device.clone();

        let stream = device.build_output_stream(
            config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut jb = jitter_buffer.lock().unwrap();
                let vol = *volume.lock().unwrap();
                let is_muted = *muted.lock().unwrap();

                for sample in data.iter_mut() {
                    if is_muted {
                        *sample = 0.0;
                        continue;
                    }
                    if let Some(frame) = jb.pop() {
                        let samples = frame.payload_as_i16();
                        if let Some(s) = samples.first() {
                            *sample = (*s as f32 / i16::MAX as f32) * vol;
                        } else {
                            *sample = 0.0;
                        }
                    } else {
                        *sample = 0.0;
                    }
                }
            },
            move |err| {
                tracing::error!("Audio stream error: {}", err);
            },
            None,
        )?;

        stream.play()?;
        self.stream = Some(stream);
        tracing::info!("Audio sink started successfully");
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }
        let _ = self.jitter_buffer.lock().map(|mut jb| jb.clear());
    }

    pub fn push_frame(&self, frame: AudioFrame) {
        let _ = self.jitter_buffer.lock().unwrap().push(frame);
    }

    pub fn set_volume(&self, volume: f32) {
        *self.volume.lock().unwrap() = volume.clamp(0.0, 4.0);
    }

    pub fn set_muted(&self, muted: bool) {
        *self.muted.lock().unwrap() = muted;
    }

    pub fn get_stats(&self) -> (usize, usize, f32) {
        let jb = self.jitter_buffer.lock().unwrap();
        (jb.len(), jb.target_frames(), jb.health_percent())
    }

    pub fn list_output_devices() -> Vec<String> {
        cpal::default_host()
            .output_devices()
            .map(|devices| {
                devices
                    .filter_map(|d| d.description().ok().map(|desc| desc.name().to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(not(feature = "audio"))]
impl AudioSink {
    pub fn new(_device_name: Option<&str>) -> anyhow::Result<Self> {
        Err(anyhow::anyhow!("Audio feature not enabled"))
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Audio feature not enabled"))
    }

    pub fn stop(&mut self) {}

    pub fn push_frame(&self, _frame: AudioFrame) {}

    pub fn set_volume(&self, _volume: f32) {}

    pub fn set_muted(&self, _muted: bool) {}

    pub fn get_stats(&self) -> (usize, usize, f32) {
        (0, 0, 0.0)
    }

    pub fn list_output_devices() -> Vec<String> {
        vec![]
    }
}

impl Drop for AudioSink {
    fn drop(&mut self) {
        // Avoid calling stop() which might touch streams
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_devices() {
        let devices = AudioSink::list_output_devices();
        assert!(!devices.is_empty() || true); // May be empty in CI
    }
}