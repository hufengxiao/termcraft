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
    Swamp,
    Jungle,
    Badlands,
    MushroomIsland,
}

impl Biome {
    pub fn tree_density(self) -> f64 {
        match self {
            Self::Forest => 0.08,
            Self::Jungle => 0.12,
            Self::Plains => 0.02,
            Self::Swamp => 0.03,
            Self::Mountains => 0.01,
            Self::MushroomIsland => 0.0,
            _ => 0.0,
        }
    }

    pub fn surface_block(self) -> crate::block::BlockType {
        match self {
            Self::Desert | Self::Badlands => crate::block::BlockType::Sand,
            Self::MushroomIsland => crate::block::BlockType::Grass,
            _ => crate::block::BlockType::Grass,
        }
    }

    pub fn sub_surface_block(self) -> crate::block::BlockType {
        match self {
            Self::Desert | Self::Badlands => crate::block::BlockType::Sand,
            Self::Swamp => crate::block::BlockType::Dirt,
            _ => crate::block::BlockType::Dirt,
        }
    }

    pub fn height_scale(self) -> f64 {
        match self {
            Self::Mountains => 2.0,
            Self::Plains => 0.6,
            Self::Desert => 0.4,
            Self::Ocean => 0.3,
            Self::Swamp => 0.2,
            Self::MushroomIsland => 0.3,
            Self::Badlands => 1.5,
            _ => 1.0,
        }
    }

    pub fn base_height(self) -> f64 {
        match self {
            Self::Ocean => -5.0,
            Self::Swamp => -2.0,
            Self::Mountains => 10.0,
            Self::Badlands => 5.0,
            _ => 0.0,
        }
    }
}

pub struct BiomeMap {
    simplex: Simplex,
    temperature: Simplex,
    moisture: Simplex,
    weirdness: Simplex,
}

impl BiomeMap {
    pub fn new(seed: u32) -> Self {
        Self {
            simplex: Simplex::new(seed + 200),
            temperature: Simplex::new(seed + 300),
            moisture: Simplex::new(seed + 400),
            weirdness: Simplex::new(seed + 500),
        }
    }

    pub fn get_biome(&self, x: i32, z: i32) -> Biome {
        let nx = x as f64 / 200.0;
        let nz = z as f64 / 200.0;

        let temp = self.temperature.get([nx, nz]);
        let moist = self.moisture.get([nx * 1.5, nz * 1.5]);
        let weird = self.weirdness.get([nx * 0.8, nz * 0.8]);

        // Extended biome selection
        if temp < -0.5 {
            Biome::Snow
        } else if temp < -0.3 && moist > 0.2 {
            Biome::Swamp
        } else if temp > 0.4 && moist < -0.3 {
            if weird > 0.2 {
                Biome::Badlands
            } else {
                Biome::Desert
            }
        } else if temp > 0.2 && moist > 0.4 {
            Biome::Jungle
        } else if moist > 0.3 {
            Biome::Forest
        } else if weird < -0.4 && moist > 0.0 {
            Biome::MushroomIsland
        } else {
            let mountain_noise = self.simplex.get([nx * 0.5, nz * 0.5]);
            if mountain_noise > 0.3 {
                Biome::Mountains
            } else {
                Biome::Plains
            }
        }
    }

    pub fn get_height(&self, x: i32, z: i32) -> i32 {
        let biome = self.get_biome(x, z);
        let nx = x as f64 / 80.0;
        let nz = z as f64 / 80.0;

        let h1 = self.simplex.get([nx, nz]) * 15.0;
        let h2 = self.simplex.get([nx * 2.0, nz * 2.0]) * 7.0;
        let h3 = self.simplex.get([nx * 4.0, nz * 4.0]) * 3.0;

        let raw_height = h1 + h2 + h3;
        let scaled = raw_height * biome.height_scale() + biome.base_height();
        let height = (20.0 + scaled) as i32;
        height.clamp(1, 63) as i32
    }
}
