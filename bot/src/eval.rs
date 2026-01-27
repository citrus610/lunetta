use std::u64;

use tetris::{
    board::Board,
    moves::{Move, Tspin},
    piece::{Piece, Rotation},
};

use crate::node::Node;

#[derive(Debug, Clone, Copy)]
pub struct Weights {
    pub height: i32,
    pub well: i32,
    pub center: i32,
    pub bumpiness: i32,
    pub holes: i32,
    pub garbage: i32,
    pub tslot: [i32; 4],
    pub b2b_bonus: i32,
    pub combo_bonus: i32,

    pub clear: [i32; 4],
    pub tspin: [i32; 3],
    pub tspin_mini: [i32; 2],
    pub combo: [i32; 5],
    pub b2b: i32,
    pub pc: i32,
    pub waste_t: i32,
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            height: -50,
            well: 25,
            center: -100,
            bumpiness: -25,
            holes: -400,
            garbage: -300,
            tslot: [150, 200, 250, 500],
            b2b_bonus: 200,
            combo_bonus: 200,

            clear: [-400, -350, -300, 250],
            tspin: [50, 400, 800],
            tspin_mini: [0, 0],
            combo: [200, 500, 1000, 1500, 2000],
            b2b: 100,
            pc: 2000,
            waste_t: -100,
        }
    }
}

// Return the well's depth and the position of the well
fn well(board: &Board, heights: &[u32; 10]) -> (i32, usize) {
    let mut x = 0;

    for i in 1..10 {
        if heights[i] < heights[x] {
            x = i;
        }
    }

    let mut mask = u64::MAX;

    for i in 0..10 {
        if i == x {
            continue;
        }

        mask &= board.cols[i];
    }

    mask >>= heights[x];

    (mask.count_ones() as i32, x)
}

fn bumpiness(heights: &[u32; 10], well_x: usize) -> i32 {
    let mut bumpiness = 0;
    let mut left = 0;

    if well_x == 0 {
        left = 1;
    }

    for i in 1..10 {
        if i == well_x {
            continue;
        }

        let diff = heights[left].abs_diff(heights[i]);

        bumpiness += diff * diff;
        left = i;
    }

    bumpiness as i32
}

// Get the number of holes overground and underground
fn holes(board: &Board, heights: &[u32; 10], well_x: usize) -> (i32, i32) {
    let min_height = heights[well_x];

    let mut holes = 0;

    for i in 0..10 {
        holes += heights[i] - min_height - (board.cols[i] >> min_height).count_ones();
    }

    (holes as i32, min_height as i32)
}

// Find the highest tslot
fn tslot(board: &Board, heights: &[u32; 10]) -> Option<Move> {
    for x in 0..8 {
        if heights[x] > heights[x + 1] && heights[x] + 1 < heights[x + 2] {
            if ((board.cols[x] >> (heights[x] - 1)) & 0b111) == 0b001
                && ((board.cols[x + 1] >> (heights[x] - 1)) & 0b111) == 0b000
                && ((board.cols[x + 2] >> (heights[x] - 1)) & 0b111) == 0b101
            {
                return Some(Move {
                    x: x as i8 + 1,
                    y: heights[x] as i8,
                    r: Rotation::South,
                    kind: Piece::T,
                    tspin: None,
                });
            }
        }

        if heights[x + 2] > heights[x + 1] && heights[x + 2] + 1 < heights[x] {
            if ((board.cols[x] >> (heights[x + 2] - 1)) & 0b111) == 0b101
                && ((board.cols[x + 1] >> (heights[x + 2] - 1)) & 0b111) == 0b000
                && ((board.cols[x + 2] >> (heights[x + 2] - 1)) & 0b111) == 0b001
            {
                return Some(Move {
                    x: x as i8 + 1,
                    y: heights[x + 2] as i8,
                    r: Rotation::South,
                    kind: Piece::T,
                    tspin: None,
                });
            }
        }

        if heights[x + 1] >= 3
            && heights[x + 1] >= heights[x]
            && heights[x + 1] + 1 < heights[x + 2]
        {
            if ((board.cols[x] >> (heights[x + 1] - 3)) & 0b11000) == 0b00000
                && ((board.cols[x + 1] >> (heights[x + 1] - 3)) & 0b11110) == 0b00100
                && ((board.cols[x + 2] >> (heights[x + 1] - 3)) & 0b11111) == 0b10000
                && (board.has(x as i8 + 1, heights[x + 1] as i8 - 3)
                    || (!board.has(x as i8 + 1, heights[x + 1] as i8 - 3)
                        && board.has(x as i8 + 2, heights[x + 1] as i8 - 4)))
            {
                return Some(Move {
                    x: x as i8 + 2,
                    y: heights[x + 1] as i8 - 2,
                    r: Rotation::West,
                    kind: Piece::T,
                    tspin: None,
                });
            }
        }

        if heights[x + 1] >= 3
            && heights[x + 1] >= heights[x + 2]
            && heights[x + 1] + 1 < heights[x]
        {
            if ((board.cols[x] >> (heights[x + 1] - 3)) & 0b11111) == 0b10000
                && ((board.cols[x + 1] >> (heights[x + 1] - 3)) & 0b11110) == 0b00100
                && ((board.cols[x + 2] >> (heights[x + 1] - 3)) & 0b11000) == 0b00000
                && (board.has(x as i8 + 1, heights[x + 1] as i8 - 3)
                    || (!board.has(x as i8 + 1, heights[x + 1] as i8 - 3)
                        && board.has(x as i8, heights[x + 1] as i8 - 4)))
            {
                return Some(Move {
                    x: x as i8,
                    y: heights[x + 1] as i8 - 2,
                    r: Rotation::East,
                    kind: Piece::T,
                    tspin: None,
                });
            }
        }
    }

    None
}

