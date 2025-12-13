use crate::maps::{EdgeConstraint, EdgeType, MapOrientation, MapQuery, MapSet};
use crate::terrain::{Block, Chunk, ChunkCoord, OreSpawnData, CHUNK_SIZE};
use noise::{NoiseFn, Perlin};
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::collections::HashMap;
use std::path::Path;

pub const SEA_LEVEL: i32 = -3;
const SEA_FLOOR: i32 = 30;
const BEACH_HEIGHT: i32 = -10;

pub enum Generator {
    Procedural(TerrainGenerator),
    Map(MapGenerator),
}

impl Generator {
    pub fn generate_chunk(&mut self, coord: ChunkCoord) -> Chunk {
        match self {
            Generator::Procedural(gen) => gen.generate_chunk(coord),
            Generator::Map(gen) => gen.generate_chunk(coord),
        }
    }
}

pub struct MapGenerator {
    pixels: Vec<u8>,
    width: i32,
    height: i32,
}

impl MapGenerator {
    pub fn from_image(path: &str) -> Result<Self, String> {
        use raylib::prelude::*;
        let mut image =
            Image::load_image(path).map_err(|e| format!("Failed to load image: {}", e))?;

        let width = image.width;
        let height = image.height;

        let mut pixels = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let color = image.get_color(x, y);
                pixels.push(color.r);
                pixels.push(color.g);
                pixels.push(color.b);
            }
        }

        Ok(MapGenerator {
            pixels,
            width,
            height,
        })
    }

    pub fn map_width(&self) -> i32 {
        self.width
    }

    pub fn map_height(&self) -> i32 {
        self.height
    }

    fn get_block_at(&self, x: i32, y: i32) -> Block {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return Block::Stone;
        }

        let idx = ((y * self.width + x) * 3) as usize;
        let r = self.pixels[idx];
        let g = self.pixels[idx + 1];
        let b = self.pixels[idx + 2];

        if r == 0 && g == 0 && b == 0 {
            Block::Stone
        } else if r == 127 && g == 127 && b == 127 {
            if self.get_block_at(x, y - 1).is_solid() {
                Block::Stalactite
            } else {
                Block::Stalagmite
            }
        } else if r == 255 && g == 255 && b == 255 {
            Block::Air
        } else if r == 0 && g == 149 && b == 199 {
            Block::Tide
        } else {
            panic!("Unknown color in map: (R: {}, G: {}, B {})", r, g, b);
        }
    }

    pub fn generate_chunk(&self, coord: ChunkCoord) -> Chunk {
        let mut chunk = Chunk::new(coord);
        let chunk_size = CHUNK_SIZE as i32;

        for lx in 0..CHUNK_SIZE {
            for ly in 0..CHUNK_SIZE {
                let wx = coord.x * chunk_size + lx as i32;
                let wy = coord.y * chunk_size + ly as i32;
                let block = self.get_block_at(wx, wy);
                chunk.set(lx, ly, block);
            }
        }
        chunk
    }
}

fn calculate_vein_count<R: Rng>(wy: i32, spawn_data: &OreSpawnData, rng: &mut R) -> usize {
    // Check if we're in the spawn range
    if wy < spawn_data.spawns_from || wy > spawn_data.spawns_to {
        return 0;
    }

    // Calculate progress through the triangular distribution
    let progress = if wy <= spawn_data.spawns_peak {
        // Rising phase: from spawns_from to spawns_peak
        let range = spawn_data.spawns_peak - spawn_data.spawns_from;
        if range == 0 {
            1.0
        } else {
            (wy - spawn_data.spawns_from) as f32 / range as f32
        }
    } else {
        // Falling phase: from spawns_peak to spawns_to
        let range = spawn_data.spawns_to - spawn_data.spawns_peak;
        if range == 0 {
            1.0
        } else {
            1.0 - (wy - spawn_data.spawns_peak) as f32 / range as f32
        }
    };

    // Expected number of veins at this depth
    let expected_veins = spawn_data.spawns_pap * progress;

    let base_count = expected_veins.floor() as usize;
    let fractional = expected_veins - expected_veins.floor();

    if rng.gen::<f32>() < fractional {
        base_count + 1
    } else {
        base_count
    }
}

pub fn halton(index: u32, base: u32) -> f64 {
    let mut result = 0.0;
    let mut f = 1.0 / base as f64;
    let mut i = index;

    while i > 0 {
        result += f * (i % base) as f64;
        i /= base;
        f /= base as f64;
    }

    result
}
pub fn halton_2d(index: u32) -> (usize, usize) {
    let base_a = 2;
    let base_b = 3;
    (
        (halton(index, base_a) * CHUNK_SIZE as f64) as usize,
        (halton(index, base_b) * CHUNK_SIZE as f64) as usize,
    )
}

fn is_adjacent_to_air(chunk: &Chunk, x: i32, y: i32) -> bool {
    (x > 0 && chunk.get((x - 1) as usize, y as usize) == Block::Air)
        || (x < CHUNK_SIZE as i32 - 1 && chunk.get((x + 1) as usize, y as usize) == Block::Air)
        || (y > 0 && chunk.get(x as usize, (y - 1) as usize) == Block::Air)
        || (y < CHUNK_SIZE as i32 - 1 && chunk.get(x as usize, (y + 1) as usize) == Block::Air)
}

