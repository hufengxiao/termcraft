use serde::{Deserialize, Serialize};
use noise::{NoiseFn, Simplex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Biome {
    Plains,
    Desert,
    Snow,
    Forest,
    Mountains,
    Ocean,
}

impl Biome {
    pub fn tree_density(self) -> f64 {
        match self {
            Self::Forest => 0.08,
            Self::Plains => 0.02,
            Self::Mountains => 0.01,
            _ => 0.0,
        }
    }

    pub fn surface_block(self) -> crate::block::BlockType {
        match self {
            Self::Desert => crate::block::BlockType::Sand,
            Self::Snow => crate::block::BlockType::Grass, // snow overlay later
            _ => crate::block::BlockType::Grass,
        }
    }

    pub fn sub_surface_block(self) -> crate::block::BlockType {
        match self {
            Self::Desert => crate::block::BlockType::Sand,
            _ => crate::block::BlockType::Dirt,
        }
    }

    pub fn height_scale(self) -> f64 {
        match self {
            Self::Mountains => 2.0,
            Self::Plains => 0.6,
            Self::Desert => 0.4,
            Self::Ocean => 0.3,
            _ => 1.0,
        }
    }

    pub fn base_height(self) -> f64 {
        match self {
            Self::Ocean => -5.0,
            Self::Mountains => 10.0,
            _ => 0.0,
        }
    }
}

pub struct BiomeMap {
    simplex: Simplex,
    temperature: Simplex,
    moisture: Simplex,
}

impl BiomeMap {
    pub fn new(seed: u32) -> Self {
        Self {
            simplex: Simplex::new(seed + 200),
            temperature: Simplex::new(seed + 300),
            moisture: Simplex::new(seed + 400),
        }
    }

    pub fn get_biome(&self, x: i32, z: i32) -> Biome {
        let nx = x as f64 / 200.0;
        let nz = z as f64 / 200.0;

        let temp = self.temperature.get([nx, nz]); // -1 to 1
        let moist = self.moisture.get([nx * 1.5, nz * 1.5]); // -1 to 1

        // Biome selection based on temperature and moisture
        if temp < -0.4 {
            Biome::Snow
        } else if temp > 0.3 && moist < -0.2 {
            Biome::Desert
        } else if moist > 0.3 {
            Biome::Forest
        } else {
            // Check for mountains
            let mountain_noise = self.simplex.get([nx * 0.5, nz * 0.5]);
            if mountain_noise > 0.3 {
                Biome::Mountains
            } else {
                Biome::Plains
            }
        }
    }

    /// Get blended terrain height considering biome
    pub fn get_height(&self, x: i32, z: i32) -> i32 {
        let biome = self.get_biome(x, z);
        let nx = x as f64 / 80.0;
        let nz = z as f64 / 80.0;

        let h1 = self.simplex.get([nx, nz]) * 15.0;
        let h2 = self.simplex.get([nx * 2.0, nz * 2.0]) * 7.0;
        let h3 = self.simplex.get([nx * 4.0, nz * 4.0]) * 3.0;

        let raw_height = h1 + h2 + h3;
        let scaled = raw_height * biome.height_scale() + biome.base_height();
        let height = (20.0 + scaled) as i32; // SEA_LEVEL = 20
        height.clamp(1, 63) as i32
    }
}
