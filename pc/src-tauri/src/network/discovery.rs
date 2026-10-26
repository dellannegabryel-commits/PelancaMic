//! UDP Broadcast Discovery Server

use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::protocol::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Beacon {
    pub app: String,
    pub v: u8,
    pub name: String,
    pub audio_port: u16,
    pub ctrl_port: u16,
}

impl Beacon {
    pub fn new(hostname: String, audio_port: u16, ctrl_port: u16) -> Self {
        Self {
            app: "pelancamic".to_string(),
            v: 1,
            name: hostname,
            audio_port,
            ctrl_port,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

pub fn start_discovery_server(
    hostname: String,
    audio_port: u16,
    ctrl_port: u16,
) -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(Duration::from_millis(100)))?;

    let beacon = Beacon::new(hostname, audio_port, ctrl_port);
    let beacon_json = beacon.to_json();
    let beacon_bytes = beacon_json.as_bytes();

    let broadcast_addr: SocketAddr = (Ipv4Addr::BROADCAST, DISCOVERY_PORT).into();

    tracing::info!("Discovery server started on port {} (broadcast)", DISCOVERY_PORT);
    tracing::info!("Beacon: {}", beacon_json);

    loop {
        if let Err(e) = socket.send_to(beacon_bytes, broadcast_addr) {
            tracing::error!("Failed to send beacon: {}", e);
        }
        thread::sleep(Duration::from_millis(BEACON_INTERVAL_MS));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beacon_serialization() {
        let beacon = Beacon::new("TEST-PC".to_string(), 41235, 41236);
        let json = beacon.to_json();
        assert!(json.contains("pelancamic"));
        assert!(json.contains("TEST-PC"));
        assert!(json.contains("41235"));
        assert!(json.contains("41236"));
    }
}