use std::cmp::Ordering;
use tetris::state::{Lock, State};

#[derive(Debug, Clone)]
pub struct Node {
    pub state: State,
    pub lock: Lock,
    pub value: i32,
    pub reward: i32,
    pub index: usize,
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.value + self.reward).cmp(&(other.value + other.reward))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.value + self.reward == other.value + other.reward
    }
}

impl Eq for Node {}
