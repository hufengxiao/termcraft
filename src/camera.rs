use crossterm::style::Color;

use crate::block::BlockType;
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
            fov: 1.2, // ~69 degrees
            max_dist: 32.0,
        }
    }

    pub fn render(&self, player: &Player, world: &World) -> Vec<Vec<(char, Color)>> {
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
                // Map screen to ray direction
                let u = (col as f64 - VIEW_WIDTH as f64 / 2.0) / VIEW_WIDTH as f64;
                let v = (VIEW_HEIGHT as f64 / 2.0 - row as f64) / VIEW_HEIGHT as f64;

                // Ray direction in camera space
                let dx_cam = u * self.fov;
                let dy_cam = v * self.fov;
                let dz_cam = 1.0;

                // Rotate by pitch then yaw
                let dy1 = dy_cam * cos_pitch - dz_cam * sin_pitch;
                let dz1 = dy_cam * sin_pitch + dz_cam * cos_pitch;
                let dx = dx_cam * cos_yaw + dz1 * sin_yaw;
                let dy = dy1;
                let dz = -dx_cam * sin_yaw + dz1 * cos_yaw;

                // Normalize
                let len = (dx * dx + dy * dy + dz * dz).sqrt();
                let (rdx, rdy, rdz) = (dx / len, dy / len, dz / len);

                // DDA ray march
                let (glyph, color) = self.ray_march(eye_x, eye_y, eye_z, rdx, rdy, rdz, world);
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
    ) -> (char, Color) {
        let mut x = ox;
        let mut y = oy;
        let mut z = oz;
        let step = 0.05;

        for _ in 0..(self.max_dist / step) as i32 {
            x += dx * step;
            y += dy * step;
            z += dz * step;

            let bx = x.floor() as i32;
            let by = y.floor() as i32;
            let bz = z.floor() as i32;

            let block = world.get(bx, by, bz);
            if block != BlockType::Air {
                // Simple lighting: darker at lower Y, brighter facing up
                let base_color = block.color().unwrap_or(Color::White);
                let light = ((y - oy) / self.max_dist).abs();
                let brightness = (0.4 + 0.6 * (1.0 - light)).min(1.0);

                return (block.glyph().unwrap_or('█'), darken(base_color, brightness));
            }
        }

        // Sky gradient
        let sky_t = dy.max(0.0);
        if sky_t > 0.3 {
            ('·', Color::DarkBlue)
        } else if sky_t > 0.0 {
            ('·', Color::Blue)
        } else {
            (' ', Color::Black)
        }
    }

    pub fn render_hud(
        player: &Player,
        frame: &mut Vec<Vec<(char, Color)>>,
    ) {
        let block_names = ["Grass", "Dirt", "Stone", "Sand", "Wood"];
        let hotbar = format!(
            " [{}|{}|{}|{}|{}]  Block: {}  Pos: ({:.0},{:.0},{:.0})  Arrows:look WASD:move SPACE:jump E:place Q:break ESC:quit",
            if player.selected_block == 0 { "►" } else { " " },
            if player.selected_block == 1 { "►" } else { " " },
            if player.selected_block == 2 { "►" } else { " " },
            if player.selected_block == 3 { "►" } else { " " },
            if player.selected_block == 4 { "►" } else { " " },
            block_names.get(player.selected_block).unwrap_or(&"?"),
            player.x, player.y, player.z,
        );

        // Bottom HUD line
        let row = VIEW_HEIGHT - 1;
        for (i, ch) in hotbar.chars().enumerate() {
            if i < VIEW_WIDTH {
                frame[row][i] = (ch, Color::White);
            }
        }
    }
}

fn darken(color: Color, factor: f64) -> Color {
    let f = factor.clamp(0.0, 1.0);
    match color {
        Color::Black => Color::Black,
        Color::DarkRed => rgb(128, 0, 0, f),
        Color::DarkGreen => rgb(0, 128, 0, f),
        Color::DarkYellow => rgb(128, 128, 0, f),
        Color::DarkBlue => rgb(0, 0, 128, f),
        Color::DarkMagenta => rgb(128, 0, 128, f),
        Color::DarkCyan => rgb(0, 128, 128, f),
        Color::Grey => rgb(192, 192, 192, f),
        Color::Red => rgb(255, 0, 0, f),
        Color::Green => rgb(0, 255, 0, f),
        Color::Yellow => rgb(255, 255, 0, f),
        Color::Blue => rgb(0, 0, 255, f),
        Color::Magenta => rgb(255, 0, 255, f),
        Color::Cyan => rgb(0, 255, 255, f),
        Color::White => rgb(255, 255, 255, f),
        _ => color,
    }
}

fn rgb(r: u8, g: u8, b: u8, factor: f64) -> Color {
    Color::Rgb {
        r: (r as f64 * factor) as u8,
        g: (g as f64 * factor) as u8,
        b: (b as f64 * factor) as u8,
    }
}
