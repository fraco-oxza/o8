use crate::board::{ALL_DIRECTIONS, Board};
use std::collections::{HashMap, HashSet, LinkedList};

#[derive(Default)]
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
}

impl Solver {
    pub fn new(strategy: ExplorerStrategy) -> Solver {
        Self {
            strategy,
            boards_to_check: LinkedList::new(),
            boards_checked: HashSet::new(),
            parents: HashMap::new(),
            to_check_size: Vec::new(),
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

    pub fn nodes_explored(&self) -> usize {
        self.boards_checked.len()
    }

    pub fn get_next_board(&mut self) -> Option<Board> {
        match self.strategy {
            ExplorerStrategy::Bfs => self.boards_to_check.pop_front(),
            ExplorerStrategy::Dfs => self.boards_to_check.pop_back(),
        }
    }

    pub fn solve(&mut self, board: Board) -> Option<Board> {
        self.boards_to_check.push_back(board);

        while let Some(board) = self.get_next_board() {
            self.boards_checked.insert(board);

            if board.is_solved() {
                return Some(board);
            }

            for direction in &ALL_DIRECTIONS {
                let _ = board.move_space(*direction).map(|b| {
                    if !self.boards_checked.contains(&b) {
                        self.boards_to_check.push_back(b);
                        self.parents.insert(b, board);
                    }
                });
            }
        }

        None
    }
}
