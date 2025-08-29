//! # O8 - 8-Puzzle Solver
//!
//! A high-performance 8-puzzle solver that compares the effectiveness of different search strategies
//! (Depth-First Search vs Breadth-First Search) using parallel processing for performance analysis.
//!
//! The 8-puzzle is a sliding puzzle consisting of a 3x3 grid with 8 numbered tiles and one empty space.
//! The goal is to arrange the tiles in numerical order by sliding them into the empty space.

use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::search_strategies::HeuristicSearchStrategy;
use crate::search_strategies::SearchStrategy;
use crate::search_strategies::SimpleSearchStrategy;
use crate::{
    board::Board,
    solver::{ExplorerStrategy, Solver},
    stats::{Stats, print_comparison_table},
};

pub(crate) mod board;
pub(crate) mod search_strategies;
pub(crate) mod solver;
pub(crate) mod stats;

/// Default number of test runs to perform
const DEFAULT_RUNS: usize = 200;

/// Default number of scramble steps to generate random boards
const DEFAULT_SCRAMBLE_STEPS: usize = 200;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum SolveAlgorithm {
    /// Depth-First Search: explores as far as possible along each branch before backtracking
    Dfs,
    /// Breadth-First Search: explores all neighbors at the current depth before moving deeper
    Bfs,
    #[default]
    Heuristic,
}

/// Command-line arguments for the 8-puzzle solver
#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Runs multiple search and compares the results
    Benchmark {
        /// Number of test runs to perform for each algorithm
        #[arg(short, long, default_value_t = DEFAULT_RUNS)]
        runs: usize,
        /// Number of scramble steps to generate random puzzle boards
        #[arg(short, long, default_value_t = DEFAULT_SCRAMBLE_STEPS)]
        scramble_steps: usize,
    },
    /// Runs one random board, solved it and show the path
    SolveRandom {
        #[arg(short, long, value_enum)]
        algorithm: Option<SolveAlgorithm>,
        /// Number of scramble steps to generate random puzzle boards
        #[arg(short, long, default_value_t = DEFAULT_SCRAMBLE_STEPS)]
        scramble_steps: usize,
    },
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
fn run_search<T>(boards: &[Board], solver: Solver<T>) -> Vec<Stats>
where
    T: SearchStrategy<board::Board> + Default + Send + Sync + Clone,
{
    boards
        .par_iter()
        .map(|b| {
            let mut solver = solver.clone();
            solver.solve(*b).expect("No solution found");
            solver.get_solution_stats()
        })
        .collect()
}

/// Benchmarks the performance of DFS vs BFS on random puzzle boards
fn benchmark(runs: usize, scramble_steps: usize) {
    println!(
        "Generating {runs} random boards with {scramble_steps} moves and comparing DFS vs BFS..."
    );

    let boards: Vec<Board> = (0..runs)
        .map(|_| Board::random_with_solution(scramble_steps))
        .collect();

    let dfs_run = run_search(
        &boards,
        Solver::new(SimpleSearchStrategy::new(ExplorerStrategy::Dfs)),
    );
    let bfs_run = run_search(
        &boards,
        Solver::new(SimpleSearchStrategy::new(ExplorerStrategy::Bfs)),
    );
    let etc = run_search(&boards, Solver::new(HeuristicSearchStrategy::default()));

    print_comparison_table(
        &dfs_run.as_slice().into(),
        &bfs_run.as_slice().into(),
        &etc.as_slice().into(),
    );
}

fn solve_one<T>(board: Board, mut solver: Solver<T>)
where
    T: SearchStrategy<Board> + Clone + Default,
{
    solver.solve(board).expect("Not solution founded");
    let solution = solver.step_by_step_solution();

    for step in &solution {
        println!("{}", "-".repeat(20));
        println!("{step}");
        println!("{}", "-".repeat(20));
    }

    println!("{:#?}", solver.get_solution_stats());
}

/// Solves a single random puzzle board and displays the solution steps
fn solve_random(scramble_steps: usize, algo: SolveAlgorithm) {
    let board = Board::random_with_solution(scramble_steps);

    match algo {
        SolveAlgorithm::Dfs => solve_one(
            board,
            Solver::new(SimpleSearchStrategy::new(ExplorerStrategy::Dfs)),
        ),
        SolveAlgorithm::Bfs => solve_one(
            board,
            Solver::new(SimpleSearchStrategy::new(ExplorerStrategy::Bfs)),
        ),
        SolveAlgorithm::Heuristic => {
            solve_one(board, Solver::new(HeuristicSearchStrategy::default()))
        }
    };
}

/// Main function that orchestrates the 8-puzzle solver comparison
///
/// Generates random puzzle boards, solves them using both DFS and BFS algorithms,
/// and displays a comparison table of the performance metrics.
fn main() {
    let Args { command } = Args::parse();

    match command {
        Commands::Benchmark {
            runs,
            scramble_steps,
        } => benchmark(runs, scramble_steps),
        Commands::SolveRandom {
            algorithm,
            scramble_steps,
        } => solve_random(scramble_steps, algorithm.unwrap_or_default()),
    }
}
