use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

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
        if let Some(entry) = self.map.get_mut(&node.state) {
            if node.reward < *entry {
                return;
            }

            *entry = node.reward;
        } else {
            self.map.insert(node.state.clone(), node.reward);
        }

        // If our layer is smaller than beam width, push normally
        if self.heap.len() < self.size {
            self.heap.push(Reverse(node));
            return;
        }

        // Only push if node is better than our's worst
        let Reverse(worst) = self.heap.peek().unwrap();

        if *worst < node {
            self.heap.pop();
            self.heap.push(Reverse(node));
        }
    }

    pub fn pop_worst(&mut self) -> Option<Node> {
        self.heap.pop().map(|node| node.0)
    }
}
