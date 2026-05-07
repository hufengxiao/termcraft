use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::block::BlockType;

#[derive(Serialize, Deserialize)]
pub struct WorldSave {
    pub blocks: Vec<Vec<Vec<BlockType>>>,
    pub player_x: f64,
    pub player_y: f64,
    pub player_z: f64,
    pub player_yaw: f64,
    pub player_pitch: f64,
    pub tick: u64,
}

fn save_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    path.push("termcraft_save.bin");
    path
}

pub fn save_world(
    blocks: &Vec<Vec<Vec<BlockType>>>,
    player_x: f64, player_y: f64, player_z: f64,
    player_yaw: f64, player_pitch: f64,
    tick: u64,
) -> Result<(), String> {
    let data = WorldSave {
        blocks: blocks.clone(),
        player_x, player_y, player_z,
        player_yaw, player_pitch,
        tick,
    };
    let encoded = bincode::serialize(&data).map_err(|e| format!("Serialize error: {e}"))?;
    fs::write(save_path(), encoded).map_err(|e| format!("Write error: {e}"))
}

pub fn load_world() -> Option<WorldSave> {
    let path = save_path();
    if !path.exists() {
        return None;
    }
    let data = fs::read(&path).ok()?;
    bincode::deserialize(&data).ok()
}