fn donations(board: &mut Board, heights: &mut [u32; 10], depth: usize) -> ([i32; 4], i32) {
    let mut tslots = [0; 4];
    let mut donations = 0;

    for _ in 0..depth {
        if let Some(tslot) = tslot(board, heights) {
            let mut clone = board.clone();

            clone.place(&tslot);

            let clear = clone.clear_lines();

            tslots[clear as usize] += 1;

            if clear >= 2 {
                *board = clone;
                *heights = board.heights();

                donations += 1;
            } else {
                break;
            }
        }
    }

    (tslots, donations)
}

pub fn evaluate(node: &mut Node, mv: Move, w: &Weights) {
    node.value = 0;

    let mut board = node.state.board.clone();
    let mut heights = board.heights();

    // Height
    let height = *heights.iter().max().unwrap() as i32;

    node.value += height * w.height;

    // Tslots
    let (tslots, donations) = donations(&mut board, &mut heights, 2);

    for i in 0..4 {
        node.value += tslots[i] * w.tslot[i];
    }

    // Well
    let (well, well_x) = well(&board, &heights);

    node.value += well.min(4) * w.well;

    // Center
    node.value += well_x.abs_diff(4).min(well_x.abs_diff(5)) as i32 * w.center;

    // Bumpiness
    let bumpiness = bumpiness(&heights, well_x);

    node.value += bumpiness * w.bumpiness;

    // Holes
    let (mut holes, garbage) = holes(&board, &heights, well_x);

    holes -= tslots[0] + tslots[1] + tslots[2] + tslots[3] - donations;

    node.value += holes * w.holes;
    node.value += garbage * w.garbage;

    // Bonus
    if node.state.b2b > 0 {
        node.value += w.b2b_bonus;
    }

    if node.state.combo > 1 {
        node.value += (node.state.combo as i32 - 1) * w.combo_bonus;
    }

    // Pc
    let pc = board.is_empty();

    if pc {
        node.reward += w.pc;
    }

    // Line clear
    if node.lock.cleared > 0 {
        if pc {
            node.reward += w.pc;
        } else if let Some(tspin) = mv.tspin {
            node.reward += match tspin {
                Tspin::Full => w.tspin[node.lock.cleared as usize - 1],
                Tspin::Mini => w.tspin_mini[node.lock.cleared as usize - 1],
            };
        } else {
            node.reward += w.clear[node.lock.cleared as usize - 1];
        }
    }

    // Back to back
    if node.state.b2b > 1 {
        node.reward += w.b2b;
    }

    // Combo
    node.reward += match node.state.combo {
        0..2 => 0,
        2..4 => w.combo[0],
        4..6 => w.combo[1],
        6..8 => w.combo[2],
        8..10 => w.combo[3],
        _ => w.combo[4],
    };

    // Waste T
    if mv.kind == Piece::T && !(mv.tspin.is_some() && node.lock.cleared > 0) && !pc {
        node.reward += w.waste_t;
    }
}
