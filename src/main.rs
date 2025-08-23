use std::time::Instant;

use crate::{board::Board, solver::Solver};

pub(crate) mod board;
pub(crate) mod solver;

fn main() {
    let b = Board::random_with_solution(1_000_000);
    println!("{}", b);

    let mut solver = Solver::default();
    let start = Instant::now();
    solver.solve(b).expect("No solution founded");
    let dt = start.elapsed();
    let progression = solver.step_by_step_solution();

    for board in &progression {
        println!("==================");
        println!("{board}");
    }

    println!("solved in {}ms", dt.as_millis());
    println!("with {} moves", progression.len());
    println!("nodos explorados: {}", solver.nodes_explored());
}
