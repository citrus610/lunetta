use criterion::*;
use strum::IntoEnumIterator;
use tetris::{board::Board, movegen::movegen, piece::Piece};

fn bench_movegen(c: &mut Criterion, name: &str, board: Board) {
    let mut group = c.benchmark_group(name);

    for kind in Piece::iter() {
        group.bench_function(format!("{:?}", kind), |b| b.iter(|| movegen(&board, kind)));
    }
}

fn bench(c: &mut Criterion) {
    bench_movegen(c, "empty", Board::new());

    #[rustfmt::skip]
    bench_movegen(c, "mini", Board {
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
    bench_movegen(c, "tspin", Board {
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
    bench_movegen(c, "dtd", Board {
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
    bench_movegen(c, "bad", Board {
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

criterion_group!(benchmark, bench);
criterion_main!(benchmark);
