use crate::block::BlockType;
use serde::{Deserialize, Serialize};

pub const CHUNK_SIZE: usize = 16;
pub const WORLD_WIDTH: usize = 256;
pub const WORLD_DEPTH: usize = 256;
pub const WORLD_HEIGHT: usize = 64;
pub const SEA_LEVEL: i32 = 20;

/// A 16x16 column of blocks
#[derive(Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub blocks: Vec<Vec<Vec<BlockType>>>, // [x_local][z_local][y]
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            blocks: vec![
                vec![vec![BlockType::Air; WORLD_HEIGHT]; CHUNK_SIZE];
                CHUNK_SIZE
            ],
        }
    }
}

pub struct World {
    chunks: Vec<Vec<Chunk>>, // [chunk_x][chunk_z]
}

impl World {
    pub fn from_blocks(blocks: Vec<Vec<Vec<BlockType>>>) -> Self {
        // Convert flat blocks to chunks
        let chunks_x = WORLD_WIDTH / CHUNK_SIZE;
        let chunks_z = WORLD_DEPTH / CHUNK_SIZE;
        let mut chunks = vec![vec![Chunk::new(); chunks_z]; chunks_x];

        for cx in 0..chunks_x {
            for cz in 0..chunks_z {
                for lx in 0..CHUNK_SIZE {
                    for lz in 0..CHUNK_SIZE {
                        let wx = cx * CHUNK_SIZE + lx;
                        let wz = cz * CHUNK_SIZE + lz;
                        if wx < WORLD_WIDTH && wz < WORLD_DEPTH {
                            for y in 0..WORLD_HEIGHT {
                                chunks[cx][cz].blocks[lx][lz][y] = blocks[wx][wz][y];
                            }
                        }
                    }
                }
            }
        }

        Self { chunks }
    }

    pub fn blocks_ref(&self) -> Vec<Vec<Vec<BlockType>>> {
        // Convert chunks back to flat blocks for save
        let mut blocks = vec![
            vec![vec![BlockType::Air; WORLD_HEIGHT]; WORLD_DEPTH];
            WORLD_WIDTH
        ];
        let chunks_x = WORLD_WIDTH / CHUNK_SIZE;
        let chunks_z = WORLD_DEPTH / CHUNK_SIZE;
        for cx in 0..chunks_x {
            for cz in 0..chunks_z {
                for lx in 0..CHUNK_SIZE {
                    for lz in 0..CHUNK_SIZE {
                        let wx = cx * CHUNK_SIZE + lx;
                        let wz = cz * CHUNK_SIZE + lz;
                        if wx < WORLD_WIDTH && wz < WORLD_DEPTH {
                            for y in 0..WORLD_HEIGHT {
                                blocks[wx][wz][y] = self.chunks[cx][cz].blocks[lx][lz][y];
                            }
                        }
                    }
                }
            }
        }
        blocks
    }

