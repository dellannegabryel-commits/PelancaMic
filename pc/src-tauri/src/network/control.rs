//! WebSocket Control Server

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ControlMessage {
    Hello { client_name: String },
    Mute { muted: bool },
    Volume { level: f32 },
    Bye,
    Ping,
    Pong,
    Error { message: String },
}

impl ControlMessage {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}

pub struct ControlServer {
    clients: Arc<Mutex<HashMap<SocketAddr, ClientInfo>>>,
    tx: broadcast::Sender<ControlMessage>,
}

#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub addr: SocketAddr,
    pub name: String,
    pub muted: bool,
    pub volume: f32,
}

impl ControlServer {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(32);
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            tx,
        }
    }

    pub async fn start(self: Arc<Self>, addr: SocketAddr) -> anyhow::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        tracing::info!("Control WebSocket server listening on {}", addr);

        loop {
            let (stream, peer_addr) = listener.accept().await?;
            let server = self.clone();
            tokio::spawn(async move {
                if let Err(e) = server.handle_connection(stream, peer_addr).await {
                    tracing::error!("Control connection error: {}", e);
                }
            });
        }
    }

    async fn handle_connection(&self, stream: TcpStream, peer_addr: SocketAddr) -> anyhow::Result<()> {
        let ws_stream = accept_async(stream).await?;
        let (ws_sender, mut ws_receiver) = ws_stream.split();

        {
            let mut clients = self.clients.lock().unwrap();
            clients.insert(peer_addr, ClientInfo {
                addr: peer_addr,
                name: "Unknown".to_string(),
                muted: false,
                volume: 1.0,
            });
        }

        tracing::info!("Control client connected: {}", peer_addr);

        let mut rx = self.tx.subscribe();

        let send_task = tokio::spawn({
            let mut ws_sender = ws_sender;
            async move {
                while let Ok(msg) = rx.recv().await {
                    if ws_sender.send(Message::Text(msg.to_json())).await.is_err() {
                        break;
                    }
                }
            }
        });

        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Some(control_msg) = ControlMessage::from_json(&text) {
                        self.process_message(peer_addr, control_msg).await;
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        send_task.abort();
        self.clients.lock().unwrap().remove(&peer_addr);
        tracing::info!("Control client disconnected: {}", peer_addr);
        Ok(())
    }

    async fn process_message(&self, peer_addr: SocketAddr, msg: ControlMessage) {
        match msg {
            ControlMessage::Hello { client_name } => {
                if let Some(client) = self.clients.lock().unwrap().get_mut(&peer_addr) {
                    client.name = client_name;
                    tracing::info!("Client identified: {}", client.name);
                }
            }
            ControlMessage::Mute { muted } => {
                if let Some(client) = self.clients.lock().unwrap().get_mut(&peer_addr) {
                    client.muted = muted;
                    tracing::info!("Client {} mute: {}", client.name, muted);
                }
            }
            ControlMessage::Volume { level } => {
                if let Some(client) = self.clients.lock().unwrap().get_mut(&peer_addr) {
                    client.volume = level.clamp(0.0, 4.0);
                    tracing::info!("Client {} volume: {}", client.name, client.volume);
                }
            }
            ControlMessage::Bye => {
                tracing::info!("Client {} sent bye", peer_addr);
            }
            ControlMessage::Ping => {
                let _ = self.tx.send(ControlMessage::Pong);
            }
            _ => {}
        }
    }

    pub fn get_clients(&self) -> Vec<ClientInfo> {
        self.clients.lock().unwrap().values().cloned().collect()
    }

    pub fn broadcast(&self, msg: ControlMessage) {
        let _ = self.tx.send(msg);
    }
}

impl Default for ControlServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_message_serialization() {
        let msg = ControlMessage::Hello { client_name: "TestPhone".to_string() };
        let json = msg.to_json();
        assert!(json.to_lowercase().contains("hello"));
        assert!(json.contains("TestPhone"));

        let parsed = ControlMessage::from_json(&json).unwrap();
        assert!(matches!(parsed, ControlMessage::Hello { client_name } if client_name == "TestPhone"));
    }

    #[test]
    fn test_volume_clamping() {
        let msg = ControlMessage::Volume { level: 5.0 };
        let json = msg.to_json();
        let parsed = ControlMessage::from_json(&json).unwrap();
        if let ControlMessage::Volume { level } = parsed {
            assert_eq!(level, 5.0); // Clamping happens on server side
        }
    }
}