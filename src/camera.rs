use crossterm::style::Color;

use crate::block::BlockType;
use crate::game::DayTime;
use crate::player::Player;
use crate::world::World;

pub const VIEW_WIDTH: usize = 80;
pub const VIEW_HEIGHT: usize = 40;

pub struct Camera {
    pub fov: f64,
    pub max_dist: f64,
    // Redstone signals: Vec of ((x,y,z), strength)
    pub redstone_signals: Vec<((i32, i32, i32), u8)>,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            fov: 1.2,
            max_dist: 64.0,
            redstone_signals: Vec::new(),
        }
    }

    pub fn render(&self, player: &Player, world: &World, daytime: &DayTime) -> Vec<Vec<(char, Color)>> {
        let mut frame = vec![vec![(' ', Color::Black); VIEW_WIDTH]; VIEW_HEIGHT];

        let eye_x = player.x;
        let eye_y = player.y + player.eye_height();
        let eye_z = player.z;

        let cos_pitch = player.pitch.cos();
        let sin_pitch = player.pitch.sin();
        let cos_yaw = player.yaw.cos();
        let sin_yaw = player.yaw.sin();

        for row in 0..VIEW_HEIGHT {
            for col in 0..VIEW_WIDTH {
                let u = (col as f64 - VIEW_WIDTH as f64 / 2.0) / VIEW_WIDTH as f64;
                let v = (VIEW_HEIGHT as f64 / 2.0 - row as f64) / VIEW_HEIGHT as f64;

                let dx_cam = u * self.fov;
                let dy_cam = v * self.fov;
                let dz_cam = 1.0;

                // Rotate by pitch then yaw
                let dy1 = dy_cam * cos_pitch - dz_cam * sin_pitch;
                let dz1 = dy_cam * sin_pitch + dz_cam * cos_pitch;
                let dx = dx_cam * cos_yaw + dz1 * sin_yaw;
                let dy = dy1;
                let dz = -dx_cam * sin_yaw + dz1 * cos_yaw;

                let (glyph, color) = self.dda_cast(eye_x, eye_y, eye_z, dx, dy, dz, world, daytime);
                frame[row][col] = (glyph, color);
            }
        }

        frame
    }

    /// Check if a position has a redstone signal
    fn signal_at(&self, x: i32, y: i32, z: i32) -> Option<u8> {
        self.redstone_signals.iter()
            .find(|&&((sx, sy, sz), _)| sx == x && sy == y && sz == z)
            .map(|&(_, s)| s)
    }

    /// Standard DDA (Digital Differential Analyzer) raycasting
    /// Steps through voxels by computing distance to next axis-aligned plane
    fn dda_cast(
        &self,
        ox: f64, oy: f64, oz: f64,
        dx: f64, dy: f64, dz: f64,
        world: &World,
        daytime: &DayTime,
    ) -> (char, Color) {
        let sky_rgb = daytime.sky_color();

        // Current voxel coordinates
        let mut ix = ox.floor() as i32;
        let mut iy = oy.floor() as i32;
        let mut iz = oz.floor() as i32;

        // Direction signs
        let step_x = if dx >= 0.0 { 1 } else { -1 };
        let step_y = if dy >= 0.0 { 1 } else { -1 };
        let step_z = if dz >= 0.0 { 1 } else { -1 };

        // Distance along ray to cross one voxel boundary in each axis
        let t_delta_x = if dx.abs() > 1e-10 { 1.0 / dx.abs() } else { f64::MAX };
        let t_delta_y = if dy.abs() > 1e-10 { 1.0 / dy.abs() } else { f64::MAX };
        let t_delta_z = if dz.abs() > 1e-10 { 1.0 / dz.abs() } else { f64::MAX };

        // Distance to first voxel boundary
        let mut t_max_x = if dx.abs() > 1e-10 {
            let boundary = if step_x > 0 { ix as f64 + 1.0 } else { ix as f64 };
            (boundary - ox) / dx
        } else {
            f64::MAX
        };
        let mut t_max_y = if dy.abs() > 1e-10 {
            let boundary = if step_y > 0 { iy as f64 + 1.0 } else { iy as f64 };
            (boundary - oy) / dy
        } else {
            f64::MAX
        };
        let mut t_max_z = if dz.abs() > 1e-10 {
            let boundary = if step_z > 0 { iz as f64 + 1.0 } else { iz as f64 };
            (boundary - oz) / dz
        } else {
            f64::MAX
        };

        let mut dist = 0.0;
        let max_dist = self.max_dist;
        let mut last_axis = 0u8; // 0=x, 1=y, 2=z

        // March through voxels
        while dist < max_dist {
            // Check current voxel
            let block = world.get(ix, iy, iz);
            if block != BlockType::Air && block != BlockType::CaveAir {
                let is_transparent = block == BlockType::Flower
                    || block == BlockType::TallGrass
                    || block == BlockType::Water;

                // Only render if close enough or opaque
                if !is_transparent || dist < 2.0 {
                    let base_color = block.color().unwrap_or(Color::White);

                    // Redstone blocks glow when powered
                    let is_lit = match block {
                        BlockType::RedstoneDust | BlockType::RedstoneLamp => {
                            self.signal_at(ix, iy, iz).unwrap_or(0) > 0
                        }
                        BlockType::RedstoneTorch => true, // torches always lit
                        _ => false,
                    };

                    // Face-dependent lighting
                    let face_light = match last_axis {
                        1 => if dy > 0.0 { 0.6 } else { 1.0 },
                        _ => 0.8,
                    };
                    let day_ambient = 0.3 + 0.7 * daytime.brightness;
                    let glow_bonus = if is_lit { 0.4 } else { 0.0 };
                    let raw_brightness = (face_light * day_ambient + glow_bonus).min(1.0);

                    // Distance fog
                    let fog_factor = (dist / max_dist).clamp(0.0, 1.0);
                    let fog_sq = fog_factor * fog_factor;
                    let block_rgb = color_to_rgb(base_color);
                    let r = (block_rgb.0 as f64 * raw_brightness * (1.0 - fog_sq)
                        + sky_rgb.0 as f64 * fog_sq) as u8;
                    let g = (block_rgb.1 as f64 * raw_brightness * (1.0 - fog_sq)
                        + sky_rgb.1 as f64 * fog_sq) as u8;
                    let b = (block_rgb.2 as f64 * raw_brightness * (1.0 - fog_sq)
                        + sky_rgb.2 as f64 * fog_sq) as u8;

                    return (block.glyph().unwrap_or('█'), Color::Rgb { r, g, b });
                }
            }

            // Step to next voxel boundary (whichever axis is closest)
            if t_max_x < t_max_y {
                if t_max_x < t_max_z {
                    dist = t_max_x;
                    ix += step_x;
                    t_max_x += t_delta_x;
                    last_axis = 0;
                } else {
                    dist = t_max_z;
                    iz += step_z;
                    t_max_z += t_delta_z;
                    last_axis = 2;
                }
            } else {
                if t_max_y < t_max_z {
                    dist = t_max_y;
                    iy += step_y;
                    t_max_y += t_delta_y;
                    last_axis = 1;
                } else {
                    dist = t_max_z;
                    iz += step_z;
                    t_max_z += t_delta_z;
                    last_axis = 2;
                }
            }
        }

        // Sky
        let sky_t = (dy + 0.5).clamp(0.0, 1.0);
        let grad_r = (sky_rgb.0 as f64 * (0.4 + 0.6 * sky_t)) as u8;
        let grad_g = (sky_rgb.1 as f64 * (0.4 + 0.6 * sky_t)) as u8;
        let grad_b = (sky_rgb.2 as f64 * (0.4 + 0.6 * sky_t)) as u8;

        // Stars at night
        if daytime.brightness < 0.3 && dy > 0.2 {
            let star_hash = ((ox * 137.0 + oz * 311.0 + dy * 997.0) as i32).unsigned_abs() % 100;
            if star_hash < 3 {
                return ('✦', Color::White);
            }
        }

        ('·', Color::Rgb { r: grad_r, g: grad_g, b: grad_b })
    }

    pub fn render_hud(
        player: &Player,
        daytime: &DayTime,
        frame: &mut Vec<Vec<(char, Color)>>,
        target_block: Option<BlockType>,
        hotbar_str: &str,
    ) {
        let time_str = if daytime.phase < 0.2 || daytime.phase > 0.85 {
            "🌙"
        } else if daytime.phase < 0.35 {
            "🌅"
        } else if daytime.phase < 0.65 {
            "☀️"
        } else {
            "🌇"
        };
        let target_str = match target_block {
            Some(b) => format!("Looking at: {:?}", b),
            None => String::new(),
        };

        let hud = format!(
            " {}  {}  Pos:({:.0},{:.0},{:.0})  WASD:move SPACE:jump E:place Q:break F5:save ESC:quit",
            hotbar_str, time_str, player.x, player.y, player.z,
        );

        // Top line: target block info
        let top_row = 0;
        for (i, ch) in target_str.chars().enumerate() {
            if i < VIEW_WIDTH {
                frame[top_row][i] = (ch, Color::Cyan);
            }
        }

        // Bottom line: hotbar
        let bot_row = VIEW_HEIGHT - 1;
        for (i, ch) in hud.chars().enumerate() {
            if i < VIEW_WIDTH {
                frame[bot_row][i] = (ch, Color::White);
            }
        }

        // Crosshair in center
        let cx = VIEW_WIDTH / 2;
        let cy = VIEW_HEIGHT / 2;
        frame[cy][cx] = ('+', Color::White);
    }

    /// Returns the block type the player is looking at (for HUD display)
    pub fn get_target_block(&self, player: &Player, world: &World) -> Option<BlockType> {
        let eye_x = player.x;
        let eye_y = player.y + player.eye_height();
        let eye_z = player.z;

        let cos_pitch = player.pitch.cos();
        let sin_pitch = player.pitch.sin();
        let cos_yaw = player.yaw.cos();
        let sin_yaw = player.yaw.sin();

        let dz1 = cos_pitch; // forward component after pitch
        let dy1 = -sin_pitch;
        let dx = dz1 * sin_yaw;
        let dy = dy1;
        let dz = dz1 * cos_yaw;

        // DDA along center ray
        let mut ix = eye_x.floor() as i32;
        let mut iy = eye_y.floor() as i32;
        let mut iz = eye_z.floor() as i32;

        let step_x = if dx >= 0.0 { 1 } else { -1 };
        let step_y = if dy >= 0.0 { 1 } else { -1 };
        let step_z = if dz >= 0.0 { 1 } else { -1 };

        let t_delta_x = if dx.abs() > 1e-10 { 1.0 / dx.abs() } else { f64::MAX };
        let t_delta_y = if dy.abs() > 1e-10 { 1.0 / dy.abs() } else { f64::MAX };
        let t_delta_z = if dz.abs() > 1e-10 { 1.0 / dz.abs() } else { f64::MAX };

        let mut t_max_x = if dx.abs() > 1e-10 {
            let b = if step_x > 0 { ix as f64 + 1.0 } else { ix as f64 };
            (b - eye_x) / dx
        } else { f64::MAX };
        let mut t_max_y = if dy.abs() > 1e-10 {
            let b = if step_y > 0 { iy as f64 + 1.0 } else { iy as f64 };
            (b - eye_y) / dy
        } else { f64::MAX };
        let mut t_max_z = if dz.abs() > 1e-10 {
            let b = if step_z > 0 { iz as f64 + 1.0 } else { iz as f64 };
            (b - eye_z) / dz
        } else { f64::MAX };

        let mut dist = 0.0;
        while dist < 8.0 {
            let block = world.get(ix, iy, iz);
            if block != BlockType::Air && block != BlockType::CaveAir {
                return Some(block);
            }

            if t_max_x < t_max_y {
                if t_max_x < t_max_z {
                    dist = t_max_x;
                    ix += step_x;
                    t_max_x += t_delta_x;
                } else {
                    dist = t_max_z;
                    iz += step_z;
                    t_max_z += t_delta_z;
                }
            } else {
                if t_max_y < t_max_z {
                    dist = t_max_y;
                    iy += step_y;
                    t_max_y += t_delta_y;
                } else {
                    dist = t_max_z;
                    iz += step_z;
                    t_max_z += t_delta_z;
                }
            }
        }
        None
    }

    /// Render mobs as overlay characters on the frame buffer
    pub fn render_mobs(
        &self,
        player: &Player,
        mobs: &[crate::mob::Mob],
        frame: &mut Vec<Vec<(char, Color)>>,
    ) {
        let eye_x = player.x;
        let eye_y = player.y + player.eye_height();
        let eye_z = player.z;

        let cos_pitch = player.pitch.cos();
        let sin_pitch = player.pitch.sin();
        let cos_yaw = player.yaw.cos();
        let sin_yaw = player.yaw.sin();

        for mob in mobs {
            // Vector from eye to mob
            let dx = mob.x - eye_x;
            let dy = mob.y + 0.5 - eye_y; // center of mob
            let dz = mob.z - eye_z;

            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            if dist > self.max_dist || dist < 0.5 {
                continue;
            }

            // Transform to camera space (inverse of the rotation applied to rays)
            // First undo yaw: rotate by -yaw
            let cam_x = dx * cos_yaw - dz * sin_yaw;
            let cam_z = dx * sin_yaw + dz * cos_yaw;
            // Then undo pitch: rotate by -pitch
            let cam_y = dy * cos_pitch + cam_z * sin_pitch;
            let cam_z2 = -dy * sin_pitch + cam_z * cos_pitch;

            // cam_z2 is forward direction; must be positive (in front of camera)
            if cam_z2 <= 0.1 {
                continue;
            }

            // Project to screen
            let screen_x = (cam_x / cam_z2 / self.fov * VIEW_WIDTH as f64 + VIEW_WIDTH as f64 / 2.0) as i32;
            let screen_y = (-cam_y / cam_z2 / self.fov * VIEW_HEIGHT as f64 + VIEW_HEIGHT as f64 / 2.0) as i32;

            if screen_x >= 0 && screen_x < VIEW_WIDTH as i32 && screen_y >= 0 && screen_y < VIEW_HEIGHT as i32 {
                let sx = screen_x as usize;
                let sy = screen_y as usize;
                frame[sy][sx] = (mob.glyph(), mob.color());
            }
        }
    }
}

fn color_to_rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Black => (0, 0, 0),
        Color::DarkRed => (128, 0, 0),
        Color::DarkGreen => (0, 128, 0),
        Color::DarkYellow => (128, 128, 0),
        Color::DarkBlue => (0, 0, 128),
        Color::DarkMagenta => (128, 0, 128),
        Color::DarkCyan => (0, 128, 128),
        Color::Grey => (192, 192, 192),
        Color::Red => (255, 0, 0),
        Color::Green => (0, 255, 0),
        Color::Yellow => (255, 255, 0),
        Color::Blue => (0, 0, 255),
        Color::Magenta => (255, 0, 255),
        Color::Cyan => (0, 255, 255),
        Color::White => (255, 255, 255),
        Color::Rgb { r, g, b } => (r, g, b),
        _ => (128, 128, 128),
    }
}
