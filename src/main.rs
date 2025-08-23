use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    board::Board,
    solver::{ExplorerStrategy, Solver, Stats},
};

pub(crate) mod board;
pub(crate) mod solver;

fn run_and_get_stats(strategy: ExplorerStrategy) -> Stats {
    let b = Board::random_with_solution(1_000_000);
    let mut solver = Solver::new(strategy);
    solver.solve(b).expect("No solution founded");

    solver.get_solution_stats()
}

fn main() {
    let n = 100_000;

    println!("Running {n} times DFS and BFS...");
    let dfs_run: Vec<Stats> = (0..n)
        .into_par_iter()
        .map(|_| run_and_get_stats(ExplorerStrategy::Dfs))
        .collect();

    let bfs_run: Vec<Stats> = (0..n)
        .into_par_iter()
        .map(|_| run_and_get_stats(ExplorerStrategy::Bfs))
        .collect();

    // Get average of runs and show it
    // DFS
    let dfs_average = Stats {
        nodes_explored: dfs_run.iter().map(|s| s.nodes_explored).sum::<usize>() / n,
        solution_moves: dfs_run.iter().map(|s| s.solution_moves).sum::<usize>() / n,
        max_list_to_explore_size: dfs_run
            .iter()
            .map(|s| s.max_list_to_explore_size)
            .sum::<usize>()
            / n,
        average_list_to_explore_size: dfs_run
            .iter()
            .map(|s| s.average_list_to_explore_size)
            .sum::<u128>()
            / n as u128,
    };
    println!("DFS average over {n} runs: {dfs_average:#?}");

    // BFS
    let bfs_average = Stats {
        nodes_explored: bfs_run.iter().map(|s| s.nodes_explored).sum::<usize>() / n,
        solution_moves: bfs_run.iter().map(|s| s.solution_moves).sum::<usize>() / n,
        max_list_to_explore_size: bfs_run
            .iter()
            .map(|s| s.max_list_to_explore_size)
            .sum::<usize>()
            / n,
        average_list_to_explore_size: bfs_run
            .iter()
            .map(|s| s.average_list_to_explore_size)
            .sum::<u128>()
            / n as u128,
    };
    println!("BFS average over {n} runs: {bfs_average:#?}");
}
