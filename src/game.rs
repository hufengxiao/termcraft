use std::io::{stdout, Write};
use std::time::Instant;
use crossterm::{
    cursor,
    execute,
    style::{Color, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};

use crate::camera::{Camera, VIEW_HEIGHT, VIEW_WIDTH};
use crate::cpu::RedstoneCPU;
use crate::dimension::{self, Dimension};
use crate::fluid::FluidSystem;
use crate::input::{Action, Input};
use crate::item::Inventory;
use crate::mob::{Mob, MobType};
use crate::player::Player;
use crate::redstone::RedstoneSystem;
use crate::save;
use crate::script::ScriptEngine;
use crate::sound::SoundEngine;
use crate::world::World;

const GRAVITY: f64 = -0.02;
const JUMP_VEL: f64 = 0.25;
const MOVE_SPEED: f64 = 0.015; // acceleration per tick
const FRICTION: f64 = 0.85;    // velocity multiplier per tick
const MAX_SPEED: f64 = 0.25;
const TICK_MS: u64 = 50; // 20 TPS
const DAY_LENGTH: u64 = 2400; // ticks per full day cycle (2 minutes)

pub struct Game {
    world: World,
    player: Player,
    camera: Camera,
    running: bool,
    tick: u64,
    sound: Option<SoundEngine>,
    prev_frame: Vec<Vec<(char, Color)>>,
    inventory: Inventory,
    mobs: Vec<Mob>,
    dimension: Dimension,
    overworld_pos: Option<(f64, f64, f64)>,
    script_engine: ScriptEngine,
    frame_time_us: u64,
    cpu: RedstoneCPU,
}

/// Time of day info passed to renderer
pub struct DayTime {
    /// 0.0 = midnight, 0.25 = sunrise, 0.5 = noon, 0.75 = sunset
    pub phase: f64,
    /// 0.0 = night, 1.0 = full day brightness
    pub brightness: f64,
}

impl DayTime {
    pub fn from_tick(tick: u64, day_length: u64) -> Self {
        let phase = (tick % day_length) as f64 / day_length as f64;
        // brightness: sinusoidal curve peaking at noon (phase=0.5)
        let brightness = ((phase - 0.25) * std::f64::consts::PI * 2.0).sin();
        let brightness = brightness.clamp(0.0, 1.0);
        Self { phase, brightness }
    }

    pub fn sky_color(&self) -> (u8, u8, u8) {
        if self.phase < 0.2 || self.phase > 0.85 {
            // Night: dark blue
            (5, 5, 25)
        } else if self.phase < 0.3 {
            // Sunrise: orange/pink
            let t = (self.phase - 0.2) / 0.1;
            lerp_rgb((5, 5, 25), (255, 140, 50), t)
        } else if self.phase < 0.4 {
            // Morning: light blue
            let t = (self.phase - 0.3) / 0.1;
            lerp_rgb((255, 140, 50), (100, 180, 255), t)
        } else if self.phase < 0.65 {
            // Day: bright blue
            (100, 180, 255)
        } else if self.phase < 0.75 {
            // Afternoon to sunset
            let t = (self.phase - 0.65) / 0.1;
            lerp_rgb((100, 180, 255), (255, 100, 50), t)
        } else {
            // Dusk to night
            let t = (self.phase - 0.75) / 0.1;
            lerp_rgb((255, 100, 50), (5, 5, 25), t)
        }
    }
}

fn lerp_rgb(a: (u8, u8, u8), b: (u8, u8, u8), t: f64) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    (
        (a.0 as f64 + (b.0 as f64 - a.0 as f64) * t) as u8,
        (a.1 as f64 + (b.1 as f64 - a.1 as f64) * t) as u8,
        (a.2 as f64 + (b.2 as f64 - a.2 as f64) * t) as u8,
    )
}

