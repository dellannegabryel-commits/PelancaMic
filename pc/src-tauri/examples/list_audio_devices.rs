#[cfg(feature = "audio-tools")]
use cpal::traits::{DeviceTrait, HostTrait};

#[cfg(feature = "audio-tools")]
fn main() {
    let host = cpal::default_host();

    println!("=== All Output Devices ===");
    match host.output_devices() {
        Ok(devices) => {
            for device in devices {
                println!("\n--- Device ---");
                if let Ok(name) = device.name() {
                    println!("  name(): {:?}", name);
                }
                if let Ok(desc) = device.description() {
                    println!("  description().name(): {:?}", desc.name());
                }
                list_supported_configs(&device);
            }
        }
        Err(e) => println!("Error enumerating devices: {}", e),
    }
}

#[cfg(feature = "audio-tools")]
fn list_supported_configs(device: &cpal::Device) {
    match device.supported_output_configs() {
        Ok(configs) => {
            for config in configs {
                let min_rate = config.min_sample_rate();
                let max_rate = config.max_sample_rate();
                if config.channels() <= 2 && max_rate.0 >= 48000 && min_rate.0 <= 48000 {
                    println!("  Config:");
                    println!("    Sample format: {:?}", config.sample_format());
                    println!("    Channels: {}", config.channels());
                    println!("    Sample rate range: {} - {}", min_rate.0, max_rate.0);
                    println!("    Buffer size: {:?}", config.buffer_size());
                }
            }
        }
        Err(e) => println!("  Error getting configs: {}", e),
    }
}

#[cfg(not(feature = "audio-tools"))]
fn main() {
    println!("This example requires the 'audio-tools' feature. Run with: cargo run --example list_audio_devices --features audio-tools");
}
