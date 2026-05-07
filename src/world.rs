use crate::block::BlockType;
use noise::{NoiseFn, Simplex};

pub const WORLD_WIDTH: usize = 256;
pub const WORLD_DEPTH: usize = 256;
pub const WORLD_HEIGHT: usize = 64;
pub const SEA_LEVEL: i32 = 20;

pub struct World {
    blocks: Vec<Vec<Vec<BlockType>>>,
}

impl World {
    pub fn new(seed: u32) -> Self {
        let simplex = Simplex::new(seed);
        let mut blocks = vec![
            vec![vec![BlockType::Air; WORLD_HEIGHT]; WORLD_DEPTH];
            WORLD_WIDTH
        ];

        for x in 0..WORLD_WIDTH {
            for z in 0..WORLD_DEPTH {
                // Multi-octave terrain height
                let nx = x as f64 / 80.0;
                let nz = z as f64 / 80.0;
                let h1 = simplex.get([nx, nz]) * 15.0;
                let h2 = simplex.get([nx * 2.0, nz * 2.0]) * 7.0;
                let h3 = simplex.get([nx * 4.0, nz * 4.0]) * 3.0;
                let height = (SEA_LEVEL as f64 + h1 + h2 + h3) as i32;
                let height = height.clamp(1, WORLD_HEIGHT as i32 - 1) as usize;

                for y in 0..WORLD_HEIGHT {
                    blocks[x][z][y] = if y == 0 {
                        BlockType::Stone // bedrock
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

        // Scatter some trees
        let mut rng_state = seed as u64;
        for x in 4..WORLD_WIDTH - 4 {
            for z in 4..WORLD_DEPTH - 4 {
                rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
                let r = (rng_state >> 33) as u32;
                if r % 50 != 0 {
                    continue;
                }

                // Find surface
                let surface_y = (0..WORLD_HEIGHT)
                    .rev()
                    .find(|&y| blocks[x][z][y] != BlockType::Air && blocks[x][z][y] != BlockType::Water);

                if let Some(sy) = surface_y {
                    if blocks[x][z][sy] == BlockType::Grass && sy + 6 < WORLD_HEIGHT {
                        // Trunk
                        for dy in 1usize..=4 {
                            blocks[x][z][sy + dy] = BlockType::Wood;
                        }
                        // Leaves
                        for dx in -2i32..=2 {
                            for dz in -2i32..=2 {
                                let lx = (x as i32 + dx) as usize;
                                let lz = (z as i32 + dz) as usize;
                                for dy in 3usize..=5 {
                                    if (dx.abs() + dz.abs()) <= 3 - (dy as i32 - 3) {
                                        if blocks[lx][lz][sy + dy] == BlockType::Air {
                                            blocks[lx][lz][sy + dy] = BlockType::Leaves;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Self { blocks }
    }

    pub fn get(&self, x: i32, y: i32, z: i32) -> BlockType {
        if x < 0
            || y < 0
            || z < 0
            || x >= WORLD_WIDTH as i32
            || y >= WORLD_HEIGHT as i32
            || z >= WORLD_DEPTH as i32
        {
            return BlockType::Air;
        }
        self.blocks[x as usize][z as usize][y as usize]
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, block: BlockType) {
        if x >= 0
            && y >= 0
            && z >= 0
            && x < WORLD_WIDTH as i32
            && y < WORLD_HEIGHT as i32
            && z < WORLD_DEPTH as i32
        {
            self.blocks[x as usize][z as usize][y as usize] = block;
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
