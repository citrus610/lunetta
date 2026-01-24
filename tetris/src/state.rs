use crate::{
    bag::{Bag, update_bag},
    board::Board,
    moves::{Move, Tspin},
    piece::Piece,
};

pub fn combo_bonus(index: usize) -> u8 {
    const TABLE: [u8; 13] = [0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 4, 5];

    TABLE[index.min(TABLE.len() - 1)]
}

#[derive(Debug, Clone, Copy)]
pub struct Lock {
    pub cleared: u8,
    pub sent: u8,
    pub softdrop: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State {
    pub board: Board,
    pub hold: Option<Piece>,
    pub bag: Bag,
    pub next: usize,
    pub b2b: u8,
    pub combo: u8,
}

impl State {
    pub const fn new() -> Self {
        Self {
            board: Board::new(),
            hold: None,
            bag: Bag::all(),
            next: 0,
            b2b: 0,
            combo: 0,
        }
    }

    pub fn make(&mut self, mv: &Move, queue: &[Piece]) -> Lock {
        let mut current = queue[self.next];

        if mv.kind != current {
            let was_hold_empty = self.hold.is_none();

            self.hold = Some(current);

            if was_hold_empty {
                update_bag(&mut self.bag, current);

                self.next += 1;

                current = queue[self.next];
            }
        }

        update_bag(&mut self.bag, current);

        self.next += 1;

        let mut lock = Lock {
            cleared: 0,
            sent: 0,
            softdrop: mv.is_underground(&self.board),
        };

        self.board.place(mv);

        lock.cleared = self.board.clear_lines();

        if lock.cleared > 0 {
            if let Some(tspin) = mv.tspin {
                lock.sent = match tspin {
                    Tspin::Full => lock.cleared * 2,
                    Tspin::Mini => lock.cleared - 1,
                };

                self.b2b += 1;
            } else if lock.cleared == 4 {
                lock.sent = 4;

                self.b2b += 1;
            } else {
                lock.sent = lock.cleared - 1;

                self.b2b = 0;
            }

            self.b2b = self.b2b.min(2);

            if self.b2b > 1 {
                lock.sent += 1;
            }

            self.combo += 1;

            lock.sent += combo_bonus(self.combo as usize);

            if self.board.is_empty() {
                lock.sent += 10;
            }
        } else {
            self.combo = 0;
        }

        lock
    }
}
