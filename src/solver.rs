use crate::board::{ALL_DIRECTIONS, Board};
use crate::stats::Stats;
use std::collections::{HashMap, HashSet, LinkedList};
use std::time::Instant;

#[derive(Clone, Copy, Debug, Default)]
pub enum ExplorerStrategy {
    #[default]
    Dfs,
    Bfs,
}

#[derive(Default)]
pub struct Solver {
    strategy: ExplorerStrategy,
    boards_to_check: LinkedList<Board>,
    parents: HashMap<Board, Board>,
    boards_checked: HashSet<Board>,
    to_check_size: Vec<usize>,
    depth_by_board: HashMap<Board, usize>,
    generated_nodes: usize,
    enqueued_nodes: usize,
    duplicates_pruned: usize,
    max_depth_reached: usize,
    solve_duration_ms: f64,
}

impl Solver {
    pub fn new(strategy: ExplorerStrategy) -> Solver {
        Self {
            strategy,
            boards_to_check: LinkedList::new(),
            boards_checked: HashSet::new(),
            parents: HashMap::new(),
            to_check_size: Vec::new(),
            depth_by_board: HashMap::new(),
            generated_nodes: 0,
            enqueued_nodes: 0,
            duplicates_pruned: 0,
            max_depth_reached: 0,
            solve_duration_ms: 0.0,
        }
    }

    pub fn get_solution_stats(&self) -> Stats {
        let size = self.to_check_size.len().max(1) as f64;
        let sum: usize = self.to_check_size.iter().copied().sum();
        let avg_frontier = sum as f64 / size;
        let max_frontier = self.to_check_size.iter().copied().max().unwrap_or(0);

        let solution_moves = self.step_by_step_solution().len().saturating_sub(1);

        Stats {
            strategy: self.strategy,
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

    pub fn get_next_board(&mut self) -> Option<Board> {
        match self.strategy {
            ExplorerStrategy::Bfs => self.boards_to_check.pop_front(),
            ExplorerStrategy::Dfs => self.boards_to_check.pop_back(),
        }
    }

    fn init_search(&mut self, start: Board) {
        self.boards_to_check.push_back(start);
        self.depth_by_board.insert(start, 0);
    }

    fn record_frontier_size(&mut self) {
        self.to_check_size.push(self.boards_to_check.len());
    }

    fn mark_explored(&mut self, board: Board) {
        self.boards_checked.insert(board);
    }

    fn finish_with_solution(&mut self, start: Instant, board: Board) -> Option<Board> {
        self.solve_duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        Some(board)
    }

    fn finish_without_solution(&mut self, start: Instant) {
        self.solve_duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    }

    fn enqueue_successor(&mut self, parent: Board, child: Board) {
        self.boards_to_check.push_back(child);
        self.enqueued_nodes += 1;
        self.parents.insert(child, parent);

        let parent_depth = *self.depth_by_board.get(&parent).unwrap_or(&0);
        let depth = parent_depth + 1;
        self.depth_by_board.insert(child, depth);
        if depth > self.max_depth_reached {
            self.max_depth_reached = depth;
        }
    }

    fn process_move(&mut self, parent: Board, dir: crate::board::Direction) {
        if let Ok(child) = parent.move_space(dir) {
            self.generated_nodes += 1; // valid successor generated
            if !self.boards_checked.contains(&child) {
                self.enqueue_successor(parent, child);
            } else {
                self.duplicates_pruned += 1;
            }
        }
    }

    fn expand_neighbors(&mut self, board: Board) {
        for direction in &ALL_DIRECTIONS {
            self.process_move(board, *direction);
        }
    }

    pub fn solve(&mut self, board: Board) -> Option<Board> {
        self.init_search(board);
        let start = Instant::now();

        while let Some(board) = self.get_next_board() {
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
}
