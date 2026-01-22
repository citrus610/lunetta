use std::u64;

use strum::IntoEnumIterator;

use crate::{
    board::Board,
    piece::{Piece, Rotation},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tspin {
    Mini,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub x: i8,
    pub y: i8,
    pub r: Rotation,
    pub kind: Piece,
    pub tspin: Option<Tspin>,
}

#[derive(Debug, Clone, Copy)]
pub struct MoveMap {
    pub data: [Board; 4],
}

impl Move {
    pub fn cells(&self) -> [(i8, i8); 4] {
        self.kind
            .cells(self.r)
            .map(|(x, y)| (self.x + x, self.y + y))
    }

    pub fn is_colliding(&self, board: &Board) -> bool {
        self.cells().iter().any(|&(x, y)| board.has(x, y))
    }

    pub fn is_underground(&self, board: &Board) -> bool {
        self.cells()
            .iter()
            .any(|&(x, y)| y < board.height(x as usize) as i8)
    }

    pub fn shifted(&self, collisions: &MoveMap, dx: i8) -> Option<Self> {
        let shifted = Self {
            x: self.x + dx,
            tspin: None,
            ..*self
        };

        match collisions.has(&shifted) {
            true => None,
            false => Some(shifted),
        }
    }

    pub fn rotated(&self, collisions: &MoveMap, board: &Board, target: Rotation) -> Option<Self> {
        let offsets = match self.kind {
            Piece::I => &I_SRS_TABLE,
            Piece::O => &O_SRS_TABLE,
            _ => &JLSTZ_SRS_TABLE,
        };

        for i in 0..5 {
            let dx = offsets[self.r as usize][i].0 - offsets[target as usize][i].0;
            let dy = offsets[self.r as usize][i].1 - offsets[target as usize][i].1;

            let mut rotated = Self {
                x: self.x + dx,
                y: self.y + dy,
                r: target,
                ..*self
            };

            if !collisions.has(&rotated) {
                rotated.tspin = if self.kind == Piece::T && rotated.has_tspin_corners(board) {
                    if i == 4 || rotated.has_front_corners(board) {
                        Some(Tspin::Full)
                    } else {
                        Some(Tspin::Mini)
                    }
                } else {
                    None
                };

                return Some(rotated);
            }
        }

        None
    }

    pub fn dropped(&self, collisions: &MoveMap) -> Self {
        let col = collisions.data[self.r as usize].cols[self.x as usize];

        Self {
            y: 64 - (col & ((1 << self.y) - 1)).leading_zeros() as i8,
            tspin: None,
            ..*self
        }
    }

    pub fn canonicalized(&self) -> Self {
        match self.kind {
            Piece::I => match self.r {
                Rotation::South => Self {
                    x: self.x - 1,
                    r: Rotation::North,
                    ..*self
                },
                Rotation::West => Self {
                    y: self.y + 1,
                    r: Rotation::East,
                    ..*self
                },
                _ => *self,
            },
            Piece::S => match self.r {
                Rotation::South => Self {
                    y: self.y - 1,
                    r: Rotation::North,
                    ..*self
                },
                Rotation::West => Self {
                    x: self.x - 1,
                    r: Rotation::East,
                    ..*self
                },
                _ => *self,
            },
            Piece::Z => match self.r {
                Rotation::South => Self {
                    y: self.y - 1,
                    r: Rotation::North,
                    ..*self
                },
                Rotation::West => Self {
                    x: self.x - 1,
                    r: Rotation::East,
                    ..*self
                },
                _ => *self,
            },
            _ => *self,
        }
    }

    fn has_tspin_corners(&self, board: &Board) -> bool {
        let corners = board.has(self.x + 1, self.y + 1) as u32
            + board.has(self.x + 1, self.y - 1) as u32
            + board.has(self.x - 1, self.y + 1) as u32
            + board.has(self.x - 1, self.y - 1) as u32;

        corners >= 3
    }

    fn has_front_corners(&self, board: &Board) -> bool {
        match self.r {
            Rotation::North => {
                board.has(self.x + 1, self.y + 1) && board.has(self.x - 1, self.y + 1)
            }
            Rotation::East => {
                board.has(self.x + 1, self.y + 1) && board.has(self.x + 1, self.y - 1)
            }
            Rotation::South => {
                board.has(self.x + 1, self.y - 1) && board.has(self.x - 1, self.y - 1)
            }
            Rotation::West => {
                board.has(self.x - 1, self.y + 1) && board.has(self.x - 1, self.y - 1)
            }
        }
    }
}

impl MoveMap {
    pub const fn new() -> Self {
        Self {
            data: [Board::new(); 4],
        }
    }

    pub fn collsions(board: &Board, kind: Piece) -> Self {
        let mut collisions = Self::new();

        for r in Rotation::iter() {
            for (dx, dy) in kind.cells(r) {
                for x in 0..10 {
                    let mut col = u64::MAX;

                    if matches!(x + dx, 0..10) {
                        col = board.cols[(x + dx) as usize];

                        col = match dy < 0 {
                            true => !(!col << -dy),
                            false => col >> dy,
                        };
                    }

                    collisions.data[r as usize].cols[x as usize] |= col
                }
            }
        }

        collisions
    }

    pub fn filled_sky(collisions: &MoveMap) -> Self {
        let mut filled = Self::new();

        for r in 0..4 {
            for x in 0..10 {
                let height = collisions.data[r].height(x);

                filled.data[r].cols[x] = match height {
                    0..64 => !((1 << height) - 1),
                    _ => 0,
                };
            }
        }

        filled
    }

    pub fn has(&self, mv: &Move) -> bool {
        self.data[mv.r as usize].has(mv.x, mv.y)
    }

    pub fn set(&mut self, mv: &Move) {
        self.data[mv.r as usize].set(mv.x, mv.y);
    }

    pub fn clear(&mut self, mv: &Move) {
        self.data[mv.r as usize].clear(mv.x, mv.y);
    }

    pub fn has_bit(&self, x: i8, y: i8, r: Rotation) -> bool {
        self.data[r as usize].has(x, y)
    }

    pub fn set_bit(&mut self, x: i8, y: i8, r: Rotation) {
        self.data[r as usize].set(x, y);
    }

    pub fn clear_bit(&mut self, x: i8, y: i8, r: Rotation) {
        self.data[r as usize].clear(x, y);
    }
}

impl std::fmt::Display for Tspin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mini => write!(f, "Mini"),
            Self::Full => write!(f, "Full"),
        }
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "x: {}", self.x)?;
        writeln!(f, "y: {}", self.y)?;
        writeln!(f, "r: {}", self.r)?;
        writeln!(f, "kind: {}", self.kind)?;

        match self.tspin {
            Some(tspin) => writeln!(f, "tspin: {}", tspin)?,
            None => writeln!(f, "tspin: None")?,
        }

        Ok(())
    }
}

const JLSTZ_SRS_TABLE: [[(i8, i8); 5]; 4] = [
    [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
    [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
];

const I_SRS_TABLE: [[(i8, i8); 5]; 4] = [
    [(0, 0), (-1, 0), (2, 0), (-1, 0), (2, 0)],
    [(-1, 0), (0, 0), (0, 0), (0, 1), (0, -2)],
    [(-1, 1), (1, 1), (-2, 1), (1, 0), (-2, 0)],
    [(0, 1), (0, 1), (0, 1), (0, -1), (0, 2)],
];

const O_SRS_TABLE: [[(i8, i8); 5]; 4] = [
    [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
    [(0, -1), (0, 0), (0, 0), (0, 0), (0, 0)],
    [(-1, -1), (0, 0), (0, 0), (0, 0), (0, 0)],
    [(-1, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
];
