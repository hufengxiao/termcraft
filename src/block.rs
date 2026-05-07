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
    CaveAir, // for cave generation distinction
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
        }
    }

    pub fn is_solid(self) -> bool {
        self != Self::Air && self != Self::Water && self != Self::CaveAir
            && self != Self::Flower && self != Self::TallGrass
    }

    pub fn all_buildable() -> &'static [BlockType] {
        &[
            Self::Grass,
            Self::Dirt,
            Self::Stone,
            Self::Sand,
            Self::Wood,
        ]
    }
}