impl Game {
    pub fn new() -> Self {
        // Try to load saved world
        if let Some(saved) = save::load_world() {
            let world = World::from_blocks(saved.blocks);
            let mut player = Player::new(saved.player_x, saved.player_y, saved.player_z);
            player.yaw = saved.player_yaw;
            player.pitch = saved.player_pitch;
            return Self {
                world,
                player,
                camera: Camera::new(),
                running: true,
                tick: saved.tick,
                sound: SoundEngine::new(),
                prev_frame: vec![vec![(' ', Color::Black); VIEW_WIDTH]; VIEW_HEIGHT],
                inventory: Inventory::new(),
                mobs: Vec::new(),
                dimension: Dimension::Overworld,
                overworld_pos: None,
                script_engine: ScriptEngine::new(),
                frame_time_us: 0,
                cpu: RedstoneCPU::new(),
            };
        }

        let seed = 42;
        let world = World::new(seed);
        let spawn_x = 128.0;
        let spawn_z = 128.0;
        let spawn_y = (world.height_at(spawn_x as i32, spawn_z as i32) + 1) as f64;

        Self {
            world,
            player: Player::new(spawn_x, spawn_y, spawn_z),
            camera: Camera::new(),
            running: true,
            tick: 600,
            sound: SoundEngine::new(),
            prev_frame: vec![vec![(' ', Color::Black); VIEW_WIDTH]; VIEW_HEIGHT],
            inventory: Inventory::new(),
            mobs: Vec::new(),
            dimension: Dimension::Overworld,
            overworld_pos: None,
            script_engine: ScriptEngine::new(),
            frame_time_us: 0,
            cpu: RedstoneCPU::new(),
        }
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        let mut stdout = stdout();
        terminal::enable_raw_mode()?;
        execute!(
            stdout,
            terminal::EnterAlternateScreen,
            terminal::Clear(ClearType::All),
            cursor::Hide
        )?;

        while self.running {
            let action = Input::poll();
            self.handle_action(action);
            self.physics();
            self.tick += 1;
            self.render(&mut stdout)?;
            std::thread::sleep(std::time::Duration::from_millis(TICK_MS));
        }

        execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.running = false,
            Action::Move { dx, dz } => {
                let (fy, fx) = self.player.forward_dir();
                let ax = dx * fy + dz * fx;
                let az = -dx * fx + dz * fy;
                let len = (ax * ax + az * az).sqrt();
                if len > 0.0 {
                    self.player.vx += ax / len * MOVE_SPEED;
                    self.player.vz += az / len * MOVE_SPEED;
                }
            }
            Action::Jump => {
                if self.player.on_ground {
                    self.player.vy = JUMP_VEL;
                    self.player.on_ground = false;
                }
            }
            Action::Look { dyaw, dpitch } => {
                self.player.yaw += dyaw;
                self.player.pitch = (self.player.pitch + dpitch).clamp(-1.2, 1.2);
            }
            Action::Place => {
                let (fx, fz) = self.player.forward_dir();
                let px = self.player.x + fx * 2.0;
                let pz = self.player.z + fz * 2.0;
                let py = self.player.y + 1.0;
                // Use item from inventory
                if let Some(item) = self.inventory.use_selected() {
                    if let crate::item::ItemType::Block(block) = item.item_type {
                        self.world.set(px as i32, py as i32, pz as i32, block);
                        if let Some(ref s) = self.sound { s.play_place(); }

                        // Check for portal formation when placing obsidian
                        if block == crate::block::BlockType::Obsidian {
                            for dx in -3i32..=0 {
                                for dy in -4i32..=0 {
                                    if dimension::check_portal(&self.world, px as i32 + dx, py as i32 + dy, pz as i32) {
                                        dimension::activate_portal(&mut self.world, px as i32 + dx, py as i32 + dy, pz as i32);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Action::Break => {
                let (fx, fz) = self.player.forward_dir();
                let px = self.player.x + fx * 2.0;
                let pz = self.player.z + fz * 2.0;
                let py = self.player.y + 1.0;
                let broken_block = self.world.get(px as i32, py as i32, pz as i32);
                if broken_block.is_solid() {
                    self.world.set(px as i32, py as i32, pz as i32, crate::block::BlockType::Air);
                    self.inventory.add_item(crate::item::Item::from_block(broken_block));
                    if let Some(ref s) = self.sound { s.play_break(); }
                }
            }
            Action::SelectBlock(i) => {
                self.inventory.selected = i;
            }
            Action::Save => {
                self.save_game();
            }
            Action::RunScript => {
                // Execute scripts/init.lua if it exists
                if std::path::Path::new("scripts/init.lua").exists() {
                    match self.script_engine.exec_file("scripts/init.lua") {
                        Ok(_) => { /* silent success */ }
                        Err(e) => { eprintln!("[Script] {e}"); }
                    }
                } else {
                    eprintln!("[Script] No scripts/init.lua found");
                }
            }
            Action::None => {}
        }
    }

    fn physics(&mut self) {
        // Gravity
        self.player.vy += GRAVITY;

        // Friction on horizontal movement
        self.player.vx *= FRICTION;
        self.player.vz *= FRICTION;

        // Cap horizontal speed
        let h_speed = (self.player.vx * self.player.vx + self.player.vz * self.player.vz).sqrt();
        if h_speed > MAX_SPEED {
            self.player.vx = self.player.vx / h_speed * MAX_SPEED;
            self.player.vz = self.player.vz / h_speed * MAX_SPEED;
        }

        // Apply velocity
        self.player.x += self.player.vx;
        self.player.y += self.player.vy;
        self.player.z += self.player.vz;

        // Ground collision
        let ground = self.world.height_at(self.player.x as i32, self.player.z as i32) + 1;
        if self.player.y <= ground as f64 {
            self.player.y = ground as f64;
            self.player.vy = 0.0;
            self.player.on_ground = true;
        } else {
            self.player.on_ground = false;
        }

        // Step sounds
        if self.player.on_ground && h_speed > 0.02 && self.tick % 8 == 0 {
            let block_under = self.world.get(
                self.player.x as i32,
                (self.player.y - 0.1) as i32,
                self.player.z as i32,
            );
            if let Some(ref s) = self.sound {
                s.play_step(block_under);
            }
        }

        self.player.x = self.player.x.clamp(1.0, 254.0);
        self.player.z = self.player.z.clamp(1.0, 254.0);

        // Generate chunks around player
        self.world.ensure_chunks_around(self.player.x, self.player.z);

        // Portal entry detection
        if self.tick % 10 == 0 {
            let block_at = self.world.get(
                self.player.x as i32,
                self.player.y as i32,
                self.player.z as i32,
            );
            if block_at == crate::block::BlockType::Portal {
                self.enter_portal();
            }
        }

        // Mob spawning (every 200 ticks, max 10 mobs)
        if self.tick % 200 == 0 && self.mobs.len() < 10 {
            let angle = (self.tick as f64 * 0.7) % (std::f64::consts::PI * 2.0);
            let spawn_dist = 15.0 + (self.tick % 100) as f64 * 0.1;
            let sx = self.player.x + angle.cos() * spawn_dist;
            let sz = self.player.z + angle.sin() * spawn_dist;
            let sy = self.world.height_at(sx as i32, sz as i32) + 1;
            if sx > 2.0 && sx < 254.0 && sz > 2.0 && sz < 254.0 {
                // Get biome at spawn location for mob type selection
                let biome = self.world.get_biome_at(sx as i32, sz as i32);
                let mob_type = MobType::for_biome(biome);
                self.mobs.push(Mob::new(mob_type, sx, sy as f64, sz));
            }
        }

        // Update mobs
        for mob in &mut self.mobs {
            mob.update(&self.world, self.player.x, self.player.y, self.player.z);
        }

        // Remove dead mobs
        self.mobs.retain(|m| m.health > 0);

        // Fluid simulation (every 40 ticks)
        if self.tick % 40 == 0 {
            let fluid_updates = FluidSystem::simulate(
                &self.world, self.player.x, self.player.z,
            );
            for (x, y, z, block) in fluid_updates {
                self.world.set(x, y, z, block);
            }
        }

        // Redstone CPU ticking (one cycle per game tick)
        if self.cpu.running {
            self.cpu.tick();
        }

        // Play mob sounds periodically
        if self.tick % 60 == 0 {
            if let Some(ref s) = self.sound {
                // Find nearest mob
                if let Some(nearest) = self.mobs.iter().min_by(|a, b| {
                    let da = (a.x - self.player.x).powi(2) + (a.z - self.player.z).powi(2);
                    let db = (b.x - self.player.x).powi(2) + (b.z - self.player.z).powi(2);
                    da.partial_cmp(&db).unwrap()
                }) {
                    let dist = ((nearest.x - self.player.x).powi(2)
                        + (nearest.z - self.player.z).powi(2)).sqrt();
                    if dist < 20.0 {
                        s.play_mob_sound(
                            nearest.x, nearest.z,
                            self.player.x, self.player.z,
                            self.player.yaw,
                        );
                    }
                }
            }
        }
    }

    fn render(&mut self, stdout: &mut impl Write) -> std::io::Result<()> {
        let frame_start = Instant::now();
        let daytime = DayTime::from_tick(self.tick, DAY_LENGTH);

        // Update redstone signals
        self.camera.redstone_signals = RedstoneSystem::propagate(
            &self.world, self.player.x, self.player.z,
        );

        let frame = self.camera.render(&self.player, &self.world, &daytime);
        let mut frame = frame;

        // Render mobs as overlay
        self.camera.render_mobs(&self.player, &self.mobs, &mut frame);

        // Apply bloom effect (every other frame for performance)
        if self.tick % 2 == 0 {
            self.camera.apply_bloom(&mut frame);
        }

        // Render mini-map
        Camera::render_minimap(&mut frame, &self.world, &self.player);

        // Get target block for HUD
        let target_block = self.camera.get_target_block(&self.player, &self.world);

        // Build hotbar string from inventory
        let hotbar_display = self.inventory.hotbar_display();
        let mut hotbar_str = String::from("[");
        for i in 0..9 {
            if i > 0 { hotbar_str.push('|'); }
            if i == self.inventory.selected {
                hotbar_str.push('►');
            } else if let Some((ref name, count)) = hotbar_display[i] {
                hotbar_str.push_str(&format!("{}×{}", name.chars().next().unwrap_or('?'), count));
            } else {
                hotbar_str.push(' ');
            }
        }
        hotbar_str.push(']');

        let dim_str = format!("[{}]", self.dimension.name());
        let fps = if self.frame_time_us > 0 { 1_000_000 / self.frame_time_us } else { 0 };
        let perf_str = format!("{}μs {}fps", self.frame_time_us, fps);
        let full_hud = format!("{} {} {}", dim_str, perf_str, hotbar_str);
        Camera::render_hud(&self.player, &daytime, &mut frame, target_block, &full_hud);

        // Double-buffered diff: only write changed cells
        let mut buf = String::with_capacity(VIEW_WIDTH * 2);
        for row in 0..VIEW_HEIGHT {
            let mut has_change = false;
            for col in 0..VIEW_WIDTH {
                if frame[row][col] != self.prev_frame[row][col] {
                    has_change = true;
                    break;
                }
            }
            if !has_change {
                continue;
            }
            // Batch consecutive changes on this row
            let mut col = 0;
            while col < VIEW_WIDTH {
                if frame[row][col] != self.prev_frame[row][col] {
                    // Seek to start of changed run
                    buf.clear();
                    buf.push_str(&format!("{}", cursor::MoveTo(col as u16, row as u16)));
                    // Write consecutive changed pixels
                    while col < VIEW_WIDTH && frame[row][col] != self.prev_frame[row][col] {
                        let (ch, color) = frame[row][col];
                        buf.push_str(&format!("{}{ch}{}", SetForegroundColor(color), ResetColor));
                        col += 1;
                    }
                    write!(stdout, "{buf}")?;
                } else {
                    col += 1;
                }
            }
        }

        stdout.flush()?;
        self.prev_frame = frame;
        self.frame_time_us = frame_start.elapsed().as_micros() as u64;
        Ok(())
    }

    fn save_game(&self) {
        match save::save_world(
            &self.world.blocks_ref(),
            self.player.x, self.player.y, self.player.z,
            self.player.yaw, self.player.pitch,
            self.tick,
        ) {
            Ok(()) => { /* silent success */ }
            Err(e) => { eprintln!("Save failed: {e}"); }
        }
    }

    fn enter_portal(&mut self) {
        match self.dimension {
            Dimension::Overworld => {
                // Save overworld position and switch to nether
                self.overworld_pos = Some((self.player.x, self.player.y, self.player.z));
                self.dimension = Dimension::Nether;
                // Create a nether world
                self.world = World::new(self.world.seed() + 1000);
                self.player.x = 128.0;
                self.player.z = 128.0;
                self.player.y = 30.0;
                self.mobs.clear();
            }
            Dimension::Nether => {
                // Return to overworld
                if let Some((x, y, z)) = self.overworld_pos {
                    self.player.x = x;
                    self.player.y = y;
                    self.player.z = z;
                }
                self.dimension = Dimension::Overworld;
                self.world = World::new(42);
                self.mobs.clear();
            }
        }
    }
}
