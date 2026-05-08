#![allow(dead_code)]
use serde::{Deserialize, Serialize};

/// Experience and enchanting system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub level: u32,
    pub points: u32,
    pub total_earned: u32,
}

impl Experience {
    pub fn new() -> Self {
        Self { level: 0, points: 0, total_earned: 0 }
    }

    /// XP needed for next level (Minecraft formula approximation)
    pub fn xp_for_level(level: u32) -> u32 {
        if level <= 16 {
            2 * level + 7
        } else if level <= 31 {
            5 * level - 38
        } else {
            9 * level - 158
        }
    }

    /// Add XP points and level up if enough
    pub fn add(&mut self, amount: u32) {
        self.points += amount;
        self.total_earned += amount;

        while self.points >= Self::xp_for_level(self.level) {
            self.points -= Self::xp_for_level(self.level);
            self.level += 1;
        }
    }

    /// Try to spend XP levels. Returns true if successful.
    pub fn spend(&mut self, levels: u32) -> bool {
        if self.level >= levels {
            self.level -= levels;
            true
        } else {
            false
        }
    }
}

/// Enchantment types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Enchantment {
    Efficiency,   // Faster mining
    Fortune,      // More drops
    Sharpness,    // More damage
    Protection,   // Less damage taken
    Unbreaking,   // More durability
    SilkTouch,    // Mine blocks as-is
}

impl Enchantment {
    pub fn name(&self) -> &str {
        match self {
            Self::Efficiency => "Efficiency",
            Self::Fortune => "Fortune",
            Self::Sharpness => "Sharpness",
            Self::Protection => "Protection",
            Self::Unbreaking => "Unbreaking",
            Self::SilkTouch => "Silk Touch",
        }
    }

    pub fn max_level(&self) -> u32 {
        match self {
            Self::Efficiency | Self::Sharpness => 5,
            Self::Fortune | Self::Protection | Self::Unbreaking => 3,
            Self::SilkTouch => 1,
        }
    }

    pub fn min_level_required(&self) -> u32 {
        match self {
            Self::Efficiency | Self::Sharpness => 1,
            Self::Fortune | Self::Unbreaking => 5,
            Self::Protection => 5,
            Self::SilkTouch => 10,
        }
    }

    /// Generate random enchantments for a given enchanting level
    pub fn generate_for_level(enchant_level: u32, seed: u64) -> Vec<(Enchantment, u32)> {
        let mut result = Vec::new();
        let mut rng = seed;

        let all = [
            Enchantment::Efficiency,
            Enchantment::Fortune,
            Enchantment::Sharpness,
            Enchantment::Protection,
            Enchantment::Unbreaking,
            Enchantment::SilkTouch,
        ];

        for ench in &all {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let roll = (rng >> 33) as u32 % 100;

            if enchant_level >= ench.min_level_required() && roll < 30 {
                let max = ench.max_level().min(enchant_level / 3 + 1);
                rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                let level = (rng >> 33) as u32 % max + 1;
                result.push((*ench, level));
            }
        }

        // At least one enchantment if level is high enough
        if result.is_empty() && enchant_level >= 1 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let idx = (rng >> 33) as usize % all.len();
            result.push((all[idx], 1));
        }

        result
    }
}

/// Enchanted item wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnchantedItem {
    pub item_index: usize, // index in inventory
    pub enchantments: Vec<(Enchantment, u32)>,
}
