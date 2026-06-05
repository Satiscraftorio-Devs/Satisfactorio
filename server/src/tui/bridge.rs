use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct PlayerInfo {
    pub id: u64,
    pub username: String,
    pub position: (f64, f64, f64),
    pub gamemode: String,
}

pub struct TuiState {
    pub players: Vec<PlayerInfo>,
    pub logs: Vec<String>,
    pub address: String,
    pub start_time: std::time::Instant,
    pub seed: u32,
    pub chunk_count: usize,
    pub modified_count: usize,
    pub connected_player_count: usize,
}

impl Default for TuiState {
    fn default() -> Self {
        Self {
            players: Vec::new(),
            logs: Vec::new(),
            address: String::new(),
            start_time: std::time::Instant::now(),
            seed: 0,
            chunk_count: 0,
            modified_count: 0,
            connected_player_count: 0,
        }
    }
}

pub enum TuiCommand {
    Shutdown,
    Save,
    Kick(u64),
    Log(String),
}

pub struct TuiBridge {
    pub state: Arc<Mutex<TuiState>>,
    pub command_tx: mpsc::UnboundedSender<TuiCommand>,
}

impl TuiBridge {
    pub fn new(state: Arc<Mutex<TuiState>>, command_tx: mpsc::UnboundedSender<TuiCommand>) -> Self {
        Self { state, command_tx }
    }
}
