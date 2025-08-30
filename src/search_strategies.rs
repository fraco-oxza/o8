//! Search strategy abstractions and implementations.
//!
//! This module defines a small strategy trait and a couple of concrete queue
//! types to drive the solver:
//!
//! - `SimpleSearchStrategy` implements FIFO (BFS) or LIFO (DFS) behavior using
//!   a `LinkedList`, depending on the configured `ExplorerStrategy`.
//! - `HeuristicSearchStrategy` implements a best-first priority queue using a
//!   `BinaryHeap`, suitable for A*-like expansions when paired with a type that
//!   implements `Ord` based on f(n) = g(n)+h(n). In this project we use
//!   `Reverse<BoardWithSteps>` so that lower cost pops first.
//!
//! The solver is generic over `SearchStrategy<T>`, so new frontier policies can
//! be plugged in easily.
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, LinkedList},
};

use crate::solver::ExplorerStrategy;

/// Minimal frontier abstraction used by the solver.
pub trait SearchStrategy<T> {
    /// Pop the next node to expand according to the policy.
    fn get_next(&mut self) -> Option<T>;
    /// Push a node into the frontier.
    fn enqueue(&mut self, node: T);
    /// Current frontier size.
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

/// A best-first priority queue based on `Ord`.
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
