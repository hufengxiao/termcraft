use std::io::{stdout, Write};
use crossterm::{
    cursor,
    execute,
    style::{ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};

use crate::camera::{Camera, VIEW_HEIGHT, VIEW_WIDTH};
use crate::input::{Action, Input};
use crate::player::Player;
use crate::save;
use crate::sound::SoundEngine;
use crate::world::World;

const GRAVITY: f64 = -0.02;
const JUMP_VEL: f64 = 0.25;
const MOVE_SPEED: f64 = 0.15;
const TICK_MS: u64 = 50; // 20 TPS
const DAY_LENGTH: u64 = 2400; // ticks per full day cycle (2 minutes)

pub struct Game {
    world: World,
    player: Player,
    camera: Camera,
    running: bool,
    tick: u64,
    sound: Option<SoundEngine>,
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
                let mx = dx * fy + dz * fx;
                let mz = -dx * fx + dz * fy;
                let len = (mx * mx + mz * mz).sqrt();
                if len > 0.0 {
                    self.player.x += mx / len * MOVE_SPEED;
                    self.player.z += mz / len * MOVE_SPEED;
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
                let block = crate::block::BlockType::all_buildable()
                    .get(self.player.selected_block)
                    .copied()
                    .unwrap_or(crate::block::BlockType::Stone);
                self.world.set(px as i32, py as i32, pz as i32, block);
                if let Some(ref s) = self.sound { s.play_place(); }
            }
            Action::Break => {
                let (fx, fz) = self.player.forward_dir();
                let px = self.player.x + fx * 2.0;
                let pz = self.player.z + fz * 2.0;
                let py = self.player.y + 1.0;
                self.world.set(px as i32, py as i32, pz as i32, crate::block::BlockType::Air);
                if let Some(ref s) = self.sound { s.play_break(); }
            }
            Action::SelectBlock(i) => {
                self.player.selected_block = i;
            }
            Action::Save => {
                self.save_game();
            }
            Action::None => {}
        }
    }

    fn physics(&mut self) {
        self.player.vy += GRAVITY;
        self.player.y += self.player.vy;

        let ground = self.world.height_at(self.player.x as i32, self.player.z as i32) + 1;
        if self.player.y <= ground as f64 {
            self.player.y = ground as f64;
            self.player.vy = 0.0;
            self.player.on_ground = true;
        } else {
            self.player.on_ground = false;
        }

        self.player.x = self.player.x.clamp(1.0, 254.0);
        self.player.z = self.player.z.clamp(1.0, 254.0);
    }

    fn render(&self, stdout: &mut impl Write) -> std::io::Result<()> {
        let daytime = DayTime::from_tick(self.tick, DAY_LENGTH);
        let frame = self.camera.render(&self.player, &self.world, &daytime);
        let mut frame = frame;

        Camera::render_hud(&self.player, &daytime, &mut frame);

        execute!(stdout, cursor::MoveTo(0, 0))?;
        let mut buf = String::with_capacity(VIEW_WIDTH * VIEW_HEIGHT * 4);
        for row in &frame {
            for (ch, color) in row {
                buf.push_str(&format!("{}{ch}{}", SetForegroundColor(*color), ResetColor));
            }
            buf.push('\r');
            buf.push('\n');
        }
        write!(stdout, "{buf}")?;
        stdout.flush()?;
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
}
