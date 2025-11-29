use crate::inventory::Inventory;
use crate::player::Player;
use crate::terrain::{Block, Chunk, ChunkCoord, MultiTileData};
use crate::terrain_generator::Generator;
use crate::world::WorldState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub player: Player,
    pub chunks: HashMap<ChunkCoord, ChunkData>,
    pub flow_timer: f32,
    pub tide_timer: f32,
    pub terrain_seed: Option<u64>,
    #[serde(default)]
    pub chests: HashMap<(i32, i32), Inventory>,
}

#[derive(Serialize, Deserialize)]
pub struct ChunkData {
    pub coord: ChunkCoord,
    pub blocks: Vec<Block>,
    pub multi_tile_data: HashMap<(usize, usize), MultiTileData>,
}

impl ChunkData {
    pub fn from_chunk(chunk: &Chunk) -> Self {
        use crate::terrain::CHUNK_SIZE;
        let mut blocks = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE);
        let mut multi_tile_data = HashMap::new();

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                blocks.push(chunk.get(x, y));

                // Save multi-tile data if present
                if let Some(data) = chunk.get_multi_tile_data(x, y) {
                    multi_tile_data.insert((x, y), data);
                }
            }
        }

        ChunkData {
            coord: chunk.coord,
            blocks,
            multi_tile_data,
        }
    }

    pub fn to_chunk(&self) -> Chunk {
        use crate::terrain::CHUNK_SIZE;
        let mut chunk = Chunk::new(self.coord);

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let index = y * CHUNK_SIZE + x;
                chunk.set(x, y, self.blocks[index]);
            }
        }

        // Restore multi-tile data
        for ((x, y), data) in &self.multi_tile_data {
            chunk.set_multi_tile_data(*x, *y, Some(*data));
        }

        chunk
    }
}

fn get_save_dir() -> PathBuf {
    let mut path = if let Some(data_dir) = dirs::data_local_dir() {
        data_dir
    } else {
        PathBuf::from(".")
    };
    path.push("waves");
    path.push("saves");
    path
}

fn get_save_path(slot: u32) -> PathBuf {
    let mut path = get_save_dir();
    path.push(format!("save_{}.bin", slot));
    path
}

pub fn save_game(world_state: &WorldState, slot: u32) -> Result<(), String> {
    let save_dir = get_save_dir();
    fs::create_dir_all(&save_dir).map_err(|e| format!("Failed to create save directory: {}", e))?;

    let mut chunks = HashMap::new();

    // Save all chunks including ocean and beach chunks
    // Water will be restored to air blocks below sea level on load
    for (coord, chunk) in &world_state.terrain.chunks {
        chunks.insert(*coord, ChunkData::from_chunk(chunk));
    }

    let chunk_count = chunks.len();

    // Extract seed from terrain generator
    let terrain_seed = match &world_state.terrain.generator {
        Generator::Procedural(gen) => Some(gen.get_seed()),
        Generator::Map(_) => None,
    };

    let save_data = SaveData {
        player: world_state.player.clone(),
        chunks,
        flow_timer: world_state.get_flow_timer(),
        tide_timer: world_state.get_tide_timer(),
        terrain_seed,
        chests: world_state.chests.clone(),
    };

    let encoded = bincode::serialize(&save_data)
        .map_err(|e| format!("Failed to serialize save data: {}", e))?;

    let save_path = get_save_path(slot);
    let mut file =
        File::create(&save_path).map_err(|e| format!("Failed to create save file: {}", e))?;

    file.write_all(&encoded)
        .map_err(|e| format!("Failed to write save file: {}", e))?;

    println!(
        "Game saved to: {:?} ({} bytes, {} chunks)",
        save_path,
        encoded.len(),
        chunk_count
    );
    Ok(())
}

pub fn load_game(slot: u32) -> Result<SaveData, String> {
    let save_path = get_save_path(slot);

    if !save_path.exists() {
        return Err(format!("Save file not found: {:?}", save_path));
    }

    let mut file =
        File::open(&save_path).map_err(|e| format!("Failed to open save file: {}", e))?;

    let mut encoded = Vec::new();
    file.read_to_end(&mut encoded)
        .map_err(|e| format!("Failed to read save file: {}", e))?;

    let save_data: SaveData = bincode::deserialize(&encoded)
        .map_err(|e| format!("Failed to deserialize save data: {}", e))?;

    println!(
        "Game loaded from: {:?} ({} bytes)",
        save_path,
        encoded.len()
    );
    Ok(save_data)
}

pub fn apply_save_data(world_state: &mut WorldState, save_data: SaveData) {
    use crate::terrain::{Block, CHUNK_SIZE};

    world_state.player = save_data.player;
    world_state.set_flow_timer(save_data.flow_timer);
    world_state.set_tide_timer(save_data.tide_timer);

    world_state.terrain.chunks.clear();

    let chunk_size = CHUNK_SIZE as i32;

    // Define sea level constant (should match terrain_generator)

    for (coord, chunk_data) in save_data.chunks {
        let mut chunk = chunk_data.to_chunk();

        // For ocean and beach chunks, restore water to air blocks below sea level
        let is_ocean = coord.x < 0;
        let is_beach = coord.x == 0 && (coord.y == -1 || coord.y == 0);

        if is_ocean || is_beach {
            for lx in 0..CHUNK_SIZE {
                for ly in 0..CHUNK_SIZE {
                    let wy = coord.y * chunk_size + ly as i32;

                    // If block is air and below sea level, fill with water
                    if chunk.get(lx, ly) == Block::Air && wy > crate::terrain_generator::SEA_LEVEL {
                        chunk.set(lx, ly, Block::Water);
                    }
                }
            }
        }

        world_state.terrain.chunks.insert(coord, chunk);
    }

    // Restore chest inventories
    world_state.chests = save_data.chests;
}

pub fn save_exists(slot: u32) -> bool {
    get_save_path(slot).exists()
}

pub fn delete_save(slot: u32) -> Result<(), String> {
    let save_path = get_save_path(slot);

    if !save_path.exists() {
        return Err(format!("Save file not found: {:?}", save_path));
    }

    fs::remove_file(&save_path).map_err(|e| format!("Failed to delete save file: {}", e))?;

    println!("Save file deleted: {:?}", save_path);
    Ok(())
}

pub fn delete_all_saves() -> Result<(), String> {
    let save_dir = get_save_dir();

    if !save_dir.exists() {
        return Ok(()); // No saves directory, nothing to delete
    }

    let entries =
        fs::read_dir(&save_dir).map_err(|e| format!("Failed to read save directory: {}", e))?;

    let mut deleted_count = 0;
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("bin") {
                if let Err(e) = fs::remove_file(&path) {
                    eprintln!("Failed to delete {:?}: {}", path, e);
                } else {
                    deleted_count += 1;
                }
            }
        }
    }

    println!(
        "Deleted {} save file(s) from: {:?}",
        deleted_count, save_dir
    );
    Ok(())
}
