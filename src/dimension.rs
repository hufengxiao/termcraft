#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use noise::{NoiseFn, Perlin};
use crate::block::BlockType;
use crate::world::{CHUNK_SIZE, WORLD_HEIGHT, ChunkColumn, ChunkPos};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dimension {
    Overworld,
    Nether,
}

impl Dimension {
    pub fn name(&self) -> &str {
        match self {
            Self::Overworld => "Overworld",
            Self::Nether => "Nether",
        }
    }
}

/// Generate nether terrain for a chunk column
pub fn generate_nether_column(pos: ChunkPos, seed: u32) -> ChunkColumn {
    let perlin = Perlin::new(seed + 500);
    let mut col = ChunkColumn::new();
    let base_x = pos.cx * CHUNK_SIZE as i32;
    let base_z = pos.cz * CHUNK_SIZE as i32;

    for lx in 0..CHUNK_SIZE {
        for lz in 0..CHUNK_SIZE {
            let x = base_x + lx as i32;
            let z = base_z + lz as i32;

            // Nether has a ceiling and floor
            for y in 0..WORLD_HEIGHT {
                let sy = y / CHUNK_SIZE;
                let ly = y % CHUNK_SIZE;

                let nx = x as f64 / 30.0;
                let ny = y as f64 / 20.0;
                let nz = z as f64 / 30.0;

                // Nether terrain: ceiling at y=50+, floor at y=5, caves in between
                let cave = perlin.get([nx, ny, nz])
                    + perlin.get([nx * 2.0, ny * 1.5, nz * 2.0]) * 0.5;

                col.sub_chunks[sy].blocks[lx][lz][ly] = if y <= 3 {
                    BlockType::Lava // lava lakes at bottom
                } else if y <= 5 {
                    BlockType::Netherrack
                } else if y >= 50 {
                    BlockType::Netherrack // ceiling
                } else if cave > 0.3 {
                    BlockType::Air // caves
                } else {
                    BlockType::Netherrack
                };
            }
        }
    }

    col
}

/// Check if a portal frame exists at the given position
/// A valid portal is a 4x5 frame of obsidian with air inside
pub fn check_portal(world: &crate::world::World, x: i32, y: i32, z: i32) -> bool {
    // Check for 4-wide, 5-tall obsidian frame
    // Bottom row
    for dx in 0..4 {
        if world.get(x + dx, y, z) != BlockType::Obsidian {
            return false;
        }
    }
    // Top row
    for dx in 0..4 {
        if world.get(x + dx, y + 4, z) != BlockType::Obsidian {
            return false;
        }
    }
    // Sides
    for dy in 1..4 {
        if world.get(x, y + dy, z) != BlockType::Obsidian {
            return false;
        }
        if world.get(x + 3, y + dy, z) != BlockType::Obsidian {
            return false;
        }
    }
    // Interior must be air
    for dx in 1..3 {
        for dy in 1..4 {
            if world.get(x + dx, y + dy, z) != BlockType::Air {
                return false;
            }
        }
    }
    true
}

/// Activate a portal: fill interior with portal blocks
pub fn activate_portal(world: &mut crate::world::World, x: i32, y: i32, z: i32) {
    for dx in 1..3 {
        for dy in 1..4 {
            world.set(x + dx, y + dy, z, BlockType::Portal);
        }
    }
}
