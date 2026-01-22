use strum::IntoEnumIterator;

use crate::{
    board::Board,
    moves::{Move, MoveMap},
    piece::{Piece, Rotation},
};

fn is_convex(board: &Board, collisions: &MoveMap) -> bool {
    let shift = board.heights().into_iter().min().unwrap();

    for r in 0..4 {
        for x in 0..10 {
            let col = collisions.data[r].cols[x] >> shift;

            if col & col.wrapping_add(1) != 0 {
                return false;
            }
        }
    }

    true
}

fn lock_height(kind: Piece, r: Rotation) -> i8 {
    const TABLE: [[i8; 4]; 7] = [
        [19, 21, 19, 20],
        [19, 20, 20, 20],
        [19, 20, 20, 20],
        [19, 20, 20, 19],
        [19, 20, 20, 20],
        [19, 20, 20, 20],
        [19, 20, 20, 20],
    ];

    TABLE[kind as usize][r as usize]
}

fn lock(mv: &Move, locked: &mut MoveMap, list: &mut Vec<Move>) {
    if mv.y <= lock_height(mv.kind, mv.r) {
        let lock = mv.canonicalized();

        if !locked.has(&lock) {
            locked.set(&lock);
            list.push(lock);
        }
    }
}

fn expand(
    mv: &Move,
    collisions: &MoveMap,
    board: &Board,
    visited: &mut MoveMap,
    locked: &mut MoveMap,
    tspin_locked: &mut MoveMap,
    list: &mut Vec<Move>,
) {
    let drop = mv.dropped(collisions);

    if drop.y != mv.y || mv.tspin.is_none() {
        lock(&drop, locked, list);
    }

    if drop.y != mv.y && !visited.has(&drop) {
        visited.set(&drop);

        expand(
            &drop,
            collisions,
            board,
            visited,
            locked,
            tspin_locked,
            list,
        );
    }

    if let Some(right) = mv.shifted(collisions, 1) {
        if !visited.has(&right) {
            visited.set(&right);

            expand(
                &right,
                collisions,
                board,
                visited,
                locked,
                tspin_locked,
                list,
            );
        }
    }

    if let Some(left) = mv.shifted(collisions, -1) {
        if !visited.has(&left) {
            visited.set(&left);

            expand(
                &left,
                collisions,
                board,
                visited,
                locked,
                tspin_locked,
                list,
            );
        }
    }

    if mv.kind == Piece::O {
        return;
    }

    if let Some(cw) = mv.rotated(collisions, board, mv.r.cw()) {
        if cw.tspin.is_some() && collisions.has_bit(cw.x, cw.y - 1, cw.r) {
            lock(&cw, tspin_locked, list);
        }

        if !visited.has(&cw) {
            visited.set(&cw);

            expand(&cw, collisions, board, visited, locked, tspin_locked, list);
        }
    }

    if let Some(ccw) = mv.rotated(collisions, board, mv.r.ccw()) {
        if ccw.tspin.is_some() && collisions.has_bit(ccw.x, ccw.y - 1, ccw.r) {
            lock(&ccw, tspin_locked, list);
        }

        if !visited.has(&ccw) {
            visited.set(&ccw);

            expand(&ccw, collisions, board, visited, locked, tspin_locked, list);
        }
    }
}

