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
}

impl Camera {
    pub fn new() -> Self {
        Self {
            fov: 1.2,
            max_dist: 32.0,
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

                let dy1 = dy_cam * cos_pitch - dz_cam * sin_pitch;
                let dz1 = dy_cam * sin_pitch + dz_cam * cos_pitch;
                let dx = dx_cam * cos_yaw + dz1 * sin_yaw;
                let dy = dy1;
                let dz = -dx_cam * sin_yaw + dz1 * cos_yaw;

                let len = (dx * dx + dy * dy + dz * dz).sqrt();
                let (rdx, rdy, rdz) = (dx / len, dy / len, dz / len);

                let (glyph, color) = self.ray_march(eye_x, eye_y, eye_z, rdx, rdy, rdz, world, daytime);
                frame[row][col] = (glyph, color);
            }
        }

        frame
    }

    fn ray_march(
        &self,
        ox: f64, oy: f64, oz: f64,
        dx: f64, dy: f64, dz: f64,
        world: &World,
        daytime: &DayTime,
    ) -> (char, Color) {
        let mut x = ox;
        let mut y = oy;
        let mut z = oz;
        let step = 0.05;
        let max_steps = (self.max_dist / step) as i32;
        let sky_rgb = daytime.sky_color();

        for _ in 0..max_steps {
            x += dx * step;
            y += dy * step;
            z += dz * step;

            let bx = x.floor() as i32;
            let by = y.floor() as i32;
            let bz = z.floor() as i32;

            let block = world.get(bx, by, bz);
            if block != BlockType::Air && block != BlockType::CaveAir {
                // Transparent blocks (flowers, tall grass, water) - render but don't fully occlude
                let is_transparent = block == BlockType::Flower
                    || block == BlockType::TallGrass
                    || block == BlockType::Water;

                let dist = ((x - ox).powi(2) + (y - oy).powi(2) + (z - oz).powi(2)).sqrt();
                let base_color = block.color().unwrap_or(Color::White);

                let face_light = if dy > 0.1 { 1.0 } else if dy < -0.1 { 0.6 } else { 0.8 };
                let day_ambient = 0.3 + 0.7 * daytime.brightness;
                let raw_brightness = face_light * day_ambient;

                let fog_factor = (dist / self.max_dist).clamp(0.0, 1.0);
                let fog_factor = fog_factor * fog_factor;
                let block_rgb = color_to_rgb(base_color);
                let r = (block_rgb.0 as f64 * raw_brightness * (1.0 - fog_factor) + sky_rgb.0 as f64 * fog_factor) as u8;
                let g = (block_rgb.1 as f64 * raw_brightness * (1.0 - fog_factor) + sky_rgb.1 as f64 * fog_factor) as u8;
                let b = (block_rgb.2 as f64 * raw_brightness * (1.0 - fog_factor) + sky_rgb.2 as f64 * fog_factor) as u8;

                // For transparent blocks, only return if close enough, otherwise keep marching
                if !is_transparent || dist < 2.0 {
                    return (block.glyph().unwrap_or('█'), Color::Rgb { r, g, b });
                }
            }
        }

        // Sky rendering with gradient
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

    pub fn render_hud(player: &Player, daytime: &DayTime, frame: &mut Vec<Vec<(char, Color)>>) {
        let block_names = ["Grass", "Dirt", "Stone", "Sand", "Wood"];
        let time_str = if daytime.phase < 0.2 || daytime.phase > 0.85 {
            "🌙 Night"
        } else if daytime.phase < 0.35 {
            "🌅 Sunrise"
        } else if daytime.phase < 0.65 {
            "☀️  Day"
        } else {
            "🌇 Sunset"
        };
        let hotbar = format!(
            " [{}|{}|{}|{}|{}]  Block: {}  Pos: ({:.0},{:.0},{:.0})  {}  WASD:move SPACE:jump E:place Q:break F5:save ESC:quit",
            if player.selected_block == 0 { "►" } else { " " },
            if player.selected_block == 1 { "►" } else { " " },
            if player.selected_block == 2 { "►" } else { " " },
            if player.selected_block == 3 { "►" } else { " " },
            if player.selected_block == 4 { "►" } else { " " },
            block_names.get(player.selected_block).unwrap_or(&"?"),
            player.x, player.y, player.z,
            time_str,
        );

        let row = VIEW_HEIGHT - 1;
        for (i, ch) in hotbar.chars().enumerate() {
            if i < VIEW_WIDTH {
                frame[row][i] = (ch, Color::White);
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
