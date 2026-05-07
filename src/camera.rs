use crossterm::style::Color;

use crate::block::BlockType;
use crate::game::DayTime;
use crate::pbr;
use crate::player::Player;
use crate::world::World;

pub const VIEW_WIDTH: usize = 80;
pub const VIEW_HEIGHT: usize = 40;

/// Pre-computed color LUT for all BlockType variants (indexed by variant ordinal)
const COLOR_LUT: [(u8, u8, u8); 20] = [
    (0, 0, 0),       // Air
    (0, 180, 0),      // Grass
    (139, 119, 42),   // Dirt
    (160, 160, 160),  // Stone
    (220, 220, 100),  // Sand
    (30, 30, 220),    // Water
    (139, 69, 19),    // Wood
    (0, 100, 0),      // Leaves
    (200, 50, 200),   // Flower
    (0, 100, 0),      // TallGrass
    (0, 0, 0),        // CaveAir
    (220, 0, 0),      // RedstoneDust
    (255, 200, 0),    // RedstoneTorch
    (160, 160, 160),  // Lever
    (255, 255, 0),    // RedstoneLamp
    (100, 20, 20),    // Netherrack
    (180, 40, 40),    // NetherBrick
    (80, 0, 120),     // Obsidian
    (200, 50, 255),   // Portal
    (255, 80, 0),     // Lava
];

/// Glyph LUT for all BlockType variants
const GLYPH_LUT: [Option<char>; 20] = [
    None,       // Air
    Some('░'),  // Grass
    Some('▒'),  // Dirt
    Some('▓'),  // Stone
    Some('░'),  // Sand
    Some('≈'),  // Water
    Some('║'),  // Wood
    Some('♣'),  // Leaves
    Some('✿'),  // Flower
    Some('╿'),  // TallGrass
    None,       // CaveAir
    Some('·'),  // RedstoneDust
    Some('i'),  // RedstoneTorch
    Some('↑'),  // Lever
    Some('□'),  // RedstoneLamp
    Some('▒'),  // Netherrack
    Some('▓'),  // NetherBrick
    Some('█'),  // Obsidian
    Some('◎'),  // Portal
    Some('~'),  // Lava
];

#[inline(always)]
fn block_color_fast(block: BlockType) -> (u8, u8, u8) {
    let idx = block as usize;
    if idx < COLOR_LUT.len() {
        COLOR_LUT[idx]
    } else {
        (128, 128, 128)
    }
}

#[inline(always)]
fn block_glyph_fast(block: BlockType) -> Option<char> {
    let idx = block as usize;
    if idx < GLYPH_LUT.len() {
        GLYPH_LUT[idx]
    } else {
        None
    }
}

#[inline(always)]
fn color_to_rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb { r, g, b } => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::White => (255, 255, 255),
        Color::Red => (255, 0, 0),
        Color::Green => (0, 255, 0),
        Color::Blue => (0, 0, 255),
        Color::Yellow => (255, 255, 0),
        Color::Cyan => (0, 255, 255),
        Color::Magenta => (255, 0, 255),
        Color::Grey => (192, 192, 192),
        Color::DarkRed => (128, 0, 0),
        Color::DarkGreen => (0, 128, 0),
        Color::DarkYellow => (128, 128, 0),
        Color::DarkBlue => (0, 0, 128),
        Color::DarkMagenta => (128, 0, 128),
        Color::DarkCyan => (0, 128, 128),
        _ => (128, 128, 128),
    }
}

