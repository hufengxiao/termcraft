#![allow(dead_code)]
use crate::block::BlockType;

/// Structure generation templates
pub struct Structures;

impl Structures {
    /// Generate a desert temple at position
    pub fn desert_temple(blocks: &mut Vec<Vec<Vec<BlockType>>>, x: usize, y: usize, z: usize) {
        let w = 9usize;
        let h = 7usize;
        let d = 9usize;

        // Sandstone base
        for dx in 0..w {
            for dz in 0..d {
                Self::set(blocks, x + dx, y, z + dz, BlockType::Sand);
                for dy in 1..=h {
                    if dx == 0 || dx == w-1 || dz == 0 || dz == d-1 {
                        Self::set(blocks, x + dx, y + dy, z + dz, BlockType::Sand);
                    }
                }
            }
        }

        // Interior air
        for dx in 1..w-1 {
            for dz in 1..d-1 {
                for dy in 1..h {
                    Self::set(blocks, x + dx, y + dy, z + dz, BlockType::Air);
                }
            }
        }

        // Roof pyramid
        for layer in 0..3usize {
            for dx in layer..w-layer {
                for dz in layer..d-layer {
                    Self::set(blocks, x + dx, y + h + 1 + layer, z + dz, BlockType::Sand);
                }
            }
        }

        // Treasure room below
        for dx in 3..6 {
            for dz in 3..6 {
                for dy in 1..4 {
                    Self::set(blocks, x + dx, y - dy, z + dz, BlockType::Air);
                }
                Self::set(blocks, x + dx, y - 4, z + dz, BlockType::Sand);
            }
        }
    }

    /// Generate a witch hut (swamp structure)
    pub fn witch_hut(blocks: &mut Vec<Vec<Vec<BlockType>>>, x: usize, y: usize, z: usize) {
        let w = 5usize;
        let d = 5usize;

        // Floor
        for dx in 0..w {
            for dz in 0..d {
                Self::set(blocks, x + dx, y, z + dz, BlockType::Wood);
            }
        }

        // Walls
        for dy in 1..=3 {
            for dx in 0..w {
                for dz in 0..d {
                    if dx == 0 || dx == w-1 || dz == 0 || dz == d-1 {
                        Self::set(blocks, x + dx, y + dy, z + dz, BlockType::Wood);
                    }
                }
            }
        }

        // Door
        Self::set(blocks, x + 2, y + 1, z, BlockType::Air);
        Self::set(blocks, x + 2, y + 2, z, BlockType::Air);

        // Roof
        for dx in 0..w {
            for dz in 0..d {
                Self::set(blocks, x + dx, y + 4, z + dz, BlockType::Leaves);
            }
        }

        // Interior
        for dx in 1..w-1 {
            for dz in 1..d-1 {
                for dy in 1..=3 {
                    Self::set(blocks, x + dx, y + dy, z + dz, BlockType::Air);
                }
            }
        }
    }

    /// Generate a mineshaft corridor
    pub fn mineshaft(blocks: &mut Vec<Vec<Vec<BlockType>>>, x: usize, y: usize, z: usize, length: usize) {
        // Wooden corridor
        for i in 0..length {
            // Floor
            Self::set(blocks, x + i, y, z, BlockType::Wood);
            Self::set(blocks, x + i, y, z + 1, BlockType::Wood);

            // Walls
            Self::set(blocks, x + i, y + 1, z - 1, BlockType::Wood);
            Self::set(blocks, x + i, y + 1, z + 2, BlockType::Wood);
            Self::set(blocks, x + i, y + 2, z - 1, BlockType::Wood);
            Self::set(blocks, x + i, y + 2, z + 2, BlockType::Wood);

            // Air inside
            Self::set(blocks, x + i, y + 1, z, BlockType::Air);
            Self::set(blocks, x + i, y + 1, z + 1, BlockType::Air);
            Self::set(blocks, x + i, y + 2, z, BlockType::Air);
            Self::set(blocks, x + i, y + 2, z + 1, BlockType::Air);
            Self::set(blocks, x + i, y + 3, z, BlockType::Air);
            Self::set(blocks, x + i, y + 3, z + 1, BlockType::Air);

            // Ceiling
            Self::set(blocks, x + i, y + 4, z, BlockType::Wood);
            Self::set(blocks, x + i, y + 4, z + 1, BlockType::Wood);
        }
    }

    fn set(blocks: &mut Vec<Vec<Vec<BlockType>>>, x: usize, y: usize, z: usize, block: BlockType) {
        if x < blocks.len() && z < blocks[0].len() && y < blocks[0][0].len() {
            blocks[x][z][y] = block;
        }
    }
}
