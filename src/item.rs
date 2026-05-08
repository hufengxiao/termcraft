#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use crate::block::BlockType;
use crate::tool::Tool;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    Block(BlockType),
    Tool(Tool),
    Food(FoodType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FoodType {
    Apple,
    Bread,
    CookedPork,
    GoldenApple,
}

impl FoodType {
    pub fn hunger_restore(self) -> f64 {
        match self {
            Self::Apple => 4.0,
            Self::Bread => 5.0,
            Self::CookedPork => 8.0,
            Self::GoldenApple => 20.0,
        }
    }

    pub fn saturation_restore(self) -> f64 {
        match self {
            Self::Apple => 2.4,
            Self::Bread => 6.0,
            Self::CookedPork => 12.8,
            Self::GoldenApple => 20.0,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Apple => "Apple",
            Self::Bread => "Bread",
            Self::CookedPork => "Pork",
            Self::GoldenApple => "G.Apple",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Item {
    pub item_type: ItemType,
    pub count: u8,
}

impl Item {
    pub fn new(item_type: ItemType, count: u8) -> Self {
        Self { item_type, count }
    }

    pub fn from_block(block: BlockType) -> Self {
        Self::new(ItemType::Block(block), 1)
    }

    pub fn from_tool(tool: Tool) -> Self {
        Self::new(ItemType::Tool(tool), 1)
    }

    pub fn name(&self) -> &str {
        match self.item_type {
            ItemType::Block(b) => match b {
                BlockType::Grass => "Grass",
                BlockType::Dirt => "Dirt",
                BlockType::Stone => "Stone",
                BlockType::Sand => "Sand",
                BlockType::Wood => "Wood",
                BlockType::Leaves => "Leaves",
                BlockType::RedstoneDust => "Redstone",
                BlockType::RedstoneTorch => "RTorch",
                BlockType::Lever => "Lever",
                BlockType::RedstoneLamp => "Lamp",
                BlockType::Obsidian => "Obsidian",
                BlockType::Netherrack => "Netherrack",
                BlockType::CoalOre => "Coal",
                BlockType::IronOre => "Iron",
                BlockType::GoldOre => "Gold",
                BlockType::DiamondOre => "Diamond",
                _ => "Block",
            },
            ItemType::Tool(tool) => match tool.tool_type {
                crate::tool::ToolType::Hand => "Hand",
                crate::tool::ToolType::WoodenPickaxe => "W.Pick",
                crate::tool::ToolType::StonePickaxe => "S.Pick",
                crate::tool::ToolType::IronPickaxe => "I.Pick",
                crate::tool::ToolType::DiamondPickaxe => "D.Pick",
                crate::tool::ToolType::WoodenAxe => "W.Axe",
                crate::tool::ToolType::StoneAxe => "S.Axe",
                crate::tool::ToolType::IronAxe => "I.Axe",
                crate::tool::ToolType::WoodenSword => "W.Sword",
                crate::tool::ToolType::StoneSword => "S.Sword",
                crate::tool::ToolType::IronSword => "I.Sword",
                crate::tool::ToolType::WoodenShovel => "W.Shvl",
                crate::tool::ToolType::StoneShovel => "S.Shvl",
                crate::tool::ToolType::IronShovel => "I.Shvl",
            },
            ItemType::Food(food) => food.name(),
        }
    }

    pub fn max_stack(&self) -> u8 {
        match self.item_type {
            ItemType::Block(_) => 64,
            ItemType::Tool(_) => 1,
            ItemType::Food(_) => 64,
        }
    }

    pub fn from_food(food: FoodType) -> Self {
        Self::new(ItemType::Food(food), 1)
    }

    pub fn is_food(&self) -> bool {
        matches!(self.item_type, ItemType::Food(_))
    }
}

pub const INVENTORY_SIZE: usize = 36; // 9 hotbar + 27 main
pub const CRAFTING_SIZE: usize = 9;   // 3x3 grid

pub struct Inventory {
    pub slots: [Option<Item>; INVENTORY_SIZE],
    pub selected: usize, // hotbar selection (0-8)
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            slots: [None; INVENTORY_SIZE],
            selected: 0,
        }
    }

    pub fn selected_item(&self) -> Option<Item> {
        self.slots[self.selected]
    }

    /// Add item to inventory, returns leftover count
    pub fn add_item(&mut self, item: Item) -> u8 {
        let mut remaining = item.count;

        // First try to stack with existing items
        for slot in &mut self.slots {
            if remaining == 0 { break; }
            if let Some(existing) = slot {
                if existing.item_type == item.item_type {
                    let space = existing.max_stack() - existing.count;
                    let add = remaining.min(space);
                    existing.count += add;
                    remaining -= add;
                }
            }
        }

        // Then try empty slots
        for slot in &mut self.slots {
            if remaining == 0 { break; }
            if slot.is_none() {
                let add = remaining.min(item.max_stack());
                *slot = Some(Item::new(item.item_type, add));
                remaining -= add;
            }
        }

        remaining
    }

    /// Remove one item from selected slot
    pub fn use_selected(&mut self) -> Option<Item> {
        if let Some(ref mut item) = self.slots[self.selected] {
            let result = Item::new(item.item_type, 1);
            item.count -= 1;
            if item.count == 0 {
                self.slots[self.selected] = None;
            }
            Some(result)
        } else {
            None
        }
    }

    /// Get hotbar items for HUD display
    pub fn hotbar_display(&self) -> [Option<(String, u8)>; 9] {
        let mut display: [Option<(String, u8)>; 9] = Default::default();
        for i in 0..9 {
            if let Some(item) = &self.slots[i] {
                display[i] = Some((item.name().to_string(), item.count));
            }
        }
        display
    }
}

/// 3x3 Crafting grid
pub struct CraftingGrid {
    pub grid: [Option<Item>; CRAFTING_SIZE],
}

impl CraftingGrid {
    pub fn new() -> Self {
        Self { grid: [None; CRAFTING_SIZE] }
    }

    /// Check if current grid matches any recipe
    pub fn craft(&self) -> Option<Item> {
        use crate::tool::{Tool, ToolType};

        // Wood -> 4 Planks
        if self.grid[0].map(|i| i.item_type) == Some(ItemType::Block(BlockType::Wood))
            && self.grid.iter().skip(1).all(|s| s.is_none())
        {
            return Some(Item::new(ItemType::Block(BlockType::Wood), 4));
        }

        // 2 Wood -> 4 Sticks
        if self.grid[0].map(|i| i.item_type) == Some(ItemType::Block(BlockType::Wood))
            && self.grid[3].map(|i| i.item_type) == Some(ItemType::Block(BlockType::Wood))
            && self.grid[1..3].iter().all(|s| s.is_none())
            && self.grid[4..9].iter().all(|s| s.is_none())
        {
            return Some(Item::new(ItemType::Block(BlockType::Wood), 4));
        }

        // Wooden Pickaxe: WWW / _W_ / _W_
        if self.grid[0..3].iter().all(|s| s.map(|i| i.item_type) == Some(ItemType::Block(BlockType::Wood)))
            && self.grid[4].map(|i| i.item_type) == Some(ItemType::Block(BlockType::Wood))
            && self.grid[7].map(|i| i.item_type) == Some(ItemType::Block(BlockType::Wood))
            && self.grid[3].is_none() && self.grid[5].is_none() && self.grid[6].is_none() && self.grid[8].is_none()
        {
            return Some(Item::from_tool(Tool::new(ToolType::WoodenPickaxe)));
        }

        // Stone Pickaxe: SSS / _W_ / _W_
        if self.grid[0..3].iter().all(|s| s.map(|i| i.item_type) == Some(ItemType::Block(BlockType::Stone)))
            && self.grid[4].map(|i| i.item_type) == Some(ItemType::Block(BlockType::Wood))
            && self.grid[7].map(|i| i.item_type) == Some(ItemType::Block(BlockType::Wood))
            && self.grid[3].is_none() && self.grid[5].is_none() && self.grid[6].is_none() && self.grid[8].is_none()
        {
            return Some(Item::from_tool(Tool::new(ToolType::StonePickaxe)));
        }

        // Iron Pickaxe: III / _W_ / _W_
        if self.grid[0..3].iter().all(|s| s.map(|i| i.item_type) == Some(ItemType::Block(BlockType::IronOre)))
            && self.grid[4].map(|i| i.item_type) == Some(ItemType::Block(BlockType::Wood))
            && self.grid[7].map(|i| i.item_type) == Some(ItemType::Block(BlockType::Wood))
            && self.grid[3].is_none() && self.grid[5].is_none() && self.grid[6].is_none() && self.grid[8].is_none()
        {
            return Some(Item::from_tool(Tool::new(ToolType::IronPickaxe)));
        }

        // Diamond Pickaxe: DDD / _W_ / _W_
        if self.grid[0..3].iter().all(|s| s.map(|i| i.item_type) == Some(ItemType::Block(BlockType::DiamondOre)))
            && self.grid[4].map(|i| i.item_type) == Some(ItemType::Block(BlockType::Wood))
            && self.grid[7].map(|i| i.item_type) == Some(ItemType::Block(BlockType::Wood))
            && self.grid[3].is_none() && self.grid[5].is_none() && self.grid[6].is_none() && self.grid[8].is_none()
        {
            return Some(Item::from_tool(Tool::new(ToolType::DiamondPickaxe)));
        }

        // Wooden Sword: _W_ / _W_ / _W_
        if self.grid[1].map(|i| i.item_type) == Some(ItemType::Block(BlockType::Wood))
            && self.grid[4].map(|i| i.item_type) == Some(ItemType::Block(BlockType::Wood))
            && self.grid[7].map(|i| i.item_type) == Some(ItemType::Block(BlockType::Wood))
            && self.grid[0].is_none() && self.grid[2].is_none() && self.grid[3].is_none()
            && self.grid[5].is_none() && self.grid[6].is_none() && self.grid[8].is_none()
        {
            return Some(Item::from_tool(Tool::new(ToolType::WoodenSword)));
        }

        // Stone Sword: _S_ / _S_ / _W_
        if self.grid[1].map(|i| i.item_type) == Some(ItemType::Block(BlockType::Stone))
            && self.grid[4].map(|i| i.item_type) == Some(ItemType::Block(BlockType::Stone))
            && self.grid[7].map(|i| i.item_type) == Some(ItemType::Block(BlockType::Wood))
            && self.grid[0].is_none() && self.grid[2].is_none() && self.grid[3].is_none()
            && self.grid[5].is_none() && self.grid[6].is_none() && self.grid[8].is_none()
        {
            return Some(Item::from_tool(Tool::new(ToolType::StoneSword)));
        }

        // Furnace: SSS / S_S / SSS
        if self.grid.iter().enumerate().all(|(i, s)| {
            if i == 4 { s.is_none() } else { s.map(|item| item.item_type) == Some(ItemType::Block(BlockType::Stone)) }
        })
        {
            return Some(Item::new(ItemType::Block(BlockType::Stone), 8));
        }

        // Torch: Coal + Wood -> 4 Torches
        if self.grid[0].map(|i| i.item_type) == Some(ItemType::Block(BlockType::CoalOre))
            && self.grid[3].map(|i| i.item_type) == Some(ItemType::Block(BlockType::Wood))
            && self.grid[1].is_none() && self.grid[2].is_none()
            && self.grid[4..9].iter().all(|s| s.is_none())
        {
            return Some(Item::new(ItemType::Block(BlockType::RedstoneTorch), 4));
        }

        // Smelting: IronOre + Coal -> Iron (simplified as block)
        if self.grid[0].map(|i| i.item_type) == Some(ItemType::Block(BlockType::IronOre))
            && self.grid[1].map(|i| i.item_type) == Some(ItemType::Block(BlockType::CoalOre))
            && self.grid[2..9].iter().all(|s| s.is_none())
        {
            return Some(Item::new(ItemType::Block(BlockType::IronOre), 1)); // iron ingot
        }

        // Smelting: GoldOre + Coal -> Gold
        if self.grid[0].map(|i| i.item_type) == Some(ItemType::Block(BlockType::GoldOre))
            && self.grid[1].map(|i| i.item_type) == Some(ItemType::Block(BlockType::CoalOre))
            && self.grid[2..9].iter().all(|s| s.is_none())
        {
            return Some(Item::new(ItemType::Block(BlockType::GoldOre), 1)); // gold ingot
        }

        // Bread: 3 Wheat (simplified as 3 Grass)
        if self.grid[0..3].iter().all(|s| s.map(|i| i.item_type) == Some(ItemType::Block(BlockType::Grass)))
            && self.grid[3..9].iter().all(|s| s.is_none())
        {
            return Some(Item::from_food(FoodType::Bread));
        }

        None
    }
}
