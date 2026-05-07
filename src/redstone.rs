use crate::block::BlockType;
use crate::world::World;
use std::collections::VecDeque;

const MAX_SIGNAL: u8 = 15;

/// Redstone signal propagation system
pub struct RedstoneSystem;

impl RedstoneSystem {
    /// Propagate redstone signals from all power sources
    /// Returns a map of (x,y,z) -> signal_strength for visual rendering
    pub fn propagate(world: &World, player_x: f64, player_z: f64) -> Vec<((i32, i32, i32), u8)> {
        let mut signals: Vec<((i32, i32, i32), u8)> = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue: VecDeque<((i32, i32, i32), u8)> = VecDeque::new();

        // Scan area around player for power sources
        let cx = player_x as i32;
        let cz = player_z as i32;
        let scan_range = 24i32;

        for x in (cx - scan_range)..=(cx + scan_range) {
            for z in (cz - scan_range)..=(cz + scan_range) {
                for y in 0..64 {
                    let block = world.get(x, y, z);
                    match block {
                        BlockType::Lever => {
                            // Levers always output max signal
                            queue.push_back(((x, y, z), MAX_SIGNAL));
                            visited.insert((x, y, z));
                            signals.push(((x, y, z), MAX_SIGNAL));
                        }
                        BlockType::RedstoneTorch => {
                            // Torches output max signal (unless placed on powered block)
                            queue.push_back(((x, y, z), MAX_SIGNAL));
                            visited.insert((x, y, z));
                            signals.push(((x, y, z), MAX_SIGNAL));
                        }
                        _ => {}
                    }
                }
            }
        }

        // BFS signal propagation
        let neighbors = [
            (1, 0, 0), (-1, 0, 0),
            (0, 1, 0), (0, -1, 0),
            (0, 0, 1), (0, 0, -1),
        ];

        while let Some(((x, y, z), strength)) = queue.pop_front() {
            if strength == 0 {
                continue;
            }

            for &(dx, dy, dz) in &neighbors {
                let nx = x + dx;
                let ny = y + dy;
                let nz = z + dz;
                let key = (nx, ny, nz);

                if visited.contains(&key) {
                    continue;
                }

                let block = world.get(nx, ny, nz);
                let new_strength = match block {
                    BlockType::RedstoneDust => strength - 1,
                    BlockType::RedstoneLamp => strength, // lamp receives but doesn't diminish
                    _ => continue, // only propagate through redstone components
                };

                if new_strength > 0 {
                    visited.insert(key);
                    signals.push((key, new_strength));
                    queue.push_back((key, new_strength));
                }
            }
        }

        signals
    }
}
