use crate::biome::BiomeMap;
use crate::block::BlockType;
use serde::{Deserialize, Serialize};
use noise::{NoiseFn, Perlin};

pub const CHUNK_SIZE: usize = 16;
pub const WORLD_HEIGHT: usize = 64;
pub const SEA_LEVEL: i32 = 20;
pub const RENDER_DISTANCE: i32 = 6;

/// A 16x16x16 sub-chunk
#[derive(Clone, Serialize, Deserialize)]
pub struct SubChunk {
    pub blocks: [[[BlockType; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

impl SubChunk {
    pub fn new() -> Self {
        Self {
            blocks: [[[BlockType::Air; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
        }
    }
}

/// A column of sub-chunks (16x16 horizontal, multiple 16-high vertical)
#[derive(Clone)]
pub struct ChunkColumn {
    pub sub_chunks: Vec<SubChunk>, // indexed by y/CHUNK_SIZE
}

impl ChunkColumn {
    pub fn new() -> Self {
        let num_subs = WORLD_HEIGHT / CHUNK_SIZE;
        Self {
            sub_chunks: vec![SubChunk::new(); num_subs],
        }
    }
}

/// Coordinate types
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct ChunkPos {
    pub cx: i32,
    pub cz: i32,
}

pub struct World {
    chunks: Vec<Vec<Option<ChunkColumn>>>,
    origin_x: i32,
    origin_z: i32,
    seed: u32,
    perlin: Perlin,
    biomes: BiomeMap,
}

impl World {
    /// Create empty world (chunks generate on demand)
    pub fn new(seed: u32) -> Self {
        let grid_size = (RENDER_DISTANCE * 2 + 2) as usize;
        let chunks = vec![vec![None; grid_size]; grid_size];

        Self {
            chunks,
            origin_x: 0,
            origin_z: 0,
            seed,
            perlin: Perlin::new(seed + 100),
            biomes: BiomeMap::new(seed),
        }
    }

    /// Create world from pre-generated blocks (for save loading)
    pub fn seed(&self) -> u32 {
        self.seed
    }

    pub fn get_biome_at(&self, x: i32, z: i32) -> crate::biome::Biome {
        self.biomes.get_biome(x, z)
    }

    pub fn from_blocks(blocks: Vec<Vec<Vec<BlockType>>>) -> Self {
        let mut world = Self::new(42);

        // Convert flat blocks to chunk columns
        for cx in 0..2 {
            for cz in 0..2 {
                let mut col = ChunkColumn::new();
                for lx in 0..CHUNK_SIZE {
                    for lz in 0..CHUNK_SIZE {
                        let wx = cx * CHUNK_SIZE + lx;
                        let wz = cz * CHUNK_SIZE + lz;
                        if wx < 256 && wz < 256 {
                            for y in 0..WORLD_HEIGHT {
                                let sy = y / CHUNK_SIZE;
                                let ly = y % CHUNK_SIZE;
                                col.sub_chunks[sy].blocks[lx][lz][ly] = blocks[wx][wz][y];
                            }
                        }
                    }
                }
                world.set_chunk(ChunkPos { cx: cx as i32, cz: cz as i32 }, col);
            }
        }

        world
    }

    /// Serialize all loaded chunks back to flat block array (for save)
    pub fn blocks_ref(&self) -> Vec<Vec<Vec<BlockType>>> {
        let mut blocks = vec![
            vec![vec![BlockType::Air; WORLD_HEIGHT]; 256];
            256
        ];
        for cx_idx in 0..self.chunks.len() {
            for cz_idx in 0..self.chunks[cx_idx].len() {
                if let Some(ref col) = self.chunks[cx_idx][cz_idx] {
                    let cx = self.origin_x + cx_idx as i32;
                    let cz = self.origin_z + cz_idx as i32;
                    for lx in 0..CHUNK_SIZE {
                        for lz in 0..CHUNK_SIZE {
                            let wx = (cx * CHUNK_SIZE as i32 + lx as i32) as usize;
                            let wz = (cz * CHUNK_SIZE as i32 + lz as i32) as usize;
                            if wx < 256 && wz < 256 {
                                for sy in 0..col.sub_chunks.len() {
                                    for ly in 0..CHUNK_SIZE {
                                        let y = sy * CHUNK_SIZE + ly;
                                        if y < WORLD_HEIGHT {
                                            blocks[wx][wz][y] = col.sub_chunks[sy].blocks[lx][lz][ly];
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        blocks
    }

    fn chunk_index(&self, pos: ChunkPos) -> Option<(usize, usize)> {
        let ix = (pos.cx - self.origin_x) as usize;
        let iz = (pos.cz - self.origin_z) as usize;
        if ix < self.chunks.len() && iz < self.chunks[ix].len() {
            Some((ix, iz))
        } else {
            None
        }
    }

    fn set_chunk(&mut self, pos: ChunkPos, col: ChunkColumn) {
        if let Some((ix, iz)) = self.chunk_index(pos) {
            self.chunks[ix][iz] = Some(col);
        }
    }

    fn get_or_generate(&mut self, pos: ChunkPos) {
        if let Some((ix, iz)) = self.chunk_index(pos) {
            if self.chunks[ix][iz].is_some() {
                return;
            }
        } else {
            // Need to expand grid - for now, skip
            return;
        }
        let col = self.generate_column(pos);
        self.set_chunk(pos, col);
    }

    /// Generate a full chunk column procedurally (optimized with cached heightmap)
    fn generate_column(&self, pos: ChunkPos) -> ChunkColumn {
        let mut col = ChunkColumn::new();
        let base_x = pos.cx * CHUNK_SIZE as i32;
        let base_z = pos.cz * CHUNK_SIZE as i32;

        // Pre-compute heightmap and biome for all columns (16x16)
        let mut heights = [[0usize; CHUNK_SIZE]; CHUNK_SIZE];
        let mut biomes_arr = [[crate::biome::Biome::Plains; CHUNK_SIZE]; CHUNK_SIZE];
        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let x = base_x + lx as i32;
                let z = base_z + lz as i32;
                biomes_arr[lx][lz] = self.biomes.get_biome(x, z);
                heights[lx][lz] = self.biomes.get_height(x, z) as usize;
            }
        }

        // Phase 1: Terrain using cached data
        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let biome = biomes_arr[lx][lz];
                let height = heights[lx][lz];
                for y in 0..WORLD_HEIGHT {
                    let sy = y / CHUNK_SIZE;
                    let ly = y % CHUNK_SIZE;
                    col.sub_chunks[sy].blocks[lx][lz][ly] = if y == 0 {
                        BlockType::Stone
                    } else if height > 4 && y < height - 4 {
                        BlockType::Stone
                    } else if height > 1 && y < height - 1 {
                        biome.sub_surface_block()
                    } else if height > 0 && y == height - 1 {
                        biome.surface_block()
                    } else if y < SEA_LEVEL as usize {
                        BlockType::Water
                    } else {
                        BlockType::Air
                    };
                }
            }
        }

        // Phase 2: Caves
        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let x = base_x + lx as i32;
                let z = base_z + lz as i32;
                for y in 2..WORLD_HEIGHT - 5 {
                    let nx = x as f64 / 20.0;
                    let ny = y as f64 / 12.0;
                    let nz = z as f64 / 20.0;
                    let cave = self.perlin.get([nx, ny, nz])
                        + self.perlin.get([nx * 2.5, ny * 2.0, nz * 2.5]) * 0.5;
                    let depth_factor = y as f64 / WORLD_HEIGHT as f64;
                    let threshold = 0.45 - depth_factor * 0.15;
                    if cave > threshold {
                        let sy = y / CHUNK_SIZE;
                        let ly = y % CHUNK_SIZE;
                        let block = col.sub_chunks[sy].blocks[lx][lz][ly];
                        if block != BlockType::Air && block != BlockType::Water {
                            col.sub_chunks[sy].blocks[lx][lz][ly] = BlockType::Air;
                        }
                    }
                }
            }
        }

        // Phase 3: Vegetation
        let mut rng = self.seed as u64
            + (pos.cx as u64).wrapping_mul(374761393)
            + (pos.cz as u64).wrapping_mul(668265263);
        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                let r = (rng >> 33) as u32;
                let _x = base_x + lx as i32;
                let _z = base_z + lz as i32;

                let surface_y = (0..WORLD_HEIGHT).rev().find(|&y| {
                    let sy = y / CHUNK_SIZE;
                    let ly = y % CHUNK_SIZE;
                    col.sub_chunks[sy].blocks[lx][lz][ly].is_solid()
                });
                if let Some(sy_val) = surface_y {
                    let block_sy = sy_val / CHUNK_SIZE;
                    let block_ly = sy_val % CHUNK_SIZE;
                    if col.sub_chunks[block_sy].blocks[lx][lz][block_ly] == BlockType::Grass
                        && sy_val + 1 < WORLD_HEIGHT
                    {
                        let above_sy = (sy_val + 1) / CHUNK_SIZE;
                        let above_ly = (sy_val + 1) % CHUNK_SIZE;
                        match r % 100 {
                            0..=3 => col.sub_chunks[above_sy].blocks[lx][lz][above_ly] = BlockType::Flower,
                            4..=12 => col.sub_chunks[above_sy].blocks[lx][lz][above_ly] = BlockType::TallGrass,
                            _ => {}
                        }
                    }
                }
            }
        }

        // Phase 4: Trees (biome-aware density)
        let mut rng2 = self.seed as u64 + 999
            + (pos.cx as u64).wrapping_mul(1234567)
            + (pos.cz as u64).wrapping_mul(7654321);
        for lx in 4..CHUNK_SIZE - 4 {
            for lz in 4..CHUNK_SIZE - 4 {
                rng2 = rng2.wrapping_mul(6364136223846793005).wrapping_add(1);
                let r = (rng2 >> 33) as u32;
                let x = base_x + lx as i32;
                let z = base_z + lz as i32;
                let biome = self.biomes.get_biome(x, z);
                let density = biome.tree_density();
                let threshold = (density * 100.0) as u32;
                if r % 100 >= threshold { continue; }

                let surface_y = (0..WORLD_HEIGHT).rev().find(|&y| {
                    let sy = y / CHUNK_SIZE;
                    let ly = y % CHUNK_SIZE;
                    col.sub_chunks[sy].blocks[lx][lz][ly] != BlockType::Air
                        && col.sub_chunks[sy].blocks[lx][lz][ly] != BlockType::Water
                });

                if let Some(sy_val) = surface_y {
                    let block_sy = sy_val / CHUNK_SIZE;
                    let block_ly = sy_val % CHUNK_SIZE;
                    let surface = col.sub_chunks[block_sy].blocks[lx][lz][block_ly];

                    // Trees only on grass
                    if surface == BlockType::Grass && sy_val + 6 < WORLD_HEIGHT {
                        // Clear vegetation
                        if sy_val + 1 < WORLD_HEIGHT {
                            let above_sy = (sy_val + 1) / CHUNK_SIZE;
                            let above_ly = (sy_val + 1) % CHUNK_SIZE;
                            col.sub_chunks[above_sy].blocks[lx][lz][above_ly] = BlockType::Air;
                        }
                        for dy in 1usize..=4 {
                            if sy_val + dy < WORLD_HEIGHT {
                                let sy = (sy_val + dy) / CHUNK_SIZE;
                                let ly = (sy_val + dy) % CHUNK_SIZE;
                                col.sub_chunks[sy].blocks[lx][lz][ly] = BlockType::Wood;
                            }
                        }
                    }
                }
            }
        }

        col
    }

    /// Ensure all chunks near the player are generated
    pub fn ensure_chunks_around(&mut self, player_x: f64, player_z: f64) {
        let pcx = (player_x as i32) / CHUNK_SIZE as i32;
        let pcz = (player_z as i32) / CHUNK_SIZE as i32;

        for dx in -RENDER_DISTANCE..=RENDER_DISTANCE {
            for dz in -RENDER_DISTANCE..=RENDER_DISTANCE {
                let pos = ChunkPos { cx: pcx + dx, cz: pcz + dz };
                self.get_or_generate(pos);
            }
        }
    }

    pub fn get(&self, x: i32, y: i32, z: i32) -> BlockType {
        if y < 0 || y >= WORLD_HEIGHT as i32 {
            return BlockType::Air;
        }
        let cx = x.div_euclid(CHUNK_SIZE as i32);
        let cz = z.div_euclid(CHUNK_SIZE as i32);
        let pos = ChunkPos { cx, cz };
        if let Some((ix, iz)) = self.chunk_index(pos) {
            if let Some(ref col) = self.chunks[ix][iz] {
                let lx = x.rem_euclid(CHUNK_SIZE as i32) as usize;
                let lz = z.rem_euclid(CHUNK_SIZE as i32) as usize;
                let sy = y as usize / CHUNK_SIZE;
                let ly = y as usize % CHUNK_SIZE;
                if sy < col.sub_chunks.len() {
                    return col.sub_chunks[sy].blocks[lx][lz][ly];
                }
            }
        }
        BlockType::Air
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, block: BlockType) {
        if y < 0 || y >= WORLD_HEIGHT as i32 {
            return;
        }
        let cx = x.div_euclid(CHUNK_SIZE as i32);
        let cz = z.div_euclid(CHUNK_SIZE as i32);
        let pos = ChunkPos { cx, cz };
        if let Some((ix, iz)) = self.chunk_index(pos) {
            if let Some(ref mut col) = self.chunks[ix][iz] {
                let lx = x.rem_euclid(CHUNK_SIZE as i32) as usize;
                let lz = z.rem_euclid(CHUNK_SIZE as i32) as usize;
                let sy = y as usize / CHUNK_SIZE;
                let ly = y as usize % CHUNK_SIZE;
                if sy < col.sub_chunks.len() {
                    col.sub_chunks[sy].blocks[lx][lz][ly] = block;
                }
            }
        }
    }

    pub fn height_at(&self, x: i32, z: i32) -> i32 {
        for y in (0..WORLD_HEIGHT as i32).rev() {
            if self.get(x, y, z).is_solid() {
                return y;
            }
        }
        0
    }
}
