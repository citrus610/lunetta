use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::collections::hash_map::Entry;

use tetris::state::State;

use crate::node::Node;

#[derive(Debug, Clone)]
pub struct Layer {
    map: HashMap<State, i32>,
    heap: BinaryHeap<Reverse<Node>>,
    size: usize,
}

impl Layer {
    pub fn new(size: usize) -> Self {
        Self {
            map: HashMap::with_capacity(1 << 12),
            heap: BinaryHeap::with_capacity(size),
            size: size,
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.heap.clear();
    }

    pub fn push(&mut self, node: Node) {
        // Check transposition table
        match self.map.entry(node.state.clone()) {
            Entry::Occupied(mut entry) => {
                if node.reward <= *entry.get() {
                    return;
                }
                entry.insert(node.reward);
            }
            Entry::Vacant(entry) => {
                entry.insert(node.reward);
            }
        }

        // If our layer is smaller than beam width, push normally
        if self.heap.len() < self.size {
            self.heap.push(Reverse(node));
            return;
        }

        // Only push if node is better than our's worst
        if self.heap.peek().expect("has worst").0 < node {
            self.heap.pop();
            self.heap.push(Reverse(node));
        }
    }

    pub fn pop_worst(&mut self) -> Option<Node> {
        self.heap.pop().map(|node| node.0)
    }
}
