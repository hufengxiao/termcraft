use crossterm::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Air,
    Grass,
    Dirt,
    Stone,
    Sand,
    Water,
    Wood,
    Leaves,
}

impl BlockType {
    pub fn color(self) -> Option<Color> {
        match self {
            Self::Air => None,
            Self::Grass => Some(Color::Green),
            Self::Dirt => Some(Color::DarkYellow),
            Self::Stone => Some(Color::Grey),
            Self::Sand => Some(Color::Yellow),
            Self::Water => Some(Color::Blue),
            Self::Wood => Some(Color::DarkRed),
            Self::Leaves => Some(Color::DarkGreen),
        }
    }

    pub fn glyph(self) -> Option<char> {
        match self {
            Self::Air => None,
            Self::Grass => Some('░'),
            Self::Dirt => Some('▒'),
            Self::Stone => Some('▓'),
            Self::Sand => Some('░'),
            Self::Water => Some('≈'),
            Self::Wood => Some('║'),
            Self::Leaves => Some('♣'),
        }
    }

    pub fn is_solid(self) -> bool {
        self != Self::Air && self != Self::Water
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