pub fn add_ore_vein(chunk: &mut Chunk, x: i32, y: i32, ore: Block, min_size: i32, max_size: i32) {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    // Create a seeded RNG based on chunk position and ore position
    let seed = ((chunk.coord.x as u64) << 32)
        | ((chunk.coord.y as u64) << 16)
        | ((x as u64) << 8)
        | (y as u64);
    let mut rng = StdRng::seed_from_u64(seed);

    // Generate vein size using the provided range
    let vein_size = rng.gen_range(min_size..=max_size);

    // Find nearest tile on border of Block::Air and not Block::Air
    let mut current_x = x;
    let mut current_y = y;

    // Check if starting position is valid (in bounds)
    if current_x < 0
        || current_x >= CHUNK_SIZE as i32
        || current_y < 0
        || current_y >= CHUNK_SIZE as i32
    {
        return;
    }

    let current_block = chunk.get(current_x as usize, current_y as usize);

    // Determine if we need to search for a stone block adjacent to air
    let needs_search = match current_block {
        Block::Air => true, // Need to find stone
        Block::Stone => !is_adjacent_to_air(chunk, current_x, current_y), // Need better position if not adjacent to air
        _ => true, // For any other block type, search for valid position
    };

    if needs_search {
        // Search in expanding radius for a solid block adjacent to air
        let mut found = false;
        'outer: for radius in 1..10 {
            for dx in -radius..=radius {
                for dy in -radius..=radius {
                    let check_x = (current_x + dx).max(0).min(CHUNK_SIZE as i32 - 1);
                    let check_y = (current_y + dy).max(0).min(CHUNK_SIZE as i32 - 1);

                    let block = chunk.get(check_x as usize, check_y as usize);
                    if block == Block::Stone && is_adjacent_to_air(chunk, check_x, check_y) {
                        current_x = check_x;
                        current_y = check_y;
                        found = true;
                        break 'outer;
                    }
                }
            }
        }

        if !found {
            return; // No suitable starting position found
        }
    }

    // For each vein size, do a "drunk walk" to place ore
    for _ in 0..vein_size {
        // Replace current block if it's stone
        if current_x >= 0
            && current_x < CHUNK_SIZE as i32
            && current_y >= 0
            && current_y < CHUNK_SIZE as i32
        {
            let block = chunk.get(current_x as usize, current_y as usize);
            if block == Block::Stone {
                chunk.set(current_x as usize, current_y as usize, ore);
            }
        }

        // Move in a random direction to find next stone block
        let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        let mut attempts = 0;
        let max_attempts = 8;

        while attempts < max_attempts {
            let dir = directions[rng.gen_range(0..directions.len())];
            let new_x = current_x + dir.0;
            let new_y = current_y + dir.1;

            // Check bounds
            if new_x >= 0 && new_x < CHUNK_SIZE as i32 && new_y >= 0 && new_y < CHUNK_SIZE as i32 {
                let block = chunk.get(new_x as usize, new_y as usize);
                if block == Block::Stone || block == ore {
                    current_x = new_x;
                    current_y = new_y;
                    break;
                }
            }

            attempts += 1;
        }

        // If we couldn't find a valid direction, stop the vein
        if attempts >= max_attempts {
            break;
        }
    }
}

// Identifies which half of a tile this is
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum HalfTilePosition {
    Left,   // Left half of horizontal tile (32x32)
    Right,  // Right half of horizontal tile (32x32)
    Top,    // Top half of vertical tile (32x32)
    Bottom, // Bottom half of vertical tile (32x32)
}

// A half-tile stores the full tile data and which portion it represents
#[derive(Clone)]
struct HalfTile {
    position: HalfTilePosition,
    edges: crate::maps::MapEdges,
    pixels: Vec<u8>, // Full tile's RGB pixels
    width: i32,      // Full tile's width
    height: i32,     // Full tile's height
}

// Coordinate in the half-tile grid (each half-tile is 32x32 pixels)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct HalfTileCoord {
    x: i32, // x * 32 = world pixel x
    y: i32, // y * 32 = world pixel y
}

pub struct TerrainGenerator {
    seed: u64,
    noise: Perlin,
    map_set: MapSet,
    // Grid storing which half-tile occupies each 32x32 cell
    half_tile_grid: HashMap<HalfTileCoord, HalfTile>,
}

impl HalfTile {
    fn get_pixel(&self, x: i32, y: i32) -> (u8, u8, u8) {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return (0, 0, 0); // Black for out of bounds
        }
        let idx = ((y * self.width + x) * 3) as usize;
        (self.pixels[idx], self.pixels[idx + 1], self.pixels[idx + 2])
    }
}

impl TerrainGenerator {
    pub fn new(seed: u64) -> Self {
        let noise = Perlin::new(seed as u32);

        // Load map tiles from assets/maps directory
        let map_set = MapSet::load_from_directory(Path::new("assets/maps")).unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load maps: {}. Using empty MapSet.", e);
            MapSet::new()
        });

        println!(
            "Loaded {} map tiles ({} horizontal, {} vertical)",
            map_set.len(),
            map_set.count_horizontal(),
            map_set.count_vertical()
        );

        TerrainGenerator {
            seed,
            noise,
            map_set,
            half_tile_grid: HashMap::new(),
        }
    }

    pub fn get_seed(&self) -> u64 {
        self.seed
    }

    /// Get the 4 half-tile coordinates that make up a chunk (64x64 = 2x2 grid of 32x32 half-tiles)
    fn get_half_tile_coords_for_chunk(&self, coord: ChunkCoord) -> [HalfTileCoord; 4] {
        let base_x = coord.x * 2; // Each chunk is 2 half-tiles wide
        let base_y = coord.y * 2; // Each chunk is 2 half-tiles tall
        [
            HalfTileCoord {
                x: base_x,
                y: base_y,
            }, // Top-left
            HalfTileCoord {
                x: base_x + 1,
                y: base_y,
            }, // Top-right
            HalfTileCoord {
                x: base_x,
                y: base_y + 1,
            }, // Bottom-left
            HalfTileCoord {
                x: base_x + 1,
                y: base_y + 1,
            }, // Bottom-right
        ]
    }

    /// Determine which half-tile position this coordinate represents based on herringbone pattern
    fn calculate_half_tile_position(
        &self,
        half_coord: HalfTileCoord,
    ) -> (MapOrientation, HalfTilePosition, HalfTileCoord) {
        // Herringbone pattern logic:
        // - Pattern alternates H-V based on diagonal position
        // - Horizontal tiles (64x32) occupy 2 horizontal half-tiles
        // - Vertical tiles (32x64) occupy 2 vertical half-tiles

        let diff = half_coord.x - half_coord.y;
        let modulo = diff.rem_euclid(4);

        match modulo {
            0 => (
                MapOrientation::Horizontal,
                HalfTilePosition::Left,
                half_coord,
            ),
            1 => {
                let tile_origin = HalfTileCoord {
                    x: half_coord.x - 1,
                    y: half_coord.y,
                };
                (
                    MapOrientation::Horizontal,
                    HalfTilePosition::Right,
                    tile_origin,
                )
            }
            2 => {
                let tile_origin = HalfTileCoord {
                    x: half_coord.x,
                    y: half_coord.y - 1,
                };
                (
                    MapOrientation::Vertical,
                    HalfTilePosition::Bottom,
                    tile_origin,
                )
            }
            3 => (MapOrientation::Vertical, HalfTilePosition::Top, half_coord),
            _ => unreachable!(),
        }
    }

    /// Ensure a half-tile exists in the grid, generating the full tile if necessary
    fn ensure_half_tile(&mut self, half_coord: HalfTileCoord) {
        if self.half_tile_grid.contains_key(&half_coord) {
            return;
        }

        let (orientation, position, tile_origin) = self.calculate_half_tile_position(half_coord);

        // Check if the tile origin already has a half-tile (meaning the full tile was already generated)
        if let Some(existing_tile) = self.half_tile_grid.get(&tile_origin).cloned() {
            // Use the existing tile data for this half
            let half_tile = HalfTile {
                position,
                edges: existing_tile.edges,
                pixels: existing_tile.pixels.clone(),
                width: existing_tile.width,
                height: existing_tile.height,
            };
            self.half_tile_grid.insert(half_coord, half_tile);
            return;
        }

        // Generate the full tile with constraints
        let (edges, pixels, width, height) =
            self.generate_tile_with_constraints(orientation, tile_origin);

        // Store both halves of the tile in the grid
        let (first_coord, first_position, second_coord, second_position) = match orientation {
            MapOrientation::Horizontal => (
                tile_origin,
                HalfTilePosition::Left,
                HalfTileCoord {
                    x: tile_origin.x + 1,
                    y: tile_origin.y,
                },
                HalfTilePosition::Right,
            ),
            MapOrientation::Vertical => (
                tile_origin,
                HalfTilePosition::Top,
                HalfTileCoord {
                    x: tile_origin.x,
                    y: tile_origin.y + 1,
                },
                HalfTilePosition::Bottom,
            ),
        };

        let first_half = HalfTile {
            position: first_position,
            edges,
            pixels: pixels.clone(),
            width,
            height,
        };

        let second_half = HalfTile {
            position: second_position,
            edges,
            pixels,
            width,
            height,
        };

        self.half_tile_grid.insert(first_coord, first_half);
        self.half_tile_grid.insert(second_coord, second_half);
    }

    /// Generate a tile with constraints based on neighboring tiles
    /// Returns (edges, pixels, width, height)
    fn generate_tile_with_constraints(
        &mut self,
        orientation: MapOrientation,
        tile_origin: HalfTileCoord,
    ) -> (crate::maps::MapEdges, Vec<u8>, i32, i32) {
        // Calculate edge constraints based on neighboring tiles
        let constraints = self.calculate_tile_edge_constraints(orientation, tile_origin);

        // Query for a matching tile
        let mut query = MapQuery::new().with_orientation(orientation);
        for (i, constraint) in constraints.iter().enumerate() {
            query = query.with_constraint(i, *constraint);
        }

        // Get random matching tile using seeded RNG
        use rand::SeedableRng;
        let tile_x = tile_origin.x * 32;
        let tile_y = tile_origin.y * 32;
        let seed = self.seed ^ ((tile_x as u64) << 32) ^ (tile_y as u64);
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        if let Some(map) = self.map_set.get_random(&query, &mut rng) {
            (map.edges, map.pixels.clone(), map.width, map.height)
        } else {
            // Fallback: generate a simple tile
            eprintln!(
                "Warning: No matching tile found for {:?} at ({}, {})",
                orientation, tile_x, tile_y
            );
            let (pixels, width, height) = self.generate_fallback_tile_data(orientation);
            (
                crate::maps::MapEdges::new([EdgeType::Tunnel; 6]),
                pixels,
                width,
                height,
            )
        }
    }

    /// Calculate edge constraints for a tile based on its neighbors in the herringbone pattern
    ///
    /// Herringbone pattern adjacencies (where H=Horizontal, V=Vertical):
    /// - Horizontal tiles have V tiles on left/right, mixed H/V on top/bottom
    /// - Vertical tiles have H tiles on top/bottom, mixed V/H on left/right
    fn calculate_tile_edge_constraints(
        &self,
        orientation: MapOrientation,
        tile_origin: HalfTileCoord,
    ) -> [EdgeConstraint; 6] {
        use rand::SeedableRng;

        let mut constraints = [EdgeConstraint::Any; 6];
        let x = tile_origin.x;
        let y = tile_origin.y;

        match orientation {
            MapOrientation::Horizontal => {
                // Horizontal tile (64x32) edges: [top_left, top_right, bottom_left, bottom_right, left, right]
                // Tile spans half-tiles (x, y) and (x+1, y)

                // Left edge (index 4): touches Vertical tile at (x-1, y)
                // That V tile has origin at (x-1, y), our left touches its right_top (index 2)
                let left_neighbor = HalfTileCoord { x: x - 1, y };
                if let Some(tile) = self.half_tile_grid.get(&left_neighbor) {
                    // V tile edges: [left_top, left_bottom, right_top, right_bottom, top, bottom]
                    constraints[4] = EdgeConstraint::Compatible(tile.edges.edges[2]);
                    // right_top
                }

                // Right edge (index 5): touches Vertical tile at (x+2, y)
                // That V tile has origin at (x+2, y-1), position is Bottom half
                // Our right touches its left_bottom (index 1)
                let right_neighbor = HalfTileCoord { x: x + 2, y };
                if let Some(tile) = self.half_tile_grid.get(&right_neighbor) {
                    // V tile edges: [left_top, left_bottom, right_top, right_bottom, top, bottom]
                    constraints[5] = EdgeConstraint::Compatible(tile.edges.edges[1]);
                    // left_bottom
                }

                // Top-left edge (index 0): touches H tile at (x, y-1) which is the right half
                // That H tile has origin at (x-1, y-1), our top_left touches its bottom_right (index 3)
                let top_left_neighbor = HalfTileCoord { x, y: y - 1 };
                if let Some(tile) = self.half_tile_grid.get(&top_left_neighbor) {
                    // H tile edges: [top_left, top_right, bottom_left, bottom_right, left, right]
                    constraints[0] = EdgeConstraint::Compatible(tile.edges.edges[3]);
                    // bottom_right
                }

                // Top-right edge (index 1): touches V tile at (x+1, y-1) which is the bottom half
                // That V tile has origin at (x+1, y-2), our top_right touches its bottom (index 5)
                let top_right_neighbor = HalfTileCoord { x: x + 1, y: y - 1 };
                if let Some(tile) = self.half_tile_grid.get(&top_right_neighbor) {
                    // V tile edges: [left_top, left_bottom, right_top, right_bottom, top, bottom]
                    constraints[1] = EdgeConstraint::Compatible(tile.edges.edges[5]);
                    // bottom
                }

                // Bottom-left edge (index 2): touches V tile at (x, y+1) which is the top half
                // That V tile has origin at (x, y+1), our bottom_left touches its top (index 4)
                let bottom_left_neighbor = HalfTileCoord { x, y: y + 1 };
                if let Some(tile) = self.half_tile_grid.get(&bottom_left_neighbor) {
                    // V tile edges: [left_top, left_bottom, right_top, right_bottom, top, bottom]
                    constraints[2] = EdgeConstraint::Compatible(tile.edges.edges[4]);
                    // top
                }

                // Bottom-right edge (index 3): touches H tile at (x+1, y+1) which is the left half
                // That H tile has origin at (x+1, y+1), our bottom_right touches its top_left (index 0)
                let bottom_right_neighbor = HalfTileCoord { x: x + 1, y: y + 1 };
                if let Some(tile) = self.half_tile_grid.get(&bottom_right_neighbor) {
                    // H tile edges: [top_left, top_right, bottom_left, bottom_right, left, right]
                    constraints[3] = EdgeConstraint::Compatible(tile.edges.edges[0]);
                    // top_left
                }
            }
            MapOrientation::Vertical => {
                // Vertical tile (32x64) edges: [left_top, left_bottom, right_top, right_bottom, top, bottom]
                // Tile spans half-tiles (x, y) and (x, y+1)

                // Top edge (index 4): touches H tile at (x, y-1) which is the right half
                // That H tile has origin at (x-1, y-1), our top touches its bottom_right (index 3)
                let top_neighbor = HalfTileCoord { x, y: y - 1 };
                if let Some(tile) = self.half_tile_grid.get(&top_neighbor) {
                    // H tile edges: [top_left, top_right, bottom_left, bottom_right, left, right]
                    constraints[4] = EdgeConstraint::Compatible(tile.edges.edges[3]);
                    // bottom_right
                }

                // Bottom edge (index 5): touches H tile at (x, y+2) which is the left half
                // That H tile has origin at (x, y+2), our bottom touches its top_left (index 0)
                let bottom_neighbor = HalfTileCoord { x, y: y + 2 };
                if let Some(tile) = self.half_tile_grid.get(&bottom_neighbor) {
                    // H tile edges: [top_left, top_right, bottom_left, bottom_right, left, right]
                    constraints[5] = EdgeConstraint::Compatible(tile.edges.edges[0]);
                    // top_left
                }

                // Left-top edge (index 0): touches V tile at (x-1, y) which is bottom half
                // That V tile has origin at (x-1, y-1), our left_top touches its right_bottom (index 3)
                let left_top_neighbor = HalfTileCoord { x: x - 1, y };
                if let Some(tile) = self.half_tile_grid.get(&left_top_neighbor) {
                    // V tile edges: [left_top, left_bottom, right_top, right_bottom, top, bottom]
                    constraints[0] = EdgeConstraint::Compatible(tile.edges.edges[3]);
                    // right_bottom
                }

                // Left-bottom edge (index 1): touches H tile at (x-1, y+1) which is right half
                // That H tile has origin at (x-2, y+1), our left_bottom touches its right (index 5)
                let left_bottom_neighbor = HalfTileCoord { x: x - 1, y: y + 1 };
                if let Some(tile) = self.half_tile_grid.get(&left_bottom_neighbor) {
                    // H tile edges: [top_left, top_right, bottom_left, bottom_right, left, right]
                    constraints[1] = EdgeConstraint::Compatible(tile.edges.edges[5]);
                    // right
                }

                // Right-top edge (index 2): touches H tile at (x+1, y) which is left half
                // That H tile has origin at (x+1, y), our right_top touches its left (index 4)
                let right_top_neighbor = HalfTileCoord { x: x + 1, y };
                if let Some(tile) = self.half_tile_grid.get(&right_top_neighbor) {
                    // H tile edges: [top_left, top_right, bottom_left, bottom_right, left, right]
                    constraints[2] = EdgeConstraint::Compatible(tile.edges.edges[4]);
                    // left
                }

                // Right-bottom edge (index 3): touches V tile at (x+1, y+1) which is top half
                // That V tile has origin at (x+1, y+1), our right_bottom touches its left_top (index 0)
                let right_bottom_neighbor = HalfTileCoord { x: x + 1, y: y + 1 };
                if let Some(tile) = self.half_tile_grid.get(&right_bottom_neighbor) {
                    // V tile edges: [left_top, left_bottom, right_top, right_bottom, top, bottom]
                    constraints[3] = EdgeConstraint::Compatible(tile.edges.edges[0]);
                    // left_top
                }
            }
        }

        // For unconstrained edges, use 75% tunnel / 25% cavern
        // Use seeded RNG based on tile position for deterministic results
        let constraint_seed = self.seed.wrapping_add(0xDEAD)
            ^ ((tile_origin.x as u64) << 32)
            ^ (tile_origin.y as u64);
        let mut rng = rand::rngs::StdRng::seed_from_u64(constraint_seed);

        for constraint in &mut constraints {
            if matches!(constraint, EdgeConstraint::Any) {
                let edge_type = if rng.gen_bool(0.75) {
                    EdgeType::Tunnel
                } else {
                    EdgeType::Cavern
                };
                *constraint = EdgeConstraint::Compatible(edge_type);
            }
        }

        constraints
    }

    /// Generate a fallback tile when no matching tile is found
    fn generate_fallback_tile_data(&self, orientation: MapOrientation) -> (Vec<u8>, i32, i32) {
        let (width, height) = match orientation {
            MapOrientation::Horizontal => (64, 32),
            MapOrientation::Vertical => (32, 64),
        };

        // Create a simple pattern: alternating stone and air
        let mut pixels = Vec::new();
        for _y in 0..height {
            for _x in 0..width {
                // Simple checkerboard: stone (0,0,0) or air (255,255,255)
                if (_x + _y) % 2 == 0 {
                    pixels.push(0);
                    pixels.push(0);
                    pixels.push(0);
                } else {
                    pixels.push(255);
                    pixels.push(255);
                    pixels.push(255);
                }
            }
        }

        (pixels, width, height)
    }

    pub fn generate_chunk(&mut self, coord: ChunkCoord) -> Chunk {
        return if coord.x < 0 {
            self.generate_ocean_chunk(coord)
        } else if coord.x == 0 && (coord.y == -1 || coord.y == 0) {
            self.generate_beach_chunk(coord)
        } else if coord.y == -1 {
            self.generate_hill_chunk(coord)
        } else if coord.y < 0 {
            self.generate_sky_chunk(coord)
        } else {
            self.generate_tunnels_chunk(coord)
        };
    }

    pub fn generate_hill_chunk(&mut self, coord: ChunkCoord) -> Chunk {
        let mut chunk = Chunk::new(coord);
        let chunk_size: i32 = CHUNK_SIZE.try_into().unwrap();
        let noise_detail = 100;
        let noise_height = 10;

        // Fade distance: distance over which terrain height fades in from zero
        let fade_distance = 10.0;
        let fade_start = 64.0;

        // Starts to show flaws if x>14000
        for lx in 0..CHUNK_SIZE {
            let wx = coord.x * chunk_size + lx as i32;
            let wx_noise = self.noise.get([wx as f64 * (1.0 / noise_detail as f64)]);

            // Calculate fade factor: 0.0 at wx=0, gradually increasing to 1.0 at fade_distance
            let fade_factor = ((wx as f64 - fade_start) / fade_distance).min(1.0);

            // Apply fade factor to noise, ensuring smooth transition from zero
            let height_f = noise_height as f64 * wx_noise;
            let desired_height = height_f as i32 - coord.x + BEACH_HEIGHT;

            // Lerp between BEACH_HEIGHT and desired_height using fade_factor
            // At fade_factor=0.0 (wx=0): height = BEACH_HEIGHT
            // At fade_factor=1.0 (wx>=fade_distance): height = desired_height
            let height = (BEACH_HEIGHT as f64 * (1.0 - fade_factor)
                + desired_height as f64 * fade_factor) as i32;
            for ly in 0..CHUNK_SIZE {
                let wy = coord.y * chunk_size + ly as i32;

                let block_type = if wy == height {
                    Block::Grass
                } else if wy > height {
                    Block::Dirt
                } else {
                    Block::Air
                };

                chunk.set(lx, ly, block_type);
            }
        }

        // Generate chunk below to find cave openings
        let below_coord = ChunkCoord {
            x: coord.x,
            y: coord.y + 1,
        };
        let below_chunk = self.generate_tunnels_chunk(below_coord);

        // Find air pockets at the top of the chunk below (y = 0)
        let mut cave_openings = Vec::new();
        for lx in 0..CHUNK_SIZE {
            if below_chunk.get(lx, 0) == Block::Air {
                // Check if this is the start of an air pocket (not already in a run)
                if lx == 0 || below_chunk.get(lx - 1, 0) != Block::Air {
                    // Find the width of this air pocket
                    let mut width = 1;
                    while lx + width < CHUNK_SIZE && below_chunk.get(lx + width, 0) == Block::Air {
                        width += 1;
                    }
                    cave_openings.push((lx, width));
                }
            }
        }

        // Create angled shafts from cave openings with varying slopes
        let mut rng = StdRng::seed_from_u64(
            self.seed
                .wrapping_add(coord.x as u64)
                .wrapping_add(coord.y as u64),
        );
        for (start_x, width) in cave_openings {
            // Decide if this opening should reach the surface (10% chance)
            let reaches_surface = rng.gen_range(0..10) == 0;

            // Random horizontal direction for the shaft (-1 or 1)
            let direction = if rng.gen_bool(0.5) { 1 } else { -1 };

            // Random slope: 0 = 2H:1V (gentle), 1 = 1H:1V (45°), 2 = 1H:2V (steep)
            let slope_type = rng.gen_range(0..3);

            // Partial shaft that doesn't reach surface
            let shaft_length = rng.gen_range(5..20).min(CHUNK_SIZE);
            let mut current_center = start_x as f32 + (width as f32 / 2.0);
            let mut step_counter = 0;

            for ly in (CHUNK_SIZE - shaft_length..CHUNK_SIZE).rev() {
                // Preserve width throughout the shaft
                let current_width = width;

                // Move horizontally based on slope type
                match slope_type {
                    0 => {
                        // 2 horizontal per 1 vertical (gentle slope)
                        current_center += direction as f32 * 2.0;
                    }
                    1 => {
                        // 1 horizontal per 1 vertical (45 degrees)
                        current_center += direction as f32;
                    }
                    2 => {
                        // 1 horizontal per 2 vertical (steep slope)
                        if step_counter % 2 == 0 {
                            current_center += direction as f32;
                        }
                        step_counter += 1;
                    }
                    _ => {}
                }

                let row_start = (current_center - (current_width as f32 / 2.0)).round() as i32;

                for dx in 0..current_width {
                    let x = row_start + dx as i32;
                    if x >= 0 && x < CHUNK_SIZE as i32 {
                        chunk.set(x as usize, ly, Block::Air);
                    }
                }
            }
        }

        // Add trees with 1/100 chance per x coordinate
        let mut tree_rng =
            StdRng::seed_from_u64(self.seed.wrapping_add(coord.x as u64).wrapping_add(1000));
        for lx in 0..CHUNK_SIZE {
            let wx = coord.x * chunk_size + lx as i32;

            // 2% chance to spawn a tree at this x coordinate
            if tree_rng.gen_range(0..100) < 2 {
                let height_f =
                    noise_height as f64 * self.noise.get([wx as f64 * (1.0 / noise_detail as f64)]);
                let ground_height = height_f as i32 - coord.x + BEACH_HEIGHT;

                // Check if the tree would be spawning over air (cave opening)
                let ground_y = (ground_height - coord.y * chunk_size) as usize;
                let block_below = if ground_y + 1 < CHUNK_SIZE {
                    chunk.get(lx, ground_y + 1)
                } else {
                    Block::Dirt // Assume solid if at chunk boundary
                };

                // Only spawn tree if there's solid ground below
                if block_below != Block::Air {
                    let tree_height = tree_rng.gen_range(8..15);
                    let tree_width = tree_rng.gen_range(3..6);
                    self.add_tree(
                        &mut chunk,
                        wx,
                        ground_height,
                        tree_width,
                        tree_height,
                        coord,
                    );
                }
            }
        }

        // Add flax flowers and canola on grass tiles (20% chance per grass block)
        let mut flax_rng =
            StdRng::seed_from_u64(self.seed.wrapping_add(coord.x as u64).wrapping_add(2000));
        for lx in 0..CHUNK_SIZE {
            for ly in 0..CHUNK_SIZE {
                // Check if this is a grass block
                if chunk.get(lx, ly) == Block::Grass {
                    // Check if the block above is air
                    if ly > 0 && chunk.get(lx, ly - 1) == Block::Air {
                        // 10% chance to spawn flax
                        if flax_rng.gen_range(0..100) < 10 {
                            chunk.set(lx, ly - 1, Block::Flax);
                        }
                        if flax_rng.gen_range(10..110) < 20 {
                            chunk.set(lx, ly - 1, Block::Canola);
                        }
                    }
                }
            }
        }

        chunk
    }

    pub fn generate_tunnels_chunk(&mut self, coord: ChunkCoord) -> Chunk {
        let mut chunk = Chunk::new(coord);

        // Get the 4 half-tile coordinates for this chunk
        let half_coords = self.get_half_tile_coords_for_chunk(coord);

        // Ensure all half-tiles exist (generates missing tiles)
        for half_coord in &half_coords {
            self.ensure_half_tile(*half_coord);
        }

        // Now render each half-tile into the chunk
        for (i, half_coord) in half_coords.iter().enumerate() {
            let half_tile = self.half_tile_grid.get(half_coord).unwrap();

            // Determine which 32x32 quadrant of the chunk this half-tile fills
            let chunk_x = (i % 2) * 32; // 0 or 32
            let chunk_y = (i / 2) * 32; // 0 or 32

            // Determine which pixels from the tile to read
            let (tile_src_x, tile_src_y) = match half_tile.position {
                HalfTilePosition::Left => (0, 0),
                HalfTilePosition::Right => (32, 0),
                HalfTilePosition::Top => (0, 0),
                HalfTilePosition::Bottom => (0, 32),
            };

            // Copy 32x32 pixels from tile to chunk
            for dy in 0..32 {
                for dx in 0..32 {
                    let tile_px = tile_src_x + dx;
                    let tile_py = tile_src_y + dy;
                    let chunk_px = chunk_x + dx as usize;
                    let chunk_py = chunk_y + dy as usize;

                    if chunk_px < CHUNK_SIZE && chunk_py < CHUNK_SIZE {
                        let (r, g, b) = half_tile.get_pixel(tile_px, tile_py);
                        let block = Self::color_to_block(r, g, b);
                        chunk.set(chunk_px, chunk_py, block);
                    }
                }
            }
        }

        // Fix spikes after all pixels are placed
        Self::fix_spikes(&mut chunk);

        let chunk_seed = ((coord.x as u64) << 32) | (coord.y as u64);
        let mut chunk_rng = StdRng::seed_from_u64(chunk_seed);
        let a = coord.x.abs();
        let b = coord.y.abs();
        let idx = (a + b) * (a + b + 1) / 2 + a;
        let mut vein_counter = 0u32;
        for block in Block::all() {
            // TODO: seperate ore spawn data from the block enum
            if let Some(spawn_data) = block.get_ore_spawn_data() {
                let vein_count =
                    calculate_vein_count(coord.y * CHUNK_SIZE as i32, &spawn_data, &mut chunk_rng);

                for i in 0..vein_count {
                    let (x, y) = halton_2d(vein_counter + idx as u32);
                    add_ore_vein(
                        &mut chunk,
                        x as i32,
                        y as i32,
                        *block,
                        spawn_data.min_vein_size,
                        spawn_data.max_vein_size,
                    );
                    vein_counter += 1;
                }
            }
        }

        chunk
    }

    /// Convert RGB color from map image to Block type
    /// Note: Gray pixels (127,127,127) return Stalagmite as a placeholder,
    /// use fix_spikes() after filling the chunk to correct stalactite/stalagmite placement
    fn color_to_block(r: u8, g: u8, b: u8) -> Block {
        if r == 0 && g == 0 && b == 0 {
            Block::Stone
        } else if r == 127 && g == 127 && b == 127 {
            // Placeholder - will be fixed by fix_spikes()
            Block::Stalagmite
        } else if r == 255 && g == 255 && b == 255 {
            Block::Air
        } else if r == 0 && g == 149 && b == 199 {
            Block::Tide
        } else {
            unimplemented!("We have unknown pixel of color r={} g={} b={}", r, g, b)
        }
    }

    /// Fix spike blocks by checking if they should be stalactites (block above is solid)
    fn fix_spikes(chunk: &mut Chunk) {
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if chunk.get(x, y).is_spike() {
                    let above_solid = if y > 0 {
                        chunk.get(x, y - 1).is_solid()
                    } else {
                        false
                    };
                    if above_solid {
                        chunk.set(x, y, Block::Stalactite);
                    } else {
                        chunk.set(x, y, Block::Stalagmite);
                    }
                }
            }
        }
    }

    pub fn generate_ocean_chunk(&mut self, coord: ChunkCoord) -> Chunk {
        let mut chunk = Chunk::new(coord);
        let chunk_size: i32 = CHUNK_SIZE.try_into().unwrap();

        for lx in 0..CHUNK_SIZE {
            for ly in 0..CHUNK_SIZE {
                let wy = coord.y * chunk_size + ly as i32;
                let block_type = if wy > SEA_FLOOR {
                    Block::Sand
                } else if wy > SEA_LEVEL {
                    Block::Water
                } else {
                    Block::Air
                };

                chunk.set(lx, ly, block_type);
            }
        }

        chunk
    }

    pub fn generate_sky_chunk(&mut self, coord: ChunkCoord) -> Chunk {
        let mut chunk = Chunk::new(coord);
        for lx in 0..CHUNK_SIZE {
            for ly in 0..CHUNK_SIZE {
                chunk.set(lx, ly, Block::Air);
            }
        }
        chunk
    }

    pub fn generate_beach_chunk(&mut self, coord: ChunkCoord) -> Chunk {
        use rand::SeedableRng;

        let mut chunk = Chunk::new(coord);
        let chunk_size: i32 = CHUNK_SIZE as i32;

        let inflection_x = 32;
        let little_slope = -0.25;
        let big_slope = -2.0;
        let offset_right = BEACH_HEIGHT as f32 - (chunk_size - inflection_x) as f32 * little_slope;
        let offset_left = offset_right - big_slope * inflection_x as f32;

        // Create seeded RNG for clam placement
        let clam_seed = self.seed ^ ((coord.x as u64) << 32) ^ (coord.y as u64) ^ 0xC1A3;
        let mut rng = rand::rngs::StdRng::seed_from_u64(clam_seed);

        for lx in 0..CHUNK_SIZE {
            let wx = coord.x * chunk_size + lx as i32;
            for ly in 0..CHUNK_SIZE {
                let wy = coord.y * chunk_size + ly as i32;
                let block_type = if wy > SEA_FLOOR {
                    Block::Sand
                } else if wx <= inflection_x {
                    if ((offset_left + wx as f32 * big_slope) as i32) < wy {
                        Block::Sand
                    } else {
                        if wy > SEA_LEVEL {
                            Block::Water
                        } else {
                            Block::Air
                        }
                    }
                } else {
                    if ((offset_right + (wx - inflection_x) as f32 * little_slope) as i32) < wy {
                        Block::Sand
                    } else {
                        if wy > SEA_LEVEL {
                            Block::Water
                        } else {
                            Block::Air
                        }
                    }
                };

                chunk.set(lx, ly, block_type);
            }
        }

        // Add clams on air tiles above sand below sea level (underwater)
        for lx in 0..CHUNK_SIZE {
            // ly + 1 must be valid, so stop at CHUNK_SIZE - 1
            for ly in 0..(CHUNK_SIZE - 1) {
                let wy = coord.y * chunk_size + ly as i32;
                // Only spawn below sea level (wy > SEA_LEVEL means underwater)
                if wy <= SEA_LEVEL + 3 {
                    continue;
                }
                // Check if current tile is air and tile below is sand
                if chunk.get(lx, ly) == Block::Air && chunk.get(lx, ly + 1) == Block::Sand {
                    // 20% chance to spawn a clam
                    if rng.gen_bool(0.2) {
                        chunk.set(lx, ly, Block::Clam);
                    }
                }
            }
        }

        chunk
    }

    fn add_tree(
        &self,
        chunk: &mut Chunk,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        chunk_coord: ChunkCoord,
    ) {
        if height < 2 || width < 1 {
            return;
        }

        let trunk_height = (height as f32 * 0.6).ceil() as i32;
        let canopy_height = height - trunk_height;

        // Trunk
        for i in 0..trunk_height {
            let trunk_y = y - 1 - i;

            if self.is_in_chunk(x, trunk_y, chunk_coord) {
                let (lx, ly) = self.world_to_local(x, trunk_y, chunk_coord);
                chunk.set(lx, ly, Block::Log);
            }
        }

        // Leaves
        let canopy_base_y = y - trunk_height;
        for layer in 0..canopy_height {
            let canopy_y = canopy_base_y - layer;

            let layer_ratio = 1.0 - (layer as f32 / canopy_height as f32) * 0.5;
            let layer_width = ((width as f32 * layer_ratio).ceil() as i32).max(1);

            let layer_width = if layer_width % 2 == 0 {
                layer_width + 1
            } else {
                layer_width
            };
            let half_width = layer_width / 2;

            for dx in -half_width..=half_width {
                let leaves_x = x + dx;

                if self.is_in_chunk(leaves_x, canopy_y, chunk_coord) {
                    let (lx, ly) = self.world_to_local(leaves_x, canopy_y, chunk_coord);
                    chunk.set(lx, ly, Block::Leaves);
                }
            }
        }
    }

    fn is_in_chunk(&self, wx: i32, wy: i32, chunk_coord: ChunkCoord) -> bool {
        let chunk_size = CHUNK_SIZE as i32;
        let chunk_start_x = chunk_coord.x * chunk_size;
        let chunk_end_x = chunk_start_x + chunk_size;
        let chunk_start_y = chunk_coord.y * chunk_size;
        let chunk_end_y = chunk_start_y + chunk_size;

        wx >= chunk_start_x && wx < chunk_end_x && wy >= chunk_start_y && wy < chunk_end_y
    }

    fn world_to_local(&self, wx: i32, wy: i32, chunk_coord: ChunkCoord) -> (usize, usize) {
        let chunk_size = CHUNK_SIZE as i32;
        let lx = (wx - chunk_coord.x * chunk_size) as usize;
        let ly = (wy - chunk_coord.y * chunk_size) as usize;
        (lx, ly)
    }
}
