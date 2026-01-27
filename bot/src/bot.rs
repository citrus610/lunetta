use std::cmp::Ordering;
use thiserror::Error;

use tetris::{
    bag::{Bag, update_bag},
    board::Board,
    movegen::movegen,
    moves::Move,
    piece::Piece,
    state::{Lock, State},
};

use crate::{
    eval::{Weights, evaluate},
    node::Node,
    selector::Selector,
};

#[derive(Debug, Error)]
pub enum BotError {
    #[error("invalid queue")]
    InvalidQueue,
    #[error("bot dead")]
    Death,
}

#[derive(Debug, Clone)]
pub struct BotState {
    state: State,
    lock: Lock,
    queue: Vec<Piece>,
    weights: Weights,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BotScore {
    pub depth: usize,
    pub score: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct BotConfigs {
    pub width: usize,
    pub depth: usize,
    pub branch: usize,
}

#[derive(Debug, Clone)]
pub struct BotResult {
    pub candidates: Vec<(Move, BotScore)>,
    pub nodes: usize,
    pub depth: usize,
    root: State,
    queue: Vec<Piece>,
}

impl BotState {
    pub fn new(
        state: &State,
        lock: &Lock,
        queue: &[Piece],
        weights: &Weights,
    ) -> Result<Self, BotError> {
        if queue.len() < 2 || !is_queue_valid(&queue, state.bag) {
            return Err(BotError::InvalidQueue);
        }

        Ok(Self {
            state: state.clone(),
            lock: *lock,
            queue: queue.into(),
            weights: *weights,
        })
    }

    pub fn make(&mut self, mv: Move, nexts: &[Piece]) -> Result<(), BotError> {
        let mut bag = self.state.bag;

        for kind in &self.queue {
            update_bag(&mut bag, *kind);
        }

        if !is_queue_valid(nexts, bag) {
            return Err(BotError::InvalidQueue);
        }

        self.lock = self.state.make(&mv, &self.queue);
        self.queue.extend(nexts);
        self.queue.drain(..self.state.next);
        self.state.next = 0;

        Ok(())
    }

    pub fn reset(&mut self, board: &Board, b2b: u8, combo: u8) -> Result<(), BotError> {
        self.state.board = *board;
        self.state.b2b = b2b;
        self.state.combo = combo;
        self.state.next = 0;

        Ok(())
    }

    pub fn search(&self, configs: BotConfigs) -> Result<BotResult, BotError> {
        let mut result = BotResult {
            candidates: Vec::new(),
            nodes: 0,
            depth: 0,
            root: self.state.clone(),
            queue: self.queue.clone(),
        };
        let mut parents = Vec::with_capacity(configs.width);
        let mut children = Selector::new(configs.width);
        let root = Node {
            state: State {
                next: 0,
                ..self.state
            },
            lock: self.lock,
            value: 0,
            reward: 0,
            index: 0,
        };

        result.nodes = expand(&root, &self.queue, |mut child, mv| {
            child.index = result.candidates.len();

            evaluate(&mut child, mv, &self.weights);

            result.candidates.push((
                mv,
                BotScore {
                    depth: 0,
                    score: child.reward as i64 + child.value as i64,
                },
            ));

            parents.push(child);
        });

        parents.sort();

        if result.candidates.is_empty() {
            return Err(BotError::Death);
        }

        result.depth = 1;
        while result.depth < self.queue.len() - self.state.hold.is_none() as usize {
            result.nodes += think(
                &mut parents,
                &mut children,
                &self.queue,
                &mut result.candidates,
                &self.weights,
                result.depth,
            );
            result.depth += 1;
        }

        Ok(result)
    }

    pub fn root(&self) -> &State {
        &self.state
    }
}

pub fn best_move(result: &BotResult, incomming: i32) -> Result<Move, BotError> {
    let best = result
        .candidates
        .iter()
        .filter(|candidate| {
            let mut root = result.root.clone();
            let lock = root.make(&candidate.0, &result.queue);
            let heights = *root.board.heights()[3..7].iter().max().unwrap();

            heights as i32 + incomming - lock.sent as i32 <= 20
        })
        .max_by_key(|c| c.1)
        .ok_or(BotError::Death)?;

    Ok(best.0)
}

fn expand(node: &Node, queue: &[Piece], mut callback: impl FnMut(Node, Move)) -> usize {
    let mut nodes = 0;

    let current = queue[node.state.next];
    let hold = node
        .state
        .hold
        .unwrap_or_else(|| queue[node.state.next + 1]);

    for kind in [current, hold] {
        let moves = movegen(&node.state.board, kind);
        nodes += moves.len();

        for mv in moves {
            let mut child = node.clone();
            child.lock = child.state.make(&mv, queue);
            callback(child, mv);
        }

        if current == hold {
            break;
        }
    }

    nodes
}

fn think(
    beam: &mut Vec<Node>,
    selector: &mut Selector,
    queue: &[Piece],
    candidates: &mut Vec<(Move, BotScore)>,
    weights: &Weights,
    depth: usize,
) -> usize {
    let mut nodes = 0;

    while let Some(parent) = beam.pop() {
        nodes += expand(&parent, &queue, |mut child, mv| {
            evaluate(&mut child, mv, &weights);

            let score = BotScore {
                depth: depth,
                score: child.reward as i64 + child.value as i64,
            };
            if candidates[child.index].1 < score {
                candidates[child.index].1 = score;
            }

            selector.push(child);
        });
    }

    while let Some(child) = selector.pop_worst() {
        beam.push(child);
    }
    selector.clear();

    nodes
}

fn is_queue_valid(queue: &[Piece], mut bag: Bag) -> bool {
    for &kind in queue {
        if !update_bag(&mut bag, kind) {
            return false;
        }
    }
    true
}

impl Ord for BotScore {
    fn cmp(&self, other: &Self) -> Ordering {
        self.depth
            .cmp(&other.depth)
            .then(self.score.cmp(&other.score))
    }
}

impl PartialOrd for BotScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}