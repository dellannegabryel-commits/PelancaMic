use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use tauri::State;
// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod audio;
mod network;
mod protocol;

use audio::sink::AudioSink;
use network::discovery::start_discovery_server;
use network::audio::AudioReceiver;
use network::control::ControlServer;

struct AppState {
    audio_sink: Arc<Mutex<Option<AudioSink>>>,
    audio_receiver: Arc<Mutex<Option<Arc<AudioReceiver>>>>,
    control_server: Arc<Mutex<Option<Arc<ControlServer>>>>,
    discovery_handle: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
    audio_handle: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
    control_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    hostname: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            audio_sink: Arc::new(Mutex::new(None)),
            audio_receiver: Arc::new(Mutex::new(None)),
            control_server: Arc::new(Mutex::new(None)),
            discovery_handle: Arc::new(Mutex::new(None)),
            audio_handle: Arc::new(Mutex::new(None)),
            control_handle: Arc::new(Mutex::new(None)),
            hostname: hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "PelancaMic-PC".to_string()),
        }
    }
}

#[tauri::command]
fn start_server(
    device_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let hostname = state.hostname.clone();

    let discovery_hostname = hostname.clone();
    let discovery_handle = std::thread::spawn(move || {
        let _ = start_discovery_server(discovery_hostname, protocol::AUDIO_PORT, protocol::CONTROL_PORT);
    });
    *state.discovery_handle.lock().unwrap() = Some(discovery_handle);

    // Initialize and start audio sink
    let device_name_clone = device_name.clone();
    let mut audio_sink = AudioSink::new(device_name_clone.as_deref())
        .map_err(|e| format!("Failed to create audio sink: {}", e))?;
    audio_sink.start()
        .map_err(|e| format!("Failed to start audio sink: {}", e))?;
    *state.audio_sink.lock().unwrap() = Some(audio_sink);

    let control_server = Arc::new(ControlServer::new());
    *state.control_server.lock().unwrap() = Some(control_server.clone());

    let control_server_clone = control_server.clone();
    let control_handle = tokio::spawn(async move {
        let addr: SocketAddr = (Ipv4Addr::UNSPECIFIED, protocol::CONTROL_PORT).into();
        let _ = control_server_clone.start(addr).await;
    });
    *state.control_handle.lock().unwrap() = Some(control_handle);

    // Start audio receiver
    let audio_receiver = Arc::new(AudioReceiver::new((Ipv4Addr::UNSPECIFIED, protocol::AUDIO_PORT).into())
        .map_err(|e| format!("Failed to create audio receiver: {}", e))?);
    *state.audio_receiver.lock().unwrap() = Some(audio_receiver.clone());

    let audio_receiver_clone = audio_receiver.clone();
    let audio_handle = std::thread::spawn(move || {
        audio_receiver_clone.start_receive_loop();
    });
    *state.audio_handle.lock().unwrap() = Some(audio_handle);

    Ok("Server started".to_string())
}

#[tauri::command]
fn stop_server(state: State<'_, AppState>) -> Result<String, String> {
    // Stop audio sink
    if let Some(mut sink) = state.audio_sink.lock().unwrap().take() {
        sink.stop();
    }

    // Stop discovery
    if let Some(handle) = state.discovery_handle.lock().unwrap().take() {
        let _ = handle.join();
    }

    // Stop audio receiver
    if let Some(handle) = state.audio_handle.lock().unwrap().take() {
        let _ = handle.join();
    }
    *state.audio_receiver.lock().unwrap() = None;

    // Stop control server
    if let Some(handle) = state.control_handle.lock().unwrap().take() {
        handle.abort();
    }
    *state.control_server.lock().unwrap() = None;

    Ok("Server stopped".to_string())
}

#[tauri::command]
fn get_status(state: State<'_, AppState>) -> Result<ServerStatus, String> {
    let running = state.discovery_handle.lock().unwrap().is_some() || state.control_server.lock().unwrap().is_some();
    let clients = state.control_server.lock().unwrap()
        .as_ref()
        .map(|s| s.get_clients().len())
        .unwrap_or(0);

    let (buffer_len, target_frames, buffer_health) = (0, 0, 0.0f32);

    let audio_stats = network::audio::AudioStats::default();

    Ok(ServerStatus {
        running,
        clients,
        audio_port: protocol::AUDIO_PORT,
        control_port: protocol::CONTROL_PORT,
        discovery_port: protocol::DISCOVERY_PORT,
        buffer_len,
        target_frames,
        buffer_health,
        packets_received: audio_stats.packets_received,
        packets_lost: audio_stats.packets_lost,
        jitter_ms: audio_stats.last_jitter_ms,
    })
}

#[tauri::command]
fn set_volume(volume: f32, state: State<'_, AppState>) -> Result<String, String> {
    if let Some(sink) = state.audio_sink.lock().unwrap().as_ref() {
        sink.set_volume(volume);
        Ok("Volume set".to_string())
    } else {
        Err("Server not running".to_string())
    }
}

#[tauri::command]
fn set_muted(muted: bool, state: State<'_, AppState>) -> Result<String, String> {
    if let Some(sink) = state.audio_sink.lock().unwrap().as_ref() {
        sink.set_muted(muted);
        Ok("Mute set".to_string())
    } else {
        Err("Server not running".to_string())
    }
}

#[tauri::command]
fn list_audio_devices() -> Result<Vec<String>, String> {
    Ok(AudioSink::list_output_devices())
}

#[derive(serde::Serialize)]
struct ServerStatus {
    running: bool,
    clients: usize,
    audio_port: u16,
    control_port: u16,
    discovery_port: u16,
    buffer_len: usize,
    target_frames: usize,
    buffer_health: f32,
    packets_received: u64,
    packets_lost: u64,
    jitter_ms: f32,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            start_server,
            stop_server,
            get_status,
            set_volume,
            set_muted,
            list_audio_devices,
        ])
        .setup(|_app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}