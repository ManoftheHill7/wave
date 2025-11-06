use std::collections::HashMap;
use crate::terrain_generator::TerrainGenerator;

pub const CHUNK_SIZE: usize = 128;
pub const CELL_RESOLUTION: usize = 1;
pub const CELLS_PER_TILE: usize = CELL_RESOLUTION * CELL_RESOLUTION;
pub const CELL_OFFSET: f32 = 1.0 / CELL_RESOLUTION as f32;
pub const NO_LIQUID_THRESHOLD: f32 = 0.0001;

const FLOW_RATE: f32 = 1.0;
const PRESSURIZED_VOLUME: f32 = 1.1;
const MIN_FLOW: f32 = NO_LIQUID_THRESHOLD;

// Block types enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block {
    Air,
    Dirt,
    Stone,
    Grass,
    Sand,
    Water,
    Lava,
    Log,
    Leaf,
}

#[derive(Debug, Clone, Copy)]
pub struct FlowData {
    pub up: f32,
    pub down: f32,
    pub left: f32,
    pub right: f32,
}

impl FlowData {
    fn zero() -> Self {
        FlowData {
            up: 0.0,
            down: 0.0,
            left: 0.0,
            right: 0.0,
        }
    }
}

impl Block {
    pub fn is_solid(self) -> bool {
        matches!(self,
            Block::Dirt |
            Block::Stone |
            Block::Grass |
            Block::Sand |
            Block::Log |
            Block::Leaf)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LiquidData {
    pub volume: f32, // 0.0 is none, 1.0 is full, more then 1.0 is pressurized
    pub flow: FlowData,
    pub flow_left: bool,
    pub flow_right: bool,
    pub flow_down: bool,
    pub flow_up: bool,
}

impl LiquidData {
    fn new(volume: f32) -> Self {
        LiquidData {
            volume,
            flow: FlowData::zero(),
            flow_left: false,
            flow_right: false,
            flow_down: false,
            flow_up: false,
        }
    }
}

// Chunk coordinate (not block coordinate)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone)]
pub struct Chunk {
    blocks: [Block; CHUNK_SIZE * CHUNK_SIZE],
    cells: [LiquidData; CELLS_PER_TILE * CHUNK_SIZE * CHUNK_SIZE],
    pub coord: ChunkCoord,
}

impl Chunk {
    pub fn new(coord: ChunkCoord) -> Self {
        Chunk {
            blocks: [Block::Air; CHUNK_SIZE * CHUNK_SIZE],
            cells: [LiquidData::new(0.0); CELLS_PER_TILE * CHUNK_SIZE * CHUNK_SIZE],
            coord,
        }
    }

    pub fn get(&self, local_x: usize, local_y: usize) -> Block {
        let index = local_y * CHUNK_SIZE + local_x;
        self.blocks[index]
    }

    pub fn set(&mut self, local_x: usize, local_y: usize, block: Block) {
        let index = local_y * CHUNK_SIZE + local_x;
        if block == Block::Water {
            let _w = CHUNK_SIZE ;
            let lx = local_x as f32;
            let ly = local_y as f32;
            for cell_y in 0..CELL_RESOLUTION {
                for cell_x in 0..CELL_RESOLUTION {
                    self.liquid_set(lx as f32 + cell_x as f32 / CELL_RESOLUTION as f32,
                        ly as f32 + cell_y as f32 / CELL_RESOLUTION as f32,
                        LiquidData::new(PRESSURIZED_VOLUME));
                }
            }
        } else {
            self.blocks[index] = block;
        }
    }

    pub fn liquid_get(&self, local_x: f32, local_y: f32) -> LiquidData {
        let cx = local_x * CELL_RESOLUTION as f32;
        let cy = local_y * CELL_RESOLUTION as f32;
        self.cells[(cy * CHUNK_SIZE as f32 * CELL_RESOLUTION as f32 + cx) as usize]
    }

    pub fn liquid_set(&mut self, local_x: f32, local_y: f32, ld: LiquidData) {
        let cx = local_x * CELL_RESOLUTION as f32;
        let cy = local_y * CELL_RESOLUTION as f32;
        self.cells[(cy * CHUNK_SIZE as f32 * CELL_RESOLUTION as f32 + cx) as usize] = ld
    }

    pub unsafe fn flow(&mut self,
                       mut neighbor_up: Option<&mut Chunk>,
                       mut neighbor_down: Option<&mut Chunk>,
                       mut neighbor_left: Option<&mut Chunk>,
                       mut neighbor_right: Option<&mut Chunk>) {
        let cells_per_chunk_axis = CHUNK_SIZE * CELL_RESOLUTION;


        // Step 1: Calculate flow values for each cell
        for cell_y in 0..cells_per_chunk_axis {
            for cell_x in 0..cells_per_chunk_axis {
                let cell_index = cell_y * cells_per_chunk_axis + cell_x;

                self.cells[cell_index].flow = FlowData::zero();
                self.cells[cell_index].flow_down = false;
                self.cells[cell_index].flow_up = false;
                self.cells[cell_index].flow_left = false;
                self.cells[cell_index].flow_right = false;

                let current_volume = self.cells[cell_index].volume;
                if current_volume < NO_LIQUID_THRESHOLD {
                    continue;
                }

                let tile_x = cell_x / CELL_RESOLUTION;
                let tile_y = cell_y / CELL_RESOLUTION;
                let tile_index = tile_y * CHUNK_SIZE + tile_x;
                let current_blocked = self.blocks[tile_index].is_solid();

                if current_blocked {
                    continue;
                }

                macro_rules! get_neighbor_volume {
                    (down) => { get_neighbor_volume!(cell_y as i32 + 1 < cells_per_chunk_axis as i32, neighbor_down,
                        tile_x,
                        cell_x,
                        (cell_y as usize + 1) / CELL_RESOLUTION * CHUNK_SIZE + tile_x,
                        ((cell_y as i32 + 1) as usize) * cells_per_chunk_axis + cell_x
                    )};
                    (up) => { get_neighbor_volume!(cell_y as i32 - 1 >= 0, neighbor_up,
                        (CHUNK_SIZE - 1) * CHUNK_SIZE + tile_x,
                        (cells_per_chunk_axis - 1) * cells_per_chunk_axis + cell_x,
                        (cell_y as usize - 1) / CELL_RESOLUTION * CHUNK_SIZE + tile_x,
                        ((cell_y as i32 - 1) as usize) * cells_per_chunk_axis + cell_x
                    )};
                    (left) => { get_neighbor_volume!(cell_x as i32 - 1 >= 0, neighbor_left,
                        tile_y * CHUNK_SIZE + (CHUNK_SIZE - 1),
                        cell_y * cells_per_chunk_axis + (cells_per_chunk_axis - 1),
                        tile_y * CHUNK_SIZE + ((cell_x as i32 - 1) as usize) / CELL_RESOLUTION,
                        cell_y * cells_per_chunk_axis + ((cell_x as i32 - 1) as usize)
                    )};
                    (right) => { get_neighbor_volume!(cell_x as i32 + 1 < cells_per_chunk_axis as i32, neighbor_right,
                        tile_y * CHUNK_SIZE,
                        cell_y * cells_per_chunk_axis,
                        tile_y * CHUNK_SIZE + ((cell_x as i32 + 1) as usize) / CELL_RESOLUTION,
                        cell_y * cells_per_chunk_axis + ((cell_x as i32 + 1) as usize)
                    )};

                    ($in_chunk:expr, $neighbor:expr, $cross_tile:expr, $cross_cell:expr, $tile_idx:expr, $neighbour_idx:expr) => {{
                        if $in_chunk {
                            if !self.blocks[$tile_idx].is_solid() {
                                Some(self.cells[$neighbour_idx].volume)
                            } else {
                                None
                            }
                        } else if let Some(ref chunk) = $neighbor {
                            if !chunk.blocks[$cross_tile].is_solid() {
                                Some(chunk.cells[$cross_cell].volume)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }};
                }

                let mut flow_down = 0.0;
                let mut flow_up = 0.0;
                let mut flow_left = 0.0;
                let mut flow_right = 0.0;

                // Gravity (downward)
                if let Some(down_volume) = get_neighbor_volume!(down) {
                    if down_volume < PRESSURIZED_VOLUME {
                        let available_space = PRESSURIZED_VOLUME - down_volume;
                        let transfer = FLOW_RATE.min(current_volume).min(available_space);
                        if transfer > MIN_FLOW {
                            flow_down = transfer;
                        }
                    }
                }

                // Horizontal equalization - use smaller fraction to reduce oscillation
                if let Some(left_volume) = get_neighbor_volume!(left) {
                    let diff = current_volume - left_volume;
                    if diff > NO_LIQUID_THRESHOLD {
                        let transfer = (diff * 0.25).min(current_volume);
                        if transfer > MIN_FLOW {
                            flow_left = transfer;
                        }
                    }
                }

                if let Some(right_volume) = get_neighbor_volume!(right) {
                    let diff = current_volume - right_volume;
                    if diff > NO_LIQUID_THRESHOLD {
                        let transfer = (diff * 0.25).min(current_volume);
                        if transfer > MIN_FLOW {
                            flow_right = transfer;
                        }
                    }
                }

                // Pressure (upward)
                if current_volume > 1.0 {
                    if let Some(up_volume) = get_neighbor_volume!(up) {
                        if up_volume < 1.0 {
                            let pressure = current_volume - 1.0;
                            let available_space = 1.0 - up_volume;
                            let transfer = (pressure * FLOW_RATE).min(available_space);
                            if transfer > MIN_FLOW {
                                flow_up = transfer;
                            }
                        }
                    }
                }

                // Normalize flows so total doesn't exceed current volume
                let total_flow = flow_up + flow_down + flow_left + flow_right;
                if total_flow > current_volume - MIN_FLOW {
                    let scale = (current_volume - MIN_FLOW).max(0.0) / total_flow.max(MIN_FLOW);
                    flow_up *= scale;
                    flow_down *= scale;
                    flow_left *= scale;
                    flow_right *= scale;
                }

                // Smoothly interpolate towards desired flow (momentum/inertia)
                const FLOW_SMOOTHING: f32 = 0.5; // 0 = instant, 1 = no change
                let current_flow = self.cells[cell_index].flow;

                self.cells[cell_index].flow = FlowData {
                    up: current_flow.up * FLOW_SMOOTHING + flow_up * (1.0 - FLOW_SMOOTHING),
                    down: current_flow.down * FLOW_SMOOTHING + flow_down * (1.0 - FLOW_SMOOTHING),
                    left: current_flow.left * FLOW_SMOOTHING + flow_left * (1.0 - FLOW_SMOOTHING),
                    right: current_flow.right * FLOW_SMOOTHING + flow_right * (1.0 - FLOW_SMOOTHING),
                };
            }
        }

        // Step 2: Apply flow to move water between cells
        for cell_y in (0..cells_per_chunk_axis).rev() {
            for cell_x in 0..cells_per_chunk_axis {
                let cell_index = cell_y * cells_per_chunk_axis + cell_x;
                let flow = self.cells[cell_index].flow;

                if flow.up < MIN_FLOW && flow.down < MIN_FLOW && flow.left < MIN_FLOW && flow.right < MIN_FLOW {
                    continue;
                }

                let tile_x = cell_x / CELL_RESOLUTION;
                let tile_y = cell_y / CELL_RESOLUTION;
                let tile_index = tile_y * CHUNK_SIZE + tile_x;
                let current_blocked = self.blocks[tile_index].is_solid();

                if current_blocked {
                    self.cells[cell_index].volume = 0.0;
                    continue;
                }

                macro_rules! get_neighbor_ptr {
                    (down) => { get_neighbor_ptr!(cell_y as i32 + 1 < cells_per_chunk_axis as i32, neighbor_down,
                        tile_x,
                        cell_x,
                        (cell_y as usize + 1) / CELL_RESOLUTION * CHUNK_SIZE + tile_x,
                        ((cell_y as i32 + 1) as usize) * cells_per_chunk_axis + cell_x
                    )};
                    (up) => { get_neighbor_ptr!(cell_y as i32 - 1 >= 0, neighbor_up,
                        (CHUNK_SIZE - 1) * CHUNK_SIZE + tile_x,
                        (cells_per_chunk_axis - 1) * cells_per_chunk_axis + cell_x,
                        (cell_y as usize - 1) / CELL_RESOLUTION * CHUNK_SIZE + tile_x,
                        ((cell_y as i32 - 1) as usize) * cells_per_chunk_axis + cell_x
                    )};
                    (left) => { get_neighbor_ptr!(cell_x as i32 - 1 >= 0, neighbor_left,
                        tile_y * CHUNK_SIZE + (CHUNK_SIZE - 1),
                        cell_y * cells_per_chunk_axis + (cells_per_chunk_axis - 1),
                        tile_y * CHUNK_SIZE + ((cell_x as i32 - 1) as usize) / CELL_RESOLUTION,
                        cell_y * cells_per_chunk_axis + ((cell_x as i32 - 1) as usize)
                    )};
                    (right) => { get_neighbor_ptr!(cell_x as i32 + 1 < cells_per_chunk_axis as i32, neighbor_right,
                        tile_y * CHUNK_SIZE,
                        cell_y * cells_per_chunk_axis,
                        tile_y * CHUNK_SIZE + ((cell_x as i32 + 1) as usize) / CELL_RESOLUTION,
                        cell_y * cells_per_chunk_axis + ((cell_x as i32 + 1) as usize)
                    )};

                    ($in_chunk:expr, $neighbor:expr, $cross_tile:expr, $cross_cell:expr, $tile_idx:expr, $neighbour_idx:expr) => {{
                        if $in_chunk {
                            if !self.blocks[$tile_idx].is_solid() {
                                Some(&mut self.cells[$neighbour_idx] as *mut LiquidData)
                            } else {
                                None
                            }
                        } else if let Some(ref mut chunk) = $neighbor {
                            if !chunk.blocks[$cross_tile].is_solid() {
                                Some(&mut chunk.cells[$cross_cell] as *mut LiquidData)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }};
                }

                // Apply downward flow
                if flow.down > MIN_FLOW {
                    if let Some(target_ptr) = get_neighbor_ptr!(down) {
                        (*target_ptr).volume += flow.down;
                        (*target_ptr).flow_down = true;
                        self.cells[cell_index].volume -= flow.down;
                    }
                }

                // Apply leftward flow
                if flow.left > MIN_FLOW {
                    if let Some(target_ptr) = get_neighbor_ptr!(left) {
                        (*target_ptr).volume += flow.left;
                        (*target_ptr).flow_left = true;
                        self.cells[cell_index].volume -= flow.left;
                    }
                }

                // Apply rightward flow
                if flow.right > MIN_FLOW {
                    if let Some(target_ptr) = get_neighbor_ptr!(right) {
                        (*target_ptr).volume += flow.right;
                        (*target_ptr).flow_right = true;
                        self.cells[cell_index].volume -= flow.right;
                    }
                }

                // Apply upward flow
                if flow.up > MIN_FLOW {
                    if let Some(target_ptr) = get_neighbor_ptr!(up) {
                        (*target_ptr).volume += flow.up;
                        (*target_ptr).flow_up = true;
                        self.cells[cell_index].volume -= flow.up;
                    }
                }
            }
        }
    }
}

pub struct Terrain {
    pub chunks: HashMap<ChunkCoord, Chunk>,
    chunk_size: i32,
    generator: TerrainGenerator,
}

impl Terrain {
    pub fn new(seed: u64) -> Self {
        Terrain {
            chunks: HashMap::new(),
            chunk_size: CHUNK_SIZE as i32,
            generator: TerrainGenerator::new(seed),
        }
    }

    pub fn flow(&mut self) {
        let chunk_coords: Vec<ChunkCoord> = self.chunks.keys().copied().collect();

        for chunk_coord in &chunk_coords {
            let neighbor_up_coord = ChunkCoord { x: chunk_coord.x, y: chunk_coord.y - 1 };
            let neighbor_down_coord = ChunkCoord { x: chunk_coord.x, y: chunk_coord.y + 1 };
            let neighbor_left_coord = ChunkCoord { x: chunk_coord.x - 1, y: chunk_coord.y };
            let neighbor_right_coord = ChunkCoord { x: chunk_coord.x + 1, y: chunk_coord.y };

            let mut coords = vec![*chunk_coord];
            if self.chunks.contains_key(&neighbor_up_coord) && neighbor_up_coord != *chunk_coord {
                coords.push(neighbor_up_coord);
            }
            if self.chunks.contains_key(&neighbor_down_coord) && neighbor_down_coord != *chunk_coord
                && !coords.contains(&neighbor_down_coord) {
                coords.push(neighbor_down_coord);
            }
            if self.chunks.contains_key(&neighbor_left_coord) && neighbor_left_coord != *chunk_coord
                && !coords.contains(&neighbor_left_coord) {
                coords.push(neighbor_left_coord);
            }
            if self.chunks.contains_key(&neighbor_right_coord) && neighbor_right_coord != *chunk_coord
                && !coords.contains(&neighbor_right_coord) {
                coords.push(neighbor_right_coord);
            }

            unsafe {
                let chunks_raw = &mut self.chunks as *mut HashMap<ChunkCoord, Chunk>;

                let current_chunk = (*chunks_raw).get_mut(chunk_coord)
                    .map(|c| c as *mut Chunk);

                if let Some(current_ptr) = current_chunk {
                    let neighbor_up = (*chunks_raw).get_mut(&neighbor_up_coord)
                        .map(|c| c as *mut Chunk);
                    let neighbor_down = (*chunks_raw).get_mut(&neighbor_down_coord)
                        .map(|c| c as *mut Chunk);
                    let neighbor_left = (*chunks_raw).get_mut(&neighbor_left_coord)
                        .map(|c| c as *mut Chunk);
                    let neighbor_right = (*chunks_raw).get_mut(&neighbor_right_coord)
                        .map(|c| c as *mut Chunk);

                    (*current_ptr).flow(
                        neighbor_up.map(|p| &mut *p),
                        neighbor_down.map(|p| &mut *p),
                        neighbor_left.map(|p| &mut *p),
                        neighbor_right.map(|p| &mut *p),
                    );
                }
            }
        }
    }

    pub fn at(&self, x: i32, y: i32) -> Block {
        let chunk_coord = self.world_to_chunk(x, y);
        let local_coord = self.world_to_local(x, y);

        if let Some(chunk) = self.chunks.get(&chunk_coord) {
            chunk.get(local_coord.0, local_coord.1)
        } else {
            Block::Air
        }
    }

    pub fn liquid_at(&self, x: f32, y: f32) -> LiquidData {
        let xi = x as i32;
        let yi = y as i32;
        let chunk_coord = self.world_to_chunk(xi, yi);
        let local_coord = self.world_to_local(xi, yi);

        if let Some(chunk) = self.chunks.get(&chunk_coord) {
            chunk.liquid_get(local_coord.0 as f32 + x.fract(), local_coord.1 as f32 + y.fract())
        } else {
            LiquidData::new(0.0)
        }
    }

    pub fn solid_terrain_at(&self, x: i32, y: i32) -> bool {
        self.at(x, y).is_solid()
    }

    pub fn liquid_terrain_at(&self, x: i32, y: i32) -> bool {
        // matches!(self.at(x, y), Block::Water | Block::Lava)
        self.liquid_at(x as f32, y as f32).volume +
            self.liquid_at(x as f32 + CELL_OFFSET, y as f32).volume +
            self.liquid_at(x as f32, y as f32 + CELL_OFFSET).volume +
            self.liquid_at(x as f32 + CELL_OFFSET, y as f32 + CELL_OFFSET).volume > 0.5
    }

    pub fn collides_with_solid_terrain(&self, x: f32, y: f32, width: f32, height: f32) -> Option<(f32, f32)> {
        let left = x.floor() as i32;
        let right = (x + width).floor() as i32;
        let top = y.floor() as i32;
        let bottom = (y + height).floor() as i32;

        for check_y in top..=bottom {
            for check_x in left..=right {
                if self.solid_terrain_at(check_x, check_y) {
                    return Some((check_x as f32, check_y as f32));
                }
            }
        }

        None
    }

    pub fn set(&mut self, x: i32, y: i32, block: Block) {
        let chunk_coord = self.world_to_chunk(x, y);
        let local_coord = self.world_to_local(x, y);

        if !self.chunks.contains_key(&chunk_coord) {
            self.load_chunk(chunk_coord);
        }

        if let Some(chunk) = self.chunks.get_mut(&chunk_coord) {
            chunk.set(local_coord.0, local_coord.1, block);
        }
    }

    pub fn world_to_chunk(&self, x: i32, y: i32) -> ChunkCoord {
        ChunkCoord {
            x: x.div_euclid(self.chunk_size),
            y: y.div_euclid(self.chunk_size),
        }
    }

    fn world_to_local(&self, x: i32, y: i32) -> (usize, usize) {
        (
            x.rem_euclid(self.chunk_size) as usize,
            y.rem_euclid(self.chunk_size) as usize,
        )
    }

    pub fn load_chunk(&mut self, coord: ChunkCoord) {
        let chunk = self.generator.generate_chunk(coord);
        self.chunks.insert(coord, chunk);
    }

    pub fn get_chunks_in_range(&self, center_x: i32, center_y: i32, range: i32) -> Vec<&Chunk> {
        let center_chunk = self.world_to_chunk(center_x, center_y);
        let mut chunks = Vec::new();

        for dx in -range..=range {
            for dy in -range..=range {
                let coord = ChunkCoord {
                    x: center_chunk.x + dx,
                    y: center_chunk.y + dy,
                };
                if let Some(chunk) = self.chunks.get(&coord) {
                    chunks.push(chunk);
                }
            }
        }
        chunks
    }

    /**
     * `max_distance`: radius to keep
     */
    pub fn unload_distant_chunks(&mut self, center_x: i32, center_y: i32, max_distance: i32) {
        let center_chunk = self.world_to_chunk(center_x, center_y);

        self.chunks.retain(|coord, _| {
            let dx = (coord.x - center_chunk.x).abs();
            let dy = (coord.y - center_chunk.y).abs();
            dx <= max_distance && dy <= max_distance
        });
    }
}
