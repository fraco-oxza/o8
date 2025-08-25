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

const DEFAULT_RUNS: usize = 200;
const DEFAULT_SCRAMBLE_STEPS: usize = 200;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = DEFAULT_RUNS)]
    runs: usize,
    #[arg(short, long, default_value_t = DEFAULT_SCRAMBLE_STEPS)]
    scramble_steps: usize,
}

fn run_search(boards: &[Board], algo: ExplorerStrategy) -> Vec<Stats> {
    boards
        .par_iter()
        .map(|b| {
            let mut solver = Solver::new(algo);
            solver.solve(*b).expect("No solution founded");
            solver.get_solution_stats()
        })
        .collect()
}

fn main() {
    let Args {
        runs,
        scramble_steps,
    } = Args::parse();

    println!(
        "Generando {runs} tableros aleatorios con {scramble_steps} movimientos y comparando DFS vs BFS..."
    );

    let boards: Vec<Board> = (0..runs)
        .map(|_| Board::random_with_solution(scramble_steps))
        .collect();

    let dfs_run = run_search(&boards, ExplorerStrategy::Dfs);
    let bfs_run = run_search(&boards, ExplorerStrategy::Bfs);

    print_comparison_table(&dfs_run.as_slice().into(), &bfs_run.as_slice().into());
}
