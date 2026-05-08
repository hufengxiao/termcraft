pub struct Player {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub yaw: f64,
    pub pitch: f64,
    pub on_ground: bool,
    // Survival stats
    pub health: f64,     // 0-20 (10 hearts)
    pub hunger: f64,     // 0-20 (10 drumsticks)
    pub saturation: f64, // 0-20 (hidden, slows hunger drain)
    pub damage_timer: u64, // ticks since last damage (invulnerability frames)
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
            health: 20.0,
            hunger: 20.0,
            saturation: 5.0,
            damage_timer: 100,
        }
    }

    pub fn forward_dir(&self) -> (f64, f64) {
        (self.yaw.sin(), self.yaw.cos())
    }

    pub fn eye_height(&self) -> f64 {
        1.6
    }

    pub fn take_damage(&mut self, amount: f64) {
        if self.damage_timer < 20 { return; } // invulnerability frames
        self.health = (self.health - amount).max(0.0);
        self.damage_timer = 0;
    }

    #[allow(dead_code)]
    pub fn heal(&mut self, amount: f64) {
        self.health = (self.health + amount).min(20.0);
    }

    pub fn eat(&mut self, hunger_restore: f64, saturation_restore: f64) {
        self.hunger = (self.hunger + hunger_restore).min(20.0);
        self.saturation = (self.saturation + saturation_restore).min(self.hunger);
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    /// Called each tick to update survival stats
    pub fn survival_tick(&mut self) {
        self.damage_timer += 1;

        // Hunger drain (very slow, ~1 point per 4 minutes at 20 TPS)
        if self.saturation > 0.0 {
            self.saturation -= 0.001;
        } else {
            self.hunger -= 0.0005;
        }

        // Health regeneration when well-fed (hunger >= 18)
        if self.hunger >= 18.0 && self.health < 20.0 {
            self.health += 0.05;
        }

        // Starvation damage when hunger is 0
        if self.hunger <= 0.0 {
            self.take_damage(0.1);
        }

        self.hunger = self.hunger.clamp(0.0, 20.0);
        self.health = self.health.clamp(0.0, 20.0);
    }
}
