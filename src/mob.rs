#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;

use crate::world::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MobType {
    Zombie,
    Slime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mob {
    pub mob_type: MobType,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vz: f64,
    pub health: i32,
    pub target_x: Option<f64>,
    pub target_z: Option<f64>,
    pub path: Vec<(i32, i32, i32)>,
    pub path_index: usize,
    pub ai_timer: u32,
}

impl Mob {
    pub fn new(mob_type: MobType, x: f64, y: f64, z: f64) -> Self {
        Self {
            mob_type,
            x, y, z,
            vx: 0.0, vz: 0.0,
            health: match mob_type {
                MobType::Zombie => 20,
                MobType::Slime => 8,
            },
            target_x: None,
            target_z: None,
            path: Vec::new(),
            path_index: 0,
            ai_timer: 0,
        }
    }

    pub fn glyph(&self) -> char {
        match self.mob_type {
            MobType::Zombie => 'Z',
            MobType::Slime => 'S',
        }
    }

    pub fn color(&self) -> crossterm::style::Color {
        match self.mob_type {
            MobType::Zombie => crossterm::style::Color::Green,
            MobType::Slime => crossterm::style::Color::DarkGreen,
        }
    }

    /// Update mob AI each tick
    pub fn update(&mut self, world: &World, player_x: f64, player_y: f64, player_z: f64) {
        self.ai_timer += 1;

        // Every 20 ticks (~1 second), recalculate path toward player
        if self.ai_timer >= 20 {
            self.ai_timer = 0;

            let dx = player_x - self.x;
            let dz = player_z - self.z;
            let dist_sq = dx * dx + dz * dz;

            // Only chase if within 16 blocks
            if dist_sq < 256.0 {
                let start = (self.x as i32, self.y as i32, self.z as i32);
                let goal = (player_x as i32, player_y as i32, player_z as i32);
                self.path = astar(world, start, goal);
                self.path_index = 0;
            } else {
                // Wander randomly
                self.path.clear();
                self.path_index = 0;
            }
        }

        // Follow path
        if !self.path.is_empty() && self.path_index < self.path.len() {
            let (tx, _ty, tz) = self.path[self.path_index];
            let dx = tx as f64 + 0.5 - self.x;
            let dz = tz as f64 + 0.5 - self.z;
            let dist = (dx * dx + dz * dz).sqrt();

            if dist < 0.3 {
                self.path_index += 1;
            } else {
                let speed = match self.mob_type {
                    MobType::Zombie => 0.04,
                    MobType::Slime => 0.06,
                };
                self.vx = dx / dist * speed;
                self.vz = dz / dist * speed;
            }
        } else {
            // Friction when no path
            self.vx *= 0.8;
            self.vz *= 0.8;
        }

        // Apply velocity
        self.x += self.vx;
        self.z += self.vz;

        // Gravity
        let ground = world.height_at(self.x as i32, self.z as i32) + 1;
        if self.y > ground as f64 {
            self.y -= 0.05;
        } else {
            self.y = ground as f64;
        }

        // Clamp to world
        self.x = self.x.clamp(2.0, 253.0);
        self.z = self.z.clamp(2.0, 253.0);
    }
}

/// A* pathfinding on the voxel grid
fn astar(world: &World, start: (i32, i32, i32), goal: (i32, i32, i32)) -> Vec<(i32, i32, i32)> {
    let max_nodes = 500; // limit search for performance

    let heuristic = |pos: (i32, i32, i32)| -> i32 {
        (pos.0 - goal.0).abs() + (pos.1 - goal.1).abs() + (pos.2 - goal.2).abs()
    };

    let mut open = BinaryHeap::new();
    let mut came_from: HashMap<(i32, i32, i32), (i32, i32, i32)> = HashMap::new();
    let mut g_score: HashMap<(i32, i32, i32), i32> = HashMap::new();

    g_score.insert(start, 0);
    open.push(Reverse((heuristic(start), start)));

    let neighbors = [
        (1, 0, 0), (-1, 0, 0),
        (0, 0, 1), (0, 0, -1),
        (0, 1, 0), (0, -1, 0),
        // Diagonals
        (1, 0, 1), (-1, 0, 1), (1, 0, -1), (-1, 0, -1),
    ];

    let mut nodes_explored = 0;

    while let Some(Reverse((_, current))) = open.pop() {
        nodes_explored += 1;
        if nodes_explored > max_nodes {
            break;
        }

        if current == goal {
            // Reconstruct path
            let mut path = Vec::new();
            let mut pos = current;
            while pos != start {
                path.push(pos);
                pos = came_from[&pos];
            }
            path.reverse();
            return path;
        }

        let current_g = *g_score.get(&current).unwrap_or(&i32::MAX);

        for &(dx, dy, dz) in &neighbors {
            let next = (current.0 + dx, current.1 + dy, current.2 + dz);

            // Check bounds
            if next.0 < 0 || next.2 < 0 || next.0 >= 256 || next.2 >= 256 || next.1 < 0 || next.1 >= 64 {
                continue;
            }

            // Check walkability: need solid ground below and air at feet and head
            let block_at = world.get(next.0, next.1, next.2);
            let block_below = world.get(next.0, next.1 - 1, next.2);
            let block_head = world.get(next.0, next.1 + 1, next.2);

            if block_at.is_solid() || block_head.is_solid() || !block_below.is_solid() {
                continue;
            }

            let move_cost = if dy != 0 { 2 } else { 1 }; // prefer flat paths
            let tentative_g = current_g + move_cost;

            if tentative_g < *g_score.get(&next).unwrap_or(&i32::MAX) {
                came_from.insert(next, current);
                g_score.insert(next, tentative_g);
                let f = tentative_g + heuristic(next);
                open.push(Reverse((f, next)));
            }
        }
    }

    Vec::new() // no path found
}
