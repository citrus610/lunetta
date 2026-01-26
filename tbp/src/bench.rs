use std::time::Instant;

use bot::{
    beam::{BeamSettings, beam_search},
    eval::Weights,
};
use tetris::{
    bag::Bag,
    board::Board,
    piece::Piece,
    state::{Lock, State},
};

pub fn bench() {
    let boards = [
        Board::new(),
        Board {
            cols: [
                0b000000111111,
                0b000000111111,
                0b000000011111,
                0b000000000111,
                0b000000000001,
                0b000000000000,
                0b000000001101,
                0b000000011111,
                0b000000111111,
                0b000011111111,
            ],
        },
        Board {
            cols: [
                0b000111111111,
                0b000111111111,
                0b000011111111,
                0b000011111111,
                0b000000111111,
                0b000000100110,
                0b000010000001,
                0b000011110111,
                0b000011111111,
                0b000011111111,
            ],
        },
        Board {
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
            ],
        },
    ];

    let mut nodes = 0;
    let mut times = 0;

    for board in boards {
        let queue = vec![
            Piece::I,
            Piece::O,
            Piece::L,
            Piece::J,
            Piece::S,
            Piece::Z,
            Piece::T,
            Piece::I,
            Piece::O,
            Piece::L,
            Piece::J,
            Piece::S,
        ];

        let start = Instant::now();

        nodes += beam_search(
            State {
                board: board,
                hold: None,
                bag: Bag::all(),
                next: 0,
                b2b: 0,
                combo: 0,
            },
            Lock {
                cleared: 0,
                sent: 0,
                softdrop: false,
            },
            queue,
            Weights::default(),
            BeamSettings {
                width: 250,
                depth: 12,
                branch: 1,
            },
        )
        .expect("bot dead?")
        .nodes;

        let elasped = start.elapsed();

        times += elasped.as_millis();
    }

    let knps = nodes as u128 / times;

    println!("nodes: {} - nps: {} kn/s", nodes, knps);
}
