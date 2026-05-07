use crossterm::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockType {
    Air,
    Grass,
    Dirt,
    Stone,
    Sand,
    Water,
    Wood,
    Leaves,
    Flower,
    TallGrass,
    CaveAir,
    // Redstone
    RedstoneDust,
    RedstoneTorch,
    Lever,
    RedstoneLamp,
    // Nether
    Netherrack,
    NetherBrick,
    Obsidian,
    Portal,
    Lava,
}

impl BlockType {
    pub fn color(self) -> Option<Color> {
        match self {
            Self::Air | Self::CaveAir => None,
            Self::Grass => Some(Color::Green),
            Self::Dirt => Some(Color::DarkYellow),
            Self::Stone => Some(Color::Grey),
            Self::Sand => Some(Color::Yellow),
            Self::Water => Some(Color::Blue),
            Self::Wood => Some(Color::DarkRed),
            Self::Leaves => Some(Color::DarkGreen),
            Self::Flower => Some(Color::Magenta),
            Self::TallGrass => Some(Color::DarkGreen),
            Self::RedstoneDust => Some(Color::Red),
            Self::RedstoneTorch => Some(Color::Yellow),
            Self::Lever => Some(Color::Grey),
            Self::RedstoneLamp => Some(Color::Yellow),
            Self::Netherrack => Some(Color::DarkRed),
            Self::NetherBrick => Some(Color::Red),
            Self::Obsidian => Some(Color::DarkMagenta),
            Self::Portal => Some(Color::Magenta),
            Self::Lava => Some(Color::Red),
        }
    }

    pub fn glyph(self) -> Option<char> {
        match self {
            Self::Air | Self::CaveAir => None,
            Self::Grass => Some('░'),
            Self::Dirt => Some('▒'),
            Self::Stone => Some('▓'),
            Self::Sand => Some('░'),
            Self::Water => Some('≈'),
            Self::Wood => Some('║'),
            Self::Leaves => Some('♣'),
            Self::Flower => Some('✿'),
            Self::TallGrass => Some('╿'),
            Self::RedstoneDust => Some('·'),
            Self::RedstoneTorch => Some('i'),
            Self::Lever => Some('↑'),
            Self::RedstoneLamp => Some('□'),
            Self::Netherrack => Some('▒'),
            Self::NetherBrick => Some('▓'),
            Self::Obsidian => Some('█'),
            Self::Portal => Some('◎'),
            Self::Lava => Some('~'),
        }
    }

    pub fn is_solid(self) -> bool {
        self != Self::Air && self != Self::Water && self != Self::CaveAir
            && self != Self::Flower && self != Self::TallGrass
    }

    #[allow(dead_code)]
    pub fn is_redstone(self) -> bool {
        matches!(self, Self::RedstoneDust | Self::RedstoneTorch | Self::Lever | Self::RedstoneLamp)
    }

    #[allow(dead_code)]
    pub fn all_buildable() -> &'static [BlockType] {
        &[
            Self::Grass, Self::Dirt, Self::Stone, Self::Sand, Self::Wood,
            Self::RedstoneDust, Self::RedstoneTorch, Self::Lever, Self::RedstoneLamp,
        ]
    }
}
