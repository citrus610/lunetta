use strum::IntoEnumIterator;

use tetris::{board::Board, movegen::*, piece::Piece};

fn print_movegen(name: &str, board: Board) {
    println!("Board: {}", name);
    println!("{}", board);

    for kind in Piece::iter() {
        let moves = movegen(&board, kind);

        println!("{}: {}", kind, moves.len());
    }
}

fn main() {
    print_movegen("empty", Board::new());

    #[rustfmt::skip]
    print_movegen("mini", Board {
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
    });

    #[rustfmt::skip]
    print_movegen("tspin", Board {
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
    });

    #[rustfmt::skip]
    print_movegen("dtd", Board {
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
    });

    #[rustfmt::skip]
    print_movegen("bad", Board {
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
    });
}
