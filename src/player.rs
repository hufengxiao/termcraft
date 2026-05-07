pub struct Player {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub yaw: f64,   // horizontal angle in radians
    pub pitch: f64, // vertical angle in radians
    pub on_ground: bool,
    pub selected_block: usize,
}

impl Player {
    pub fn new(spawn_x: f64, spawn_y: f64, spawn_z: f64) -> Self {
        Self {
            x: spawn_x,
            y: spawn_y,
            z: spawn_z,
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: false,
            selected_block: 0,
        }
    }

    pub fn forward_dir(&self) -> (f64, f64) {
        (self.yaw.sin(), self.yaw.cos())
    }

    pub fn eye_height(&self) -> f64 {
        1.6
    }
}