pub fn movegen(board: &Board, kind: Piece) -> Vec<Move> {
    let mut list = Vec::new();

    let collisions = MoveMap::collsions(board, kind);

    let mut visited = MoveMap::new();
    let mut locked = MoveMap::new();
    let mut tspin_locked = MoveMap::new();

    let is_low = board.heights().iter().all(|&h| h <= 16);

    if is_low && kind != Piece::T && is_convex(board, &collisions) {
        for r in Rotation::iter() {
            for x in 0..10 {
                if collisions.has_bit(x, 20, r) {
                    continue;
                }

                list.push(Move {
                    x: x,
                    y: collisions.data[r as usize].height(x as usize) as i8,
                    r: r,
                    kind: kind,
                    tspin: None,
                });
            }

            if kind == Piece::O {
                break;
            }

            if r > Rotation::North && matches!(kind, Piece::I | Piece::S | Piece::Z) {
                break;
            }
        }

        return list;
    }

    if is_low {
        visited = MoveMap::filled_sky(&collisions);

        for r in Rotation::iter() {
            for x in 0..10 {
                if collisions.has_bit(x, 20, r) {
                    continue;
                }

                let dropped = Move {
                    x: x,
                    y: collisions.data[r as usize].height(x as usize) as i8,
                    r: r,
                    kind: kind,
                    tspin: None,
                };

                expand(
                    &dropped,
                    &collisions,
                    board,
                    &mut visited,
                    &mut locked,
                    &mut tspin_locked,
                    &mut list,
                );
            }

            if kind == Piece::O {
                break;
            }
        }

        return list;
    }

    let init = Move {
        x: 4,
        y: 20,
        r: Rotation::North,
        kind: kind,
        tspin: None,
    };

    if collisions.has(&init) {
        return list;
    }

    expand(
        &init,
        &collisions,
        board,
        &mut visited,
        &mut locked,
        &mut tspin_locked,
        &mut list,
    );

    list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rustfmt::skip]
    #[test]
    fn empty() {
        let board = Board::new();
        let tests = [
            (Piece::I, 17),
            (Piece::J, 34),
            (Piece::L, 34),
            (Piece::O, 9),
            (Piece::S, 17),
            (Piece::T, 34),
            (Piece::Z, 17),
        ];

        for test in tests {
            assert_eq!(movegen(&board, test.0).len(), test.1);
        }
    }

    #[rustfmt::skip]
    #[test]
    fn mini() {
        let board = Board {
            cols: [
                0b00000000,
                0b00000001,
                0b00000001,
                0b00000001,
                0b00000001,
                0b00000001,
                0b00000001,
                0b00000001,
                0b00000001,
                0b00000001,
            ]
        };
        let tests = [
            (Piece::I, 17),
            (Piece::J, 34),
            (Piece::L, 34),
            (Piece::O, 9),
            (Piece::S, 17),
            (Piece::T, 35),
            (Piece::Z, 17),
        ];

        for test in tests {
            assert_eq!(movegen(&board, test.0).len(), test.1);
        }
    }

    #[rustfmt::skip]
    #[test]
    fn tspin() {
        let board = Board {
            cols: [
                0b00111111,
                0b00111111,
                0b00011111,
                0b00000111,
                0b00000001,
                0b00000000,
                0b00001101,
                0b00011111,
                0b00111111,
                0b11111111,
            ]
        };
        let tests = [
            (Piece::I, 17),
            (Piece::J, 35),
            (Piece::L, 35),
            (Piece::O, 9),
            (Piece::S, 17),
            (Piece::T, 38),
            (Piece::Z, 18),
        ];

        for test in tests {
            assert_eq!(movegen(&board, test.0).len(), test.1);
        }
    }

    #[rustfmt::skip]
    #[test]
    fn dtd() {
        let board = Board {
            cols: [
                0b111111111,
                0b111111111,
                0b011111111,
                0b011111111,
                0b000111111,
                0b000100110,
                0b010000001,
                0b011110111,
                0b011111111,
                0b011111111,
            ]
        };
        let tests = [
            (Piece::I, 17),
            (Piece::J, 37),
            (Piece::L, 35),
            (Piece::O, 9),
            (Piece::S, 17),
            (Piece::T, 40),
            (Piece::Z, 18),
        ];

        for test in tests {
            assert_eq!(movegen(&board, test.0).len(), test.1);
        }
    }

    #[rustfmt::skip]
    #[test]
    fn bad() {
        let board = Board {
            cols: [
                0b000011111111,
                0b000011000000,
                0b110011000000,
                0b110011001100,
                0b110011001100,
                0b110011001100,
                0b110011001100,
                0b110000001100,
                0b110000001100,
                0b111111111100,
            ]
        };
        let tests = [
            (Piece::I, 38),
            (Piece::J, 80),
            (Piece::L, 81),
            (Piece::O, 29),
            (Piece::S, 42),
            (Piece::T, 83),
            (Piece::Z, 41),
        ];

        for test in tests {
            assert_eq!(movegen(&board, test.0).len(), test.1);
        }
    }
}
