use std::{
    cmp::Reverse,
    collections::{BinaryHeap, LinkedList},
};

use crate::solver::ExplorerStrategy;

pub trait SearchStrategy<T> {
    fn get_next(&mut self) -> Option<T>;
    fn enqueue(&mut self, node: T);
    fn len(&self) -> usize;
}

#[derive(Default, Clone)]
pub struct SimpleSearchStrategy<T> {
    nodes: LinkedList<T>,
    strategy: ExplorerStrategy,
}

impl<T> SimpleSearchStrategy<T> {
    pub fn new(algorithm: ExplorerStrategy) -> Self {
        Self {
            nodes: LinkedList::default(),
            strategy: algorithm,
        }
    }
}

impl<T> SearchStrategy<T> for SimpleSearchStrategy<T> {
    fn get_next(&mut self) -> Option<T> {
        match self.strategy {
            ExplorerStrategy::Bfs => self.nodes.pop_front(),
            ExplorerStrategy::Dfs => self.nodes.pop_back(),
        }
    }

    fn enqueue(&mut self, node: T) {
        self.nodes.push_back(node);
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }
}

#[derive(Default, Clone)]
pub struct HeuristicSearchStrategy<T: Ord + PartialOrd>(BinaryHeap<T>);

impl<T: Ord + PartialOrd> SearchStrategy<T> for HeuristicSearchStrategy<Reverse<T>> {
    fn get_next(&mut self) -> Option<T> {
        self.0.pop().map(|b| b.0)
    }

    fn enqueue(&mut self, node: T) {
        self.0.push(Reverse(node));
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}
