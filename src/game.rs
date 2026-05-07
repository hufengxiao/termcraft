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
use crate::world::World;

const GRAVITY: f64 = -0.02;
const JUMP_VEL: f64 = 0.25;
const MOVE_SPEED: f64 = 0.15;
const TICK_MS: u64 = 50; // 20 TPS

pub struct Game {
    world: World,
    player: Player,
    camera: Camera,
    running: bool,
}

impl Game {
    pub fn new() -> Self {
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

        // Main game loop
        while self.running {
            let action = Input::poll();
            self.handle_action(action);
            self.physics();
            self.render(&mut stdout)?;

            std::thread::sleep(std::time::Duration::from_millis(TICK_MS));
        }

        // Cleanup
        execute!(
            stdout,
            cursor::Show,
            terminal::LeaveAlternateScreen
        )?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.running = false,
            Action::Move { dx, dz } => {
                let (fy, fx) = self.player.forward_dir();
                // Rotate movement by yaw
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
            }
            Action::Break => {
                let (fx, fz) = self.player.forward_dir();
                let px = self.player.x + fx * 2.0;
                let pz = self.player.z + fz * 2.0;
                let py = self.player.y + 1.0;
                self.world.set(px as i32, py as i32, pz as i32, crate::block::BlockType::Air);
            }
            Action::SelectBlock(i) => {
                self.player.selected_block = i;
            }
            Action::None => {}
        }
    }

    fn physics(&mut self) {
        // Gravity
        self.player.vy += GRAVITY;
        self.player.y += self.player.vy;

        // Ground collision
        let ground = self.world.height_at(self.player.x as i32, self.player.z as i32) + 1;
        if self.player.y <= ground as f64 {
            self.player.y = ground as f64;
            self.player.vy = 0.0;
            self.player.on_ground = true;
        } else {
            self.player.on_ground = false;
        }

        // Clamp to world
        self.player.x = self.player.x.clamp(1.0, 254.0);
        self.player.z = self.player.z.clamp(1.0, 254.0);
    }

    fn render(&self, stdout: &mut impl Write) -> std::io::Result<()> {
        let frame = self.camera.render(&self.player, &self.world);
        let mut frame = frame;

        // Draw HUD overlay
        Camera::render_hud(&self.player, &mut frame);

        // Write to terminal
        execute!(stdout, cursor::MoveTo(0, 0))?;
        let mut buf = String::with_capacity(VIEW_WIDTH * VIEW_HEIGHT * 4);
        for row in &frame {
            for (ch, color) in row {
                buf.push_str(&format!(
                    "{}{ch}{}",
                    SetForegroundColor(*color),
                    ResetColor
                ));
            }
            buf.push('\r');
            buf.push('\n');
        }
        write!(stdout, "{buf}")?;
        stdout.flush()?;
        Ok(())
    }
}