    pub fn new(seed: u32) -> Self {
        use noise::{NoiseFn, Simplex, Perlin};

        let simplex = Simplex::new(seed);
        let perlin = Perlin::new(seed + 100);
        let chunks_x = WORLD_WIDTH / CHUNK_SIZE;
        let chunks_z = WORLD_DEPTH / CHUNK_SIZE;
        let mut chunks = vec![vec![Chunk::new(); chunks_z]; chunks_x];

        // Phase 1: Generate terrain per chunk
        for cx in 0..chunks_x {
            for cz in 0..chunks_z {
                for lx in 0..CHUNK_SIZE {
                    for lz in 0..CHUNK_SIZE {
                        let x = cx * CHUNK_SIZE + lx;
                        let z = cz * CHUNK_SIZE + lz;

                        let nx = x as f64 / 80.0;
                        let nz = z as f64 / 80.0;
                        let h1 = simplex.get([nx, nz]) * 15.0;
                        let h2 = simplex.get([nx * 2.0, nz * 2.0]) * 7.0;
                        let h3 = simplex.get([nx * 4.0, nz * 4.0]) * 3.0;
                        let height = (SEA_LEVEL as f64 + h1 + h2 + h3) as i32;
                        let height = height.clamp(1, WORLD_HEIGHT as i32 - 1) as usize;

                        for y in 0..WORLD_HEIGHT {
                            chunks[cx][cz].blocks[lx][lz][y] = if y == 0 {
                                BlockType::Stone
                            } else if height > 4 && y < height - 4 {
                                BlockType::Stone
                            } else if height > 1 && y < height - 1 {
                                if height < SEA_LEVEL as usize + 2 {
                                    BlockType::Sand
                                } else {
                                    BlockType::Dirt
                                }
                            } else if height > 0 && y == height - 1 {
                                if height < SEA_LEVEL as usize + 2 {
                                    BlockType::Sand
                                } else {
                                    BlockType::Grass
                                }
                            } else if y < SEA_LEVEL as usize {
                                BlockType::Water
                            } else {
                                BlockType::Air
                            };
                        }
                    }
                }
            }
        }

        // Phase 2: Caves
        for cx in 0..chunks_x {
            for cz in 0..chunks_z {
                for lx in 0..CHUNK_SIZE {
                    for lz in 0..CHUNK_SIZE {
                        let x = cx * CHUNK_SIZE + lx;
                        let z = cz * CHUNK_SIZE + lz;
                        for y in 2..WORLD_HEIGHT - 5 {
                            let nx = x as f64 / 20.0;
                            let ny = y as f64 / 12.0;
                            let nz = z as f64 / 20.0;
                            let cave_noise = perlin.get([nx, ny, nz]);
                            let cave2 = perlin.get([nx * 2.5, ny * 2.0, nz * 2.5]) * 0.5;
                            let combined = cave_noise + cave2;
                            let depth_factor = y as f64 / WORLD_HEIGHT as f64;
                            let threshold = 0.45 - depth_factor * 0.15;

                            if combined > threshold
                                && chunks[cx][cz].blocks[lx][lz][y] != BlockType::Air
                                && chunks[cx][cz].blocks[lx][lz][y] != BlockType::Water
                            {
                                chunks[cx][cz].blocks[lx][lz][y] = BlockType::Air;
                            }
                        }
                    }
                }
            }
        }

        // Phase 3: Vegetation
        let mut rng_state = seed as u64;
        for cx in 0..chunks_x {
            for cz in 0..chunks_z {
                for lx in 4..CHUNK_SIZE - 4 {
                    for lz in 4..CHUNK_SIZE - 4 {
                        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
                        let r = (rng_state >> 33) as u32;

                        let surface_y = (0..WORLD_HEIGHT)
                            .rev()
                            .find(|&y| chunks[cx][cz].blocks[lx][lz][y].is_solid());

                        if let Some(sy) = surface_y {
                            if chunks[cx][cz].blocks[lx][lz][sy] == BlockType::Grass && sy + 1 < WORLD_HEIGHT {
                                match r % 100 {
                                    0..=3 => chunks[cx][cz].blocks[lx][lz][sy + 1] = BlockType::Flower,
                                    4..=12 => chunks[cx][cz].blocks[lx][lz][sy + 1] = BlockType::TallGrass,
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        // Phase 4: Trees (cross-chunk aware)
        let mut rng_state = seed as u64 + 999;
        for x in 4..WORLD_WIDTH - 4 {
            for z in 4..WORLD_DEPTH - 4 {
                rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
                let r = (rng_state >> 33) as u32;
                if r % 50 != 0 { continue; }

                let surface_y = (0..WORLD_HEIGHT).rev().find(|&y| {
                    let cx = x / CHUNK_SIZE;
                    let cz = z / CHUNK_SIZE;
                    let lx = x % CHUNK_SIZE;
                    let lz = z % CHUNK_SIZE;
                    if cx < chunks_x && cz < chunks_z {
                        chunks[cx][cz].blocks[lx][lz][y] != BlockType::Air
                            && chunks[cx][cz].blocks[lx][lz][y] != BlockType::Water
                    } else {
                        false
                    }
                });

                if let Some(sy) = surface_y {
                    let cx = x / CHUNK_SIZE;
                    let cz = z / CHUNK_SIZE;
                    let lx = x % CHUNK_SIZE;
                    let lz = z % CHUNK_SIZE;

                    if chunks[cx][cz].blocks[lx][lz][sy] == BlockType::Grass && sy + 6 < WORLD_HEIGHT {
                        if sy + 1 < WORLD_HEIGHT {
                            chunks[cx][cz].blocks[lx][lz][sy + 1] = BlockType::Air;
                        }
                        for dy in 1usize..=4 {
                            if sy + dy < WORLD_HEIGHT {
                                chunks[cx][cz].blocks[lx][lz][sy + dy] = BlockType::Wood;
                            }
                        }
                        for dx in -2i32..=2 {
                            for dz in -2i32..=2 {
                                let wx = x as i32 + dx;
                                let wz = z as i32 + dz;
                                if wx >= 0 && wz >= 0 && wx < WORLD_WIDTH as i32 && wz < WORLD_DEPTH as i32 {
                                    let tcx = wx as usize / CHUNK_SIZE;
                                    let tcz = wz as usize / CHUNK_SIZE;
                                    let tlx = wx as usize % CHUNK_SIZE;
                                    let tlz = wz as usize % CHUNK_SIZE;
                                    for dy in 3usize..=5 {
                                        if (dx.abs() + dz.abs()) <= 3 - (dy as i32 - 3) {
                                            if sy + dy < WORLD_HEIGHT
                                                && chunks[tcx][tcz].blocks[tlx][tlz][sy + dy] == BlockType::Air
                                            {
                                                chunks[tcx][tcz].blocks[tlx][tlz][sy + dy] = BlockType::Leaves;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Phase 5: Villages
        let village_positions = [
            (80, 80), (80, 160), (160, 80), (160, 160),
            (120, 60), (120, 200), (60, 120), (200, 120),
        ];

        for &(vx, vz) in &village_positions {
            let cx = vx / CHUNK_SIZE;
            let cz = vz / CHUNK_SIZE;
            let lx = vx % CHUNK_SIZE;
            let lz = vz % CHUNK_SIZE;

            let sy_opt = (0..WORLD_HEIGHT).rev().find(|&y| {
                chunks[cx][cz].blocks[lx][lz][y].is_solid()
            });

            if let Some(sy) = sy_opt {
                if chunks[cx][cz].blocks[lx][lz][sy] == BlockType::Grass
                    && sy > SEA_LEVEL as usize + 2
                    && sy + 7 < WORLD_HEIGHT
                {
                    Self::build_house_in_chunks(&mut chunks, vx, sy + 1, vz);
                }
            }
        }

        Self { chunks }
    }

    fn build_house_in_chunks(chunks: &mut Vec<Vec<Chunk>>, x: usize, y: usize, z: usize) {
        let w = 5usize;
        let h = 4usize;
        let d = 5usize;
        let chunks_x = WORLD_WIDTH / CHUNK_SIZE;
        let chunks_z = WORLD_DEPTH / CHUNK_SIZE;

        let set_block = |chunks: &mut Vec<Vec<Chunk>>, bx: usize, by: usize, bz: usize, block: BlockType| {
            if bx < WORLD_WIDTH && bz < WORLD_DEPTH && by < WORLD_HEIGHT {
                let cx = bx / CHUNK_SIZE;
                let cz = bz / CHUNK_SIZE;
                let lx = bx % CHUNK_SIZE;
                let lz = bz % CHUNK_SIZE;
                if cx < chunks_x && cz < chunks_z {
                    chunks[cx][cz].blocks[lx][lz][by] = block;
                }
            }
        };

        // Floor
        for dx in 0..w {
            for dz in 0..d {
                set_block(chunks, x + dx, y, z + dz, BlockType::Wood);
            }
        }

        // Walls
        for dy in 1..=h {
            for dx in 0..w {
                for dz in 0..d {
                    if dx == 0 || dx == w - 1 || dz == 0 || dz == d - 1 {
                        if dy <= 2 && dx == w / 2 && dz == 0 {
                            set_block(chunks, x + dx, y + dy, z + dz, BlockType::Air);
                        } else {
                            set_block(chunks, x + dx, y + dy, z + dz, BlockType::Wood);
                        }
                    }
                }
            }
        }

        // Roof
        for dx in 0..w {
            for dz in 0..d {
                set_block(chunks, x + dx, y + h + 1, z + dz, BlockType::Leaves);
            }
        }

        // Interior air
        for dy in 1..=h {
            for dx in 1..w - 1 {
                for dz in 1..d - 1 {
                    set_block(chunks, x + dx, y + dy, z + dz, BlockType::Air);
                }
            }
        }
    }

    pub fn get(&self, x: i32, y: i32, z: i32) -> BlockType {
        if x < 0 || y < 0 || z < 0
            || x >= WORLD_WIDTH as i32
            || y >= WORLD_HEIGHT as i32
            || z >= WORLD_DEPTH as i32
        {
            return BlockType::Air;
        }
        let cx = x as usize / CHUNK_SIZE;
        let cz = z as usize / CHUNK_SIZE;
        let lx = x as usize % CHUNK_SIZE;
        let lz = z as usize % CHUNK_SIZE;
        self.chunks[cx][cz].blocks[lx][lz][y as usize]
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, block: BlockType) {
        if x >= 0 && y >= 0 && z >= 0
            && x < WORLD_WIDTH as i32
            && y < WORLD_HEIGHT as i32
            && z < WORLD_DEPTH as i32
        {
            let cx = x as usize / CHUNK_SIZE;
            let cz = z as usize / CHUNK_SIZE;
            let lx = x as usize % CHUNK_SIZE;
            let lz = z as usize % CHUNK_SIZE;
            self.chunks[cx][cz].blocks[lx][lz][y as usize] = block;
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
