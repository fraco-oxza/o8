//! # O8 - 8-Puzzle Solver
//!
//! A high-performance 8-puzzle solver that compares the effectiveness of different search strategies
//! (Depth-First Search vs Breadth-First Search) using parallel processing for performance analysis.
//!
//! The 8-puzzle is a sliding puzzle consisting of a 3x3 grid with 8 numbered tiles and one empty space.
//! The goal is to arrange the tiles in numerical order by sliding them into the empty space.

use clap::Parser;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    board::Board,
    solver::{ExplorerStrategy, Solver},
    stats::{Stats, print_comparison_table},
};

pub(crate) mod board;
pub(crate) mod solver;
pub(crate) mod stats;

/// Default number of test runs to perform
const DEFAULT_RUNS: usize = 200;

/// Default number of scramble steps to generate random boards
const DEFAULT_SCRAMBLE_STEPS: usize = 200;

/// Command-line arguments for the 8-puzzle solver
#[derive(Parser, Debug)]
struct Args {
    /// Number of test runs to perform for each algorithm
    #[arg(short, long, default_value_t = DEFAULT_RUNS)]
    runs: usize,

    /// Number of scramble steps to generate random puzzle boards
    #[arg(short, long, default_value_t = DEFAULT_SCRAMBLE_STEPS)]
    scramble_steps: usize,
}

/// Runs a search algorithm on a collection of boards in parallel
///
/// # Arguments
///
/// * `boards` - A slice of puzzle boards to solve
/// * `algo` - The search strategy to use (DFS or BFS)
///
/// # Returns
///
/// A vector of statistics for each solved board
fn run_search(boards: &[Board], algo: ExplorerStrategy) -> Vec<Stats> {
    boards
        .par_iter()
        .map(|b| {
            let mut solver = Solver::new(algo);
            solver.solve(*b).expect("No solution found");
            solver.get_solution_stats()
        })
        .collect()
}

/// Main function that orchestrates the 8-puzzle solver comparison
///
/// Generates random puzzle boards, solves them using both DFS and BFS algorithms,
/// and displays a comparison table of the performance metrics.
fn main() {
    let Args {
        runs,
        scramble_steps,
    } = Args::parse();

    println!(
        "Generating {runs} random boards with {scramble_steps} moves and comparing DFS vs BFS..."
    );

    let boards: Vec<Board> = (0..runs)
        .map(|_| Board::random_with_solution(scramble_steps))
        .collect();

    let dfs_run = run_search(&boards, ExplorerStrategy::Dfs);
    let bfs_run = run_search(&boards, ExplorerStrategy::Bfs);

    print_comparison_table(&dfs_run.as_slice().into(), &bfs_run.as_slice().into());
}
