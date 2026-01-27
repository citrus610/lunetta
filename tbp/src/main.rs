use bot::{
    bot::{BotConfigs, BotState, best_move},
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

    let weights = Weights::default();

    let configs = BotConfigs {
        width: 250,
        depth: 12,
        branch: 1,
    };

    let mut queue = random_queue(1000);

    let mut bot = BotState::new(
        &State {
            board: Board::new(),
            hold: None,
            bag: Bag::all(),
            next: 0,
            b2b: 0,
            combo: 0,
        },
        &Lock {
            cleared: 0,
            sent: 0,
            softdrop: false,
        },
        &queue[..12],
        &weights,
    )
    .expect("bot should be valid smh!");

    queue.drain(..12);

    let mut holded = false;

    for _ in 0..1000 {
        if let Ok(result) = bot.search(configs) {
            let mv = match best_move(&result, 0) {
                Ok(mv) => mv,
                _ => {
                    println!("death!");
                    break;
                }
            };

            let mut nexts = Vec::new();

            if mv.kind == *queue.first().unwrap() && !holded {
                holded = true;
                nexts.push(queue.remove(0));
            }

            nexts.push(queue.remove(0));

            if bot.make(mv, &nexts).is_err() {
                println!("invalid nexts!");
                break;
            }

            println!("{}", bot.root().board);
            println!("nodes: {}", result.nodes);
            println!("depth: {}", result.depth);

            std::thread::sleep(std::time::Duration::from_millis(200));
        } else {
            println!("death!");
            break;
        }
    }
}
