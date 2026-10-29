#[cfg(feature = "audio-tools")]
use cpal::traits::{DeviceTrait, HostTrait};

#[cfg(feature = "audio-tools")]
fn main() {
    let host = cpal::default_host();

    println!("=== Testing device selection logic ===");
    for device in host.output_devices().unwrap() {
        if let Ok(desc) = device.description() {
            let name = desc.name();
            println!("Device: {}", name);

            // Test the matching logic from sink.rs
            let search_terms = ["VB-Audio", "CABLE", "Virtual Cable", "VB-Audio Virtual Cable"];
            for term in &search_terms {
                if name.contains(term) {
                    println!("  -> MATCHES '{}'", term);
                }
            }
        }
    }
}

#[cfg(not(feature = "audio-tools"))]
fn main() {
    println!("This binary requires the 'audio-tools' feature. Run with: cargo run --bin test_device_select --features audio-tools");
}
