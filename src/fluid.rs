use crate::block::BlockType;
use crate::world::World;
use std::collections::VecDeque;

/// Fluid simulation system
/// Water spreads horizontally first (4 blocks), then down, creating realistic flow
pub struct FluidSystem;

/// Fluid state per block: level 0-4 (4=source, 1=trickle, 0=empty)
pub type FluidMap = std::collections::HashMap<(i32, i32, i32), u8>;

impl FluidSystem {
    /// Simulate one tick of fluid dynamics
    /// Returns block updates to apply
    pub fn simulate(world: &World, center_x: f64, center_z: f64) -> Vec<(i32, i32, i32, BlockType)> {
        let mut updates = Vec::new();
        let mut fluid_map: FluidMap = std::collections::HashMap::new();
        let mut queue: VecDeque<(i32, i32, i32, u8)> = VecDeque::new();

        // Scan area around player for fluid sources
        let cx = center_x as i32;
        let cz = center_z as i32;
        let scan = 20i32;

        for x in (cx - scan)..=(cx + scan) {
            for z in (cz - scan)..=(cz + scan) {
                for y in 0..64 {
                    let block = world.get(x, y, z);
                    if block == BlockType::Water {
                        // Check if it's a source (water with solid below or water below)
                        let below = world.get(x, y - 1, z);
                        if below == BlockType::Stone || below == BlockType::Dirt || below == BlockType::Sand {
                            fluid_map.insert((x, y, z), 4);
                            queue.push_back((x, y, z, 4));
                        } else if below == BlockType::Water {
                            fluid_map.insert((x, y, z), 3);
                            queue.push_back((x, y, z, 3));
                        }
                    }
                }
            }
        }

        // BFS fluid propagation
        let neighbors_h = [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)];

        while let Some((x, y, z, level)) = queue.pop_front() {
            if level == 0 { continue; }

            // Try spreading down first (priority)
            let below = world.get(x, y - 1, z);
            if below == BlockType::Air && y > 0 {
                let key = (x, y - 1, z);
                if !fluid_map.contains_key(&key) {
                    fluid_map.insert(key, 4); // falls as full source
                    queue.push_back((x, y - 1, z, 4));
                    updates.push((x, y - 1, z, BlockType::Water));
                }
                continue; // don't spread horizontally if can fall
            }

            // Spread horizontally (with level decay)
            if level > 1 {
                for &(dx, dz) in &neighbors_h {
                    let nx = x + dx;
                    let nz = z + dz;
                    let key = (nx, y, nz);

                    if fluid_map.contains_key(&key) {
                        continue;
                    }

                    let target = world.get(nx, y, nz);
                    if target == BlockType::Air {
                        let new_level = level - 1;
                        fluid_map.insert(key, new_level);
                        queue.push_back((nx, y, nz, new_level));
                        updates.push((nx, y, nz, BlockType::Water));
                    }
                }
            }
        }

        // Remove water that should drain (no source feeding it)
        // Check existing water blocks that aren't in the fluid map
        for x in (cx - scan)..=(cx + scan) {
            for z in (cz - scan)..=(cz + scan) {
                for y in 0..64 {
                    if world.get(x, y, z) == BlockType::Water {
                        let key = (x, y, z);
                        if !fluid_map.contains_key(&key) {
                            // This water has no source - remove it
                            let above = world.get(x, y + 1, z);
                            if above != BlockType::Water {
                                updates.push((x, y, z, BlockType::Air));
                            }
                        }
                    }
                }
            }
        }

        updates
    }
}
