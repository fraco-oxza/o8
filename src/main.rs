use rayon::prelude::*;
use std::env;

use crate::{
    board::Board,
    solver::{ExplorerStrategy, Solver},
    stats::{Stats, StatsSummary, print_comparison_table},
};

pub(crate) mod board;
pub(crate) mod solver;
pub(crate) mod stats;

fn main() {
    // Configurable via env (fish/zsh/bash compatible): O8_RUNS, O8_SCRAMBLE_STEPS
    let n: usize = env::var("O8_RUNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(200);
    let scramble_steps: usize = env::var("O8_SCRAMBLE_STEPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(200);

    println!(
        "Generando {n} tableros aleatorios con {scramble_steps} movimientos y comparando DFS vs BFS..."
    );

    // Generar los mismos tableros para ambas estrategias
    let boards: Vec<Board> = (0..n)
        .into_par_iter()
        .map(|_| Board::random_with_solution(scramble_steps))
        .collect();

    // Ejecutar DFS
    let dfs_run: Vec<Stats> = boards
        .par_iter()
        .map(|b| {
            let mut solver = Solver::new(ExplorerStrategy::Dfs);
            solver.solve(*b).expect("No solution founded");
            solver.get_solution_stats()
        })
        .collect();

    // Ejecutar BFS
    let bfs_run: Vec<Stats> = boards
        .par_iter()
        .map(|b| {
            let mut solver = Solver::new(ExplorerStrategy::Bfs);
            solver.solve(*b).expect("No solution founded");
            solver.get_solution_stats()
        })
        .collect();

    let dfs_summary = StatsSummary::from_runs(ExplorerStrategy::Dfs, &dfs_run);
    let bfs_summary = StatsSummary::from_runs(ExplorerStrategy::Bfs, &bfs_run);

    print_comparison_table(&dfs_summary, &bfs_summary);
}
