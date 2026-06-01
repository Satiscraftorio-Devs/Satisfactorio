use std::path::PathBuf;

use anyhow::Result;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use game::world::data::block::BlockInstance;
use game::world::data::chunk::IntraChunkCoords;
use game::world::modified_chunk::ModifiedChunk;
use game::world::modified_chunk::ModifiedWorld;

use crate::player::Player;

const SAVE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
pub struct SaveChunk {
    pub blocks: Vec<(IntraChunkCoords, BlockInstance)>,
}

#[derive(Serialize, Deserialize)]
pub struct SaveWorld {
    pub chunks: FxHashMap<(i32, i32, i32), SaveChunk>,
}

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    version: u32,
    pub seed: u32,
    pub world: SaveWorld,
    pub players: Vec<Player>,
}

impl SaveData {
    pub fn new(seed: u32, world: SaveWorld, players: Vec<Player>) -> Self {
        Self {
            version: SAVE_VERSION,
            seed,
            world,
            players,
        }
    }
}

impl From<&ModifiedWorld> for SaveWorld {
    fn from(world: &ModifiedWorld) -> Self {
        let chunks = world
            .chunks()
            .iter()
            .map(|(&key, chunk)| {
                let blocks = chunk.blocks().to_vec();
                (key, SaveChunk { blocks })
            })
            .collect();
        Self { chunks }
    }
}

impl From<SaveWorld> for ModifiedWorld {
    fn from(world: SaveWorld) -> Self {
        let chunks = world
            .chunks
            .into_iter()
            .map(|(key, chunk)| (key, ModifiedChunk::from(chunk)))
            .collect();
        ModifiedWorld { chunks }
    }
}

impl From<SaveChunk> for ModifiedChunk {
    fn from(chunk: SaveChunk) -> Self {
        ModifiedChunk::from_blocks(chunk.blocks)
    }
}

#[derive(Clone)]
pub struct PersistenceService {
    save_path: PathBuf,
}

impl PersistenceService {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { save_path: path.into() }
    }

    pub fn exists(&self) -> bool {
        self.save_path.exists()
    }

    pub fn save(&self, data: &SaveData) -> Result<()> {
        if let Some(parent) = self.save_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = bincode::serialize(data)?;
        std::fs::write(&self.save_path, bytes)?;
        Ok(())
    }

    pub fn load(&self) -> Result<Option<SaveData>> {
        if !self.exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(&self.save_path)?;
        let data = bincode::deserialize(&bytes)?;
        Ok(Some(data))
    }
}
