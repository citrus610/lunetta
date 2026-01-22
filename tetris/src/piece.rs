use enumset::*;
use strum::*;

#[derive(Debug, PartialOrd, Ord, EnumIter, FromRepr, EnumSetType)]
#[enumset(repr = "u8")]
pub enum Piece {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumIter, FromRepr)]
pub enum Rotation {
    North,
    East,
    South,
    West,
}

macro_rules! relative_to {
    ($rotation:expr, [$(($x:expr, $y:expr)),*]) => {
        match $rotation {
            Rotation::North => [$(($x, $y)),*],
            Rotation::East => [$(($y, -$x)),*],
            Rotation::South => [$((-$x, -$y)),*],
            Rotation::West => [$((-$y, $x)),*],
        }
    }
}

impl Piece {
    pub const fn cells(self, rotation: Rotation) -> [(i8, i8); 4] {
        match self {
            Piece::I => relative_to!(rotation, [(-1, 0), (0, 0), (1, 0), (2, 0)]),
            Piece::J => relative_to!(rotation, [(-1, 1), (-1, 0), (0, 0), (1, 0)]),
            Piece::L => relative_to!(rotation, [(-1, 0), (0, 0), (1, 1), (1, 0)]),
            Piece::O => relative_to!(rotation, [(0, 1), (0, 0), (1, 1), (1, 0)]),
            Piece::S => relative_to!(rotation, [(-1, 0), (0, 1), (0, 0), (1, 1)]),
            Piece::T => relative_to!(rotation, [(-1, 0), (0, 1), (0, 0), (1, 0)]),
            Piece::Z => relative_to!(rotation, [(-1, 1), (0, 1), (0, 0), (1, 0)]),
        }
    }
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::I => write!(f, "I"),
            Self::J => write!(f, "J"),
            Self::L => write!(f, "L"),
            Self::O => write!(f, "O"),
            Self::S => write!(f, "S"),
            Self::T => write!(f, "T"),
            Self::Z => write!(f, "Z"),
        }
    }
}

impl Rotation {
    pub const fn cw(&self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }

    pub const fn ccw(&self) -> Self {
        self.cw().cw().cw()
    }
}

impl std::fmt::Display for Rotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::North => write!(f, "North"),
            Self::East => write!(f, "East"),
            Self::South => write!(f, "South"),
            Self::West => write!(f, "West"),
        }
    }
}
