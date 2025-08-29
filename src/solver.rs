//! # Solver Module
//!
//! This module implements search algorithms for solving the 8-puzzle.
//! It supports both Depth-First Search (DFS) and Breadth-First Search (BFS)
//! strategies, providing detailed statistics about the search process.
use clap::ValueEnum;

use crate::board::{ALL_DIRECTIONS, Board};
use crate::search_strategies::SearchStrategy;
use crate::stats::Stats;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

/// Search strategy enumeration for the puzzle solver
///
/// Determines the order in which nodes are explored during the search.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ExplorerStrategy {
    /// Depth-First Search: explores as far as possible along each branch before backtracking
    Dfs,
    /// Breadth-First Search: explores all neighbors at the current depth before moving deeper
    #[default]
    Bfs,
}

/// 8-puzzle solver with comprehensive statistics tracking
///
/// The solver uses either DFS or BFS to find a solution path from any given
/// board state to the solved state. It maintains detailed statistics about
/// the search process including nodes explored, frontier size, and timing.
#[derive(Default, Clone)]
pub struct Solver<T>
where
    T: Default + Clone,
{
    /// Parent relationships for reconstructing the solution path
    parents: HashMap<Board, Board>,
    /// Set of already explored board states
    boards_checked: HashSet<Board>,
    boards_to_check: T,
    /// History of frontier sizes throughout the search
    to_check_size: Vec<usize>,
    /// Depth of each board state in the search tree
    depth_by_board: HashMap<Board, usize>,
    /// Total number of successor states generated
    generated_nodes: usize,
    /// Total number of states added to the frontier
    enqueued_nodes: usize,
    /// Number of duplicate states that were pruned
    duplicates_pruned: usize,
    /// Maximum depth reached during the search
    max_depth_reached: usize,
    /// Time taken to solve the puzzle in milliseconds
    solve_duration_ms: u128,
}

impl<T> Solver<T>
where
    T: SearchStrategy<Board> + Default + Clone,
{
    /// Solves the puzzle using the configured search strategy
    ///
    /// # Arguments
    ///
    /// * `board` - The initial board state to solve
    ///
    /// # Returns
    ///
    /// `Some(solved_board)` if a solution is found, `None` if no solution exists
    pub fn solve(&mut self, board: Board) -> Option<Board> {
        self.init_search(board);
        let start = Instant::now();

        while let Some(board) = self.boards_to_check.get_next() {
            self.mark_explored(board);
            self.record_frontier_size();

            if board.is_solved() {
                return self.finish_with_solution(start, board);
            }

            self.expand_neighbors(board);
        }

        self.finish_without_solution(start);
        None
    }

    /// Creates a new solver with the specified search strategy
    ///
    /// # Arguments
    ///
    /// * `strategy` - The search strategy to use (DFS or BFS)
    ///
    /// # Returns
    ///
    /// A new solver instance ready to solve puzzles
    pub fn new(search_strategy: T) -> Solver<T> {
        Self {
            boards_to_check: search_strategy,
            ..Default::default()
        }
    }

    /// Generates comprehensive statistics about the search process
    ///
    /// # Returns
    ///
    /// A `Stats` struct containing detailed metrics about the search performance
    pub fn get_solution_stats(&self) -> Stats {
        let size = self.to_check_size.len().max(1) as f64;
        let sum: usize = self.to_check_size.iter().copied().sum();
        let avg_frontier = sum as f64 / size;
        let max_frontier = self.to_check_size.iter().copied().max().unwrap_or(0);

        let solution_moves = self.step_by_step_solution().len().saturating_sub(1);

        Stats {
            nodes_explored: self.boards_checked.len(),
            solution_moves,
            max_frontier,
            avg_frontier,
            generated_nodes: self.generated_nodes,
            enqueued_nodes: self.enqueued_nodes,
            duplicates_pruned: self.duplicates_pruned,
            max_depth_reached: self.max_depth_reached,
            duration_ms: self.solve_duration_ms,
        }
    }

    /// Reconstructs the solution path from start to goal
    ///
    /// Uses the parent relationships tracked during the search to build
    /// the complete sequence of board states from initial to solved.
    ///
    /// # Returns
    ///
    /// A vector of board states representing the solution path
    pub fn step_by_step_solution(&self) -> Vec<Board> {
        let mut c = Board::default();
        let mut solution = vec![c];

        while let Some(nc) = self.parents.get(&c) {
            solution.push(*nc);
            c = *nc;
        }

        solution.reverse();
        solution
    }

    /// Initializes the search with the starting board state
    ///
    /// # Arguments
    ///
    /// * `start` - The initial board state to begin searching from
    fn init_search(&mut self, start: Board) {
        self.boards_to_check.enqueue(start);
        self.depth_by_board.insert(start, 0);
    }

    /// Records the current frontier size for statistics
    fn record_frontier_size(&mut self) {
        self.to_check_size.push(self.boards_to_check.len());
    }

    /// Marks a board as explored to avoid revisiting it
    ///
    /// # Arguments
    ///
    /// * `board` - The board state to mark as explored
    fn mark_explored(&mut self, board: Board) {
        self.boards_checked.insert(board);
    }

    /// Completes the search when a solution is found
    ///
    /// # Arguments
    ///
    /// * `start` - The time when the search began
    /// * `board` - The solved board state
    ///
    /// # Returns
    ///
    /// The solved board state
    fn finish_with_solution(&mut self, start: Instant, board: Board) -> Option<Board> {
        self.solve_duration_ms = start.elapsed().as_millis();
        Some(board)
    }

    /// Completes the search when no solution is found
    ///
    /// # Arguments
    ///
    /// * `start` - The time when the search began
    fn finish_without_solution(&mut self, start: Instant) {
        self.solve_duration_ms = start.elapsed().as_millis();
    }

    /// Adds a successor board to the frontier with proper bookkeeping
    ///
    /// Updates parent relationships, depth tracking, and statistics.
    ///
    /// # Arguments
    ///
    /// * `parent` - The parent board state
    /// * `child` - The successor board state to enqueue
    fn enqueue_successor(&mut self, parent: Board, child: Board) {
        self.boards_to_check.enqueue(child);
        self.enqueued_nodes += 1;
        self.parents.insert(child, parent);

        let parent_depth = *self.depth_by_board.get(&parent).unwrap_or(&0);
        let depth = parent_depth + 1;
        self.depth_by_board.insert(child, depth);
        if depth > self.max_depth_reached {
            self.max_depth_reached = depth;
        }
    }

    /// Processes a single move attempt from a parent board
    ///
    /// Generates a successor state and either enqueues it or records it as a duplicate.
    ///
    /// # Arguments
    ///
    /// * `parent` - The parent board state
    /// * `dir` - The direction to move the empty space
    fn process_move(&mut self, parent: Board, dir: crate::board::Direction) {
        if let Ok(child) = parent.move_space(dir) {
            self.generated_nodes += 1;
            if !self.boards_checked.contains(&child) {
                self.enqueue_successor(parent, child);
            } else {
                self.duplicates_pruned += 1;
            }
        }
    }

    /// Expands all possible successor states from the current board
    ///
    /// Attempts to move the empty space in all four directions to generate
    /// all valid successor states.
    ///
    /// # Arguments
    ///
    /// * `board` - The current board state to expand
    fn expand_neighbors(&mut self, board: Board) {
        for direction in &ALL_DIRECTIONS {
            self.process_move(board, *direction);
        }
    }
}
