#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use crate::block::BlockType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolType {
    Hand,
    WoodenPickaxe,
    StonePickaxe,
    IronPickaxe,
    DiamondPickaxe,
    WoodenAxe,
    StoneAxe,
    IronAxe,
    WoodenSword,
    StoneSword,
    IronSword,
    WoodenShovel,
    StoneShovel,
    IronShovel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolMaterial {
    Hand,
    Wood,
    Stone,
    Iron,
    Diamond,
}

impl ToolMaterial {
    pub fn durability(self) -> u32 {
        match self {
            Self::Hand => 0,
            Self::Wood => 60,
            Self::Stone => 132,
            Self::Iron => 251,
            Self::Diamond => 1562,
        }
    }

    pub fn mining_speed(self) -> f64 {
        match self {
            Self::Hand => 1.0,
            Self::Wood => 2.0,
            Self::Stone => 4.0,
            Self::Iron => 6.0,
            Self::Diamond => 8.0,
        }
    }

    pub fn attack_damage(self) -> f64 {
        match self {
            Self::Hand => 1.0,
            Self::Wood => 3.0,
            Self::Stone => 4.0,
            Self::Iron => 5.0,
            Self::Diamond => 7.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tool {
    pub tool_type: ToolType,
    pub durability: u32,
    pub max_durability: u32,
}

impl Tool {
    pub fn new(tool_type: ToolType) -> Self {
        let mat = Self::material(tool_type);
        Self {
            tool_type,
            durability: mat.durability(),
            max_durability: mat.durability(),
        }
    }

    pub fn material(tool_type: ToolType) -> ToolMaterial {
        match tool_type {
            ToolType::Hand => ToolMaterial::Hand,
            ToolType::WoodenPickaxe | ToolType::WoodenAxe | ToolType::WoodenSword | ToolType::WoodenShovel => ToolMaterial::Wood,
            ToolType::StonePickaxe | ToolType::StoneAxe | ToolType::StoneSword | ToolType::StoneShovel => ToolMaterial::Stone,
            ToolType::IronPickaxe | ToolType::IronAxe | ToolType::IronSword | ToolType::IronShovel => ToolMaterial::Iron,
            ToolType::DiamondPickaxe => ToolMaterial::Diamond,
        }
    }

    pub fn is_pickaxe(self) -> bool {
        matches!(self.tool_type,
            ToolType::WoodenPickaxe | ToolType::StonePickaxe |
            ToolType::IronPickaxe | ToolType::DiamondPickaxe
        )
    }

    pub fn is_axe(self) -> bool {
        matches!(self.tool_type, ToolType::WoodenAxe | ToolType::StoneAxe | ToolType::IronAxe)
    }

    pub fn is_sword(self) -> bool {
        matches!(self.tool_type, ToolType::WoodenSword | ToolType::StoneSword | ToolType::IronSword)
    }

    pub fn is_shovel(self) -> bool {
        matches!(self.tool_type, ToolType::WoodenShovel | ToolType::StoneShovel | ToolType::IronShovel)
    }

    /// Mining speed multiplier for a given block
    pub fn mining_speed_for(self, block: BlockType) -> f64 {
        let base = Self::material(self.tool_type).mining_speed();
        let correct_tool = match block {
            BlockType::Stone | BlockType::Netherrack | BlockType::Obsidian => self.is_pickaxe(),
            BlockType::Wood | BlockType::Leaves => self.is_axe(),
            BlockType::Dirt | BlockType::Sand | BlockType::Grass => self.is_shovel(),
            _ => true,
        };
        if correct_tool { base } else { 1.0 }
    }

    /// Use the tool (reduce durability). Returns true if broken.
    pub fn use_tool(&mut self) -> bool {
        if self.durability > 0 {
            self.durability -= 1;
            self.durability == 0
        } else {
            true
        }
    }
}
