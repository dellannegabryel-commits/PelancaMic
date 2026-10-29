#[cfg(feature = "audio-tools")]
use cpal::traits::{DeviceTrait, HostTrait};

#[cfg(feature = "audio-tools")]
fn main() {
    let host = cpal::default_host();
    const SAMPLE_RATE: u32 = 48_000;

    for device in host.output_devices().unwrap() {
        if let Ok(desc) = device.description() {
            let name = desc.name();
            if name.contains("CABLE") || name.contains("VB-Audio") {
                println!("Testing device: {}", name);

                let configs: Vec<_> = device.supported_output_configs().unwrap().collect();

                // Try mono first
                let mono_config = configs.iter().find(|c| {
                    c.channels() == 1
                        && c.min_sample_rate() <= SAMPLE_RATE
                        && c.max_sample_rate() >= SAMPLE_RATE
                });

                if let Some(c) = mono_config {
                    println!("  MONO config found: {}ch, {}Hz-{}Hz",
                        c.channels(), c.min_sample_rate(), c.max_sample_rate());
                } else {
                    println!("  NO mono config at 48kHz");
                }

                // Try stereo fallback
                let stereo_config = configs.iter().find(|c| {
                    c.channels() == 2
                        && c.min_sample_rate() <= SAMPLE_RATE
                        && c.max_sample_rate() >= SAMPLE_RATE
                });

                if let Some(c) = stereo_config {
                    println!("  STEREO config found: {}ch, {}Hz-{}Hz",
                        c.channels(), c.min_sample_rate(), c.max_sample_rate());
                } else {
                    println!("  NO stereo config at 48kHz");
                }
            }
        }
    }
}

#[cfg(not(feature = "audio-tools"))]
fn main() {
    println!("This binary requires the 'audio-tools' feature. Run with: cargo run --bin test_vbconfig --features audio-tools");
}
