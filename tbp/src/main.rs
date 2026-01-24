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
    let w = Weights {
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
    };

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

        if let Some(result) = beam_search(state.clone(), lock.clone(), queue.clone(), w.clone(), settings.clone()) {
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
        }
        else {
            println!("death!");
            break;
        }
    }
}
