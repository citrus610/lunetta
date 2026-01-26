use bot::{
    beam::{BeamSettings, beam_search},
    eval::Weights,
};
use rand::{rng, seq::SliceRandom};
use tetris::{
    bag::Bag,
    board::Board,
    piece::Piece,
    state::{Lock, State},
};

use crate::bench::bench;

mod bench;

fn random_queue(bag: usize) -> Vec<Piece> {
    let mut queue = Vec::new();

    for _ in 0..bag {
        let mut full = vec![
            Piece::I,
            Piece::O,
            Piece::L,
            Piece::J,
            Piece::S,
            Piece::Z,
            Piece::T,
        ];

        let mut rng = rng();

        full.shuffle(&mut rng);

        queue.append(&mut full);
    }

    queue
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.len() {
        2 => match &args[1][..] {
            "bench" => {
                bench();
                return;
            }
            _ => {}
        },
        _ => {}
    }

    let w = Weights::default();

    let settings = BeamSettings {
        width: 250,
        depth: 7,
        branch: 1,
    };

    let mut state = State {
        board: Board::new(),
        hold: None,
        bag: Bag::all(),
        next: 0,
        b2b: 0,
        combo: 0,
    };

    let mut lock = Lock {
        cleared: 0,
        sent: 0,
        softdrop: false,
    };

    let queue_full = random_queue(1000);

    let mut holded = false;

    for i in 0..1000 {
        let mut queue = Vec::new();

        for p in 0..12 {
            queue.push(queue_full[i + p + holded as usize]);
        }

        if let Some(result) = beam_search(
            state.clone(),
            lock.clone(),
            &queue,
            &w,
            &settings,
        ) {
            let mv = result.candidates.first().unwrap().mv;

            if *queue.first().unwrap() != mv.kind {
                holded = true;
            }

            lock = state.make(&mv, &queue);

            println!("{}", state.board);
            println!("nodes: {}", result.nodes);
            println!("depth: {}", result.depth);

            state.next = 0;

            std::thread::sleep(std::time::Duration::from_millis(500));
        } else {
            println!("death!");
            break;
        }
    }
}