pub struct Camera {
    pub fov: f64,
    pub max_dist: f64,
    pub redstone_signals: Vec<((i32, i32, i32), u8)>,
    // Pre-allocated scratch buffers (zero-allocation rendering)
    bloom_r: Vec<Vec<f64>>,
    bloom_g: Vec<Vec<f64>>,
    bloom_b: Vec<Vec<f64>>,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            fov: 1.2,
            max_dist: 64.0,
            redstone_signals: Vec::new(),
            bloom_r: vec![vec![0.0; VIEW_WIDTH]; VIEW_HEIGHT],
            bloom_g: vec![vec![0.0; VIEW_WIDTH]; VIEW_HEIGHT],
            bloom_b: vec![vec![0.0; VIEW_WIDTH]; VIEW_HEIGHT],
        }
    }

    pub fn render(&mut self, player: &Player, world: &World, daytime: &DayTime) -> Vec<Vec<(char, Color)>> {
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

    /// Standard DDA raycasting — hot path optimized
    #[inline(always)]
    fn dda_cast(
        &self,
        ox: f64, oy: f64, oz: f64,
        dx: f64, dy: f64, dz: f64,
        world: &World,
        daytime: &DayTime,
    ) -> (char, Color) {
        let sky_rgb = daytime.sky_color();

        let mut ix = ox.floor() as i32;
        let mut iy = oy.floor() as i32;
        let mut iz = oz.floor() as i32;

        let step_x: i32 = if dx >= 0.0 { 1 } else { -1 };
        let step_y: i32 = if dy >= 0.0 { 1 } else { -1 };
        let step_z: i32 = if dz >= 0.0 { 1 } else { -1 };

        let inv_dx = if dx.abs() > 1e-10 { 1.0 / dx.abs() } else { f64::MAX };
        let inv_dy = if dy.abs() > 1e-10 { 1.0 / dy.abs() } else { f64::MAX };
        let inv_dz = if dz.abs() > 1e-10 { 1.0 / dz.abs() } else { f64::MAX };

        let mut t_max_x = if dx.abs() > 1e-10 {
            ((if step_x > 0 { ix as f64 + 1.0 } else { ix as f64 }) - ox) * inv_dx
        } else { f64::MAX };
        let mut t_max_y = if dy.abs() > 1e-10 {
            ((if step_y > 0 { iy as f64 + 1.0 } else { iy as f64 }) - oy) * inv_dy
        } else { f64::MAX };
        let mut t_max_z = if dz.abs() > 1e-10 {
            ((if step_z > 0 { iz as f64 + 1.0 } else { iz as f64 }) - oz) * inv_dz
        } else { f64::MAX };

        let max_dist = self.max_dist;
        let mut dist = 0.0f64;
        let mut last_axis = 0u8;

        while dist < max_dist {
            let block = world.get(ix, iy, iz);
            if block as u8 != 0 && block as u8 != 10 { // not Air and not CaveAir
                let is_transparent = block == BlockType::Flower
                    || block == BlockType::TallGrass
                    || block == BlockType::Water;

                if !is_transparent || dist < 2.0 {
                    let (br, bg, bb) = block_color_fast(block);
                    let material = pbr::get_material(block);

                    let face_light: f64 = if last_axis == 1 {
                        if dy > 0.0 { 0.6 } else { 1.0 }
                    } else {
                        0.8
                    };
                    let day_ambient = 0.3 + 0.7 * daytime.brightness;

                    // PBR lighting
                    let (pr, pg, pb) = pbr::apply_pbr(br, bg, bb, material, face_light, day_ambient);

                    // Emissive glow (redstone, lava, portal)
                    let glow_bonus: f64 = match block {
                        BlockType::RedstoneDust | BlockType::RedstoneLamp => {
                            if self.signal_at(ix, iy, iz).unwrap_or(0) > 0 { 0.4 } else { 0.0 }
                        }
                        BlockType::RedstoneTorch => 0.4,
                        BlockType::Lava | BlockType::Portal => 0.3,
                        _ => 0.0,
                    };

                    let fog = (dist / max_dist).min(1.0);
                    let fog_sq = fog * fog;
                    let inv_fog = 1.0 - fog_sq;
                    let glow_r = (pr as f64 + glow_bonus * 255.0).min(255.0);
                    let glow_g = (pg as f64 + glow_bonus * 200.0).min(255.0);
                    let glow_b = (pb as f64 + glow_bonus * 100.0).min(255.0);

                    let r = (glow_r * inv_fog + sky_rgb.0 as f64 * fog_sq) as u8;
                    let g = (glow_g * inv_fog + sky_rgb.1 as f64 * fog_sq) as u8;
                    let b = (glow_b * inv_fog + sky_rgb.2 as f64 * fog_sq) as u8;

                    let glyph = pbr::pbr_glyph(block_glyph_fast(block).unwrap_or('█'), material.roughness);
                    return (glyph, Color::Rgb { r, g, b });
                }
            }

            // DDA step — branchless axis selection
            if t_max_x < t_max_y {
                if t_max_x < t_max_z {
                    dist = t_max_x;
                    ix += step_x;
                    t_max_x += inv_dx;
                    last_axis = 0;
                } else {
                    dist = t_max_z;
                    iz += step_z;
                    t_max_z += inv_dz;
                    last_axis = 2;
                }
            } else {
                if t_max_y < t_max_z {
                    dist = t_max_y;
                    iy += step_y;
                    t_max_y += inv_dy;
                    last_axis = 1;
                } else {
                    dist = t_max_z;
                    iz += step_z;
                    t_max_z += inv_dz;
                    last_axis = 2;
                }
            }
        }

        // Sky
        let sky_t = (dy + 0.5).clamp(0.0, 1.0);
        let grad_r = (sky_rgb.0 as f64 * (0.4 + 0.6 * sky_t)) as u8;
        let grad_g = (sky_rgb.1 as f64 * (0.4 + 0.6 * sky_t)) as u8;
        let grad_b = (sky_rgb.2 as f64 * (0.4 + 0.6 * sky_t)) as u8;

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
    /// Render a mini-map in the top-right corner
    pub fn render_minimap(
        frame: &mut Vec<Vec<(char, Color)>>,
        world: &World,
        player: &Player,
    ) {
        let map_size = 15usize;
        let map_x = VIEW_WIDTH - map_size - 2;
        let map_y = 1;
        let px = player.x as i32;
        let pz = player.z as i32;

        // Border
        for i in 0..map_size + 2 {
            if map_x + i < VIEW_WIDTH && map_y < VIEW_HEIGHT {
                frame[map_y][map_x + i] = ('─', Color::DarkCyan);
                if map_y + map_size + 1 < VIEW_HEIGHT {
                    frame[map_y + map_size + 1][map_x + i] = ('─', Color::DarkCyan);
                }
            }
        }
        for j in 0..map_size + 2 {
            if map_x < VIEW_WIDTH && map_y + j < VIEW_HEIGHT {
                frame[map_y + j][map_x] = ('│', Color::DarkCyan);
                if map_x + map_size + 1 < VIEW_WIDTH {
                    frame[map_y + j][map_x + map_size + 1] = ('│', Color::DarkCyan);
                }
            }
        }
        // Corners
        if map_x < VIEW_WIDTH && map_y < VIEW_HEIGHT {
            frame[map_y][map_x] = ('┌', Color::DarkCyan);
        }
        if map_x + map_size + 1 < VIEW_WIDTH && map_y < VIEW_HEIGHT {
            frame[map_y][map_x + map_size + 1] = ('┐', Color::DarkCyan);
        }
        if map_x < VIEW_WIDTH && map_y + map_size + 1 < VIEW_HEIGHT {
            frame[map_y + map_size + 1][map_x] = ('└', Color::DarkCyan);
        }
        if map_x + map_size + 1 < VIEW_WIDTH && map_y + map_size + 1 < VIEW_HEIGHT {
            frame[map_y + map_size + 1][map_x + map_size + 1] = ('┘', Color::DarkCyan);
        }

        let half = map_size as i32 / 2;
        for dy in 0..map_size as i32 {
            for dx in 0..map_size as i32 {
                let wx = px + dx - half;
                let wz = pz + dy - half;
                let sy = map_y + 1 + dy as usize;
                let sx = map_x + 1 + dx as usize;

                if sx >= VIEW_WIDTH || sy >= VIEW_HEIGHT {
                    continue;
                }

                // Player marker
                if dx == half && dy == half {
                    frame[sy][sx] = ('@', Color::White);
                    continue;
                }

                let height = world.height_at(wx, wz);
                let block = world.get(wx, height, wz);
                let color = match block {
                    BlockType::Water => Color::Blue,
                    BlockType::Sand => Color::Yellow,
                    BlockType::Grass => Color::Green,
                    BlockType::Stone => Color::Grey,
                    BlockType::Leaves | BlockType::Wood => Color::DarkGreen,
                    BlockType::Netherrack => Color::DarkRed,
                    BlockType::Lava => Color::Red,
                    _ => Color::DarkGrey,
                };
                // Height-based brightness
                let ch = if height > 30 { '▲' } else if height > 20 { '▪' } else if height > 10 { '·' } else { '▾' };
                frame[sy][sx] = (ch, color);
            }
        }
    }

    /// Apply bloom effect using pre-allocated scratch buffers (zero heap allocation)
    pub fn apply_bloom(&mut self, frame: &mut Vec<Vec<(char, Color)>>) {
        let width = frame[0].len();
        let height = frame.len();

        // Clear scratch buffers
        for y in 0..height {
            for x in 0..width {
                self.bloom_r[y][x] = 0.0;
                self.bloom_g[y][x] = 0.0;
                self.bloom_b[y][x] = 0.0;
            }
        }

        // Accumulate bloom contributions
        for y in 0..height {
            for x in 0..width {
                let (r, g, b) = color_to_rgb(frame[y][x].1);
                let brightness = (r as f64 + g as f64 + b as f64) * (1.0 / 765.0);

                if brightness > 0.7 {
                    let intensity = (brightness - 0.7) * 2.0;

                    for dy in -1i32..=1 {
                        for dx in -1i32..=1 {
                            let ny = y as i32 + dy;
                            let nx = x as i32 + dx;
                            if ny >= 0 && ny < height as i32 && nx >= 0 && nx < width as i32 {
                                let dist = ((dx * dx + dy * dy) as f64).sqrt();
                                let weight = intensity * (1.0 - dist * 0.3).max(0.0) * 0.15;
                                let nyu = ny as usize;
                                let nxu = nx as usize;
                                self.bloom_r[nyu][nxu] += r as f64 * weight;
                                self.bloom_g[nyu][nxu] += g as f64 * weight;
                                self.bloom_b[nyu][nxu] += b as f64 * weight;
                            }
                        }
                    }
                }
            }
        }

        // Apply bloom to frame
        for y in 0..height {
            for x in 0..width {
                let br = self.bloom_r[y][x];
                let bg = self.bloom_g[y][x];
                let bb = self.bloom_b[y][x];
                if br > 1.0 || bg > 1.0 || bb > 1.0 {
                    let (r, g, b) = color_to_rgb(frame[y][x].1);
                    let nr = (r as f64 + br).min(255.0) as u8;
                    let ng = (g as f64 + bg).min(255.0) as u8;
                    let nb = (b as f64 + bb).min(255.0) as u8;
                    frame[y][x].1 = Color::Rgb { r: nr, g: ng, b: nb };
                }
            }
        }
    }

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
