use std::cmp::Ordering;

use tetris::{
    bag::{Bag, update_bag},
    movegen::movegen,
    moves::Move,
    piece::Piece,
    state::{Lock, State},
};

use crate::{
    eval::{Weights, evaluate},
    layer::Layer,
    node::Node,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BeamReward {
    pub depth: usize,
    pub reward: i32,
}

#[derive(Debug, Clone)]
pub struct BeamSettings {
    pub width: usize,
    pub depth: usize,
    pub branch: usize,
}

#[derive(Debug, Clone)]
pub struct BeamCandidate {
    pub mv: Move,
    pub reward: BeamReward,
}

#[derive(Debug, Clone)]
pub struct BeamResult {
    pub candidates: Vec<BeamCandidate>,
    pub nodes: usize,
    pub depth: usize,
}

fn expand(node: &Node, queue: &Vec<Piece>, mut callback: impl FnMut(Node, Move)) -> usize {
    let mut nodes = 0;

    let current = queue[node.state.next];
    let hold = match node.state.hold {
        Some(kind) => kind,
        None => queue[node.state.next + 1],
    };

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
    parents: &mut Vec<Node>,
    children: &mut Layer,
    queue: &Vec<Piece>,
    candidates: &mut Vec<BeamCandidate>,
    w: &Weights,
    depth: usize,
) -> usize {
    let mut nodes = 0;

    while let Some(parent) = parents.pop() {
        nodes += expand(&parent, queue, |mut child, mv| {
            evaluate(&mut child, mv, w);

            let reward = BeamReward {
                depth: depth,
                reward: child.reward,
            };

            if candidates[child.index].reward < reward {
                candidates[child.index].reward = reward;
            }

            children.push(child);
        });
    }

    while let Some(child) = children.pop_worst() {
        parents.push(child);
    }

    nodes
}

fn is_queue_valid(queue: &Vec<Piece>, mut bag: Bag) -> bool {
    for kind in queue {
        if !bag.contains(*kind) {
            return false;
        }

        update_bag(&mut bag, *kind);
    }

    true
}

pub fn beam_search(
    state: State,
    lock: Lock,
    queue: Vec<Piece>,
    w: Weights,
    settings: BeamSettings,
) -> Option<BeamResult> {
    // Check if queue is valid
    if queue.len() < 2 || !is_queue_valid(&queue, state.bag.clone()) {
        println!("invalid queue!");
        return None;
    }

    // Initialize search variables
    let mut result = BeamResult {
        candidates: Vec::new(),
        nodes: 0,
        depth: 0,
    };

    let root = Node {
        state: State { next: 0, ..state },
        lock: lock,
        value: 0,
        reward: 0,
        index: 0,
    };

    let mut parents = Vec::<Node>::with_capacity(settings.width);
    let mut children = Layer::new(settings.width);

    // Initialize candidates
    result.nodes += expand(&root, &queue, |mut child, mv| {
        child.index = result.candidates.len();

        evaluate(&mut child, mv, &w);

        result.candidates.push(BeamCandidate {
            mv: mv,
            reward: BeamReward {
                depth: 0,
                reward: child.reward,
            },
        });

        parents.push(child);
    });

    parents.sort();

    // If there are no candidates, return none
    if result.candidates.is_empty() {
        println!("no candidate!");
        return None;
    }

    // Search
    result.depth = 1;

    while result.depth < queue.len() - state.hold.is_none() as usize {
        result.nodes += think(
            &mut parents,
            &mut children,
            &queue,
            &mut result.candidates,
            &w,
            result.depth,
        );

        result.depth += 1;
    }

    // Sort candidates
    result.candidates.sort_by(|a, b| b.cmp(a));

    Some(result)
}

impl Ord for BeamReward {
    fn cmp(&self, other: &Self) -> Ordering {
        self.depth
            .cmp(&other.depth)
            .then(self.reward.cmp(&other.reward))
    }
}

impl PartialOrd for BeamReward {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BeamCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.reward.cmp(&other.reward)
    }
}

impl PartialOrd for BeamCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for BeamCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.reward == other.reward
    }
}

impl Eq for BeamCandidate {}
