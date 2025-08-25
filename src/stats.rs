use std::fmt::{self, Display};

use crate::solver::ExplorerStrategy;

#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    pub strategy: ExplorerStrategy,
    pub nodes_explored: usize,
    pub solution_moves: usize,
    pub max_frontier: usize,
    pub avg_frontier: f64,
    pub generated_nodes: usize,
    pub enqueued_nodes: usize,
    pub duplicates_pruned: usize,
    pub max_depth_reached: usize,
    pub duration_ms: f64,
}

impl Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}: explored={}, moves={}, max_frontier={}, avg_frontier={:.2}, gen={}, enq={}, pruned={}, max_depth={}, time={:.3}ms",
            self.strategy,
            self.nodes_explored,
            self.solution_moves,
            self.max_frontier,
            self.avg_frontier,
            self.generated_nodes,
            self.enqueued_nodes,
            self.duplicates_pruned,
            self.max_depth_reached,
            self.duration_ms,
        )
    }
}

#[derive(Clone, Debug, Default)]
pub struct StatsSummary {
    pub strategy: ExplorerStrategy,
    pub runs: usize,
    pub avg_nodes_explored: f64,
    pub avg_solution_moves: f64,
    pub avg_max_frontier: f64,
    pub avg_frontier: f64,
    pub avg_generated_nodes: f64,
    pub avg_enqueued_nodes: f64,
    pub avg_duplicates_pruned: f64,
    pub avg_max_depth_reached: f64,
    pub avg_duration_ms: f64,
    pub throughput_nodes_per_ms: f64,
}

impl From<&[Stats]> for StatsSummary {
    fn from(value: &[Stats]) -> Self {
        let n = value.len().max(1) as f64;
        let sum = |mut acc: f64, v: f64| {
            acc += v;
            acc
        };

        let avg_nodes_explored = value
            .iter()
            .fold(0.0, |a, s| sum(a, s.nodes_explored as f64))
            / n;
        let avg_solution_moves = value
            .iter()
            .fold(0.0, |a, s| sum(a, s.solution_moves as f64))
            / n;
        let avg_max_frontier = value.iter().fold(0.0, |a, s| sum(a, s.max_frontier as f64)) / n;
        let avg_frontier = value.iter().fold(0.0, |a, s| sum(a, s.avg_frontier)) / n;
        let avg_generated_nodes = value
            .iter()
            .fold(0.0, |a, s| sum(a, s.generated_nodes as f64))
            / n;
        let avg_enqueued_nodes = value
            .iter()
            .fold(0.0, |a, s| sum(a, s.enqueued_nodes as f64))
            / n;
        let avg_duplicates_pruned = value
            .iter()
            .fold(0.0, |a, s| sum(a, s.duplicates_pruned as f64))
            / n;
        let avg_max_depth_reached = value
            .iter()
            .fold(0.0, |a, s| sum(a, s.max_depth_reached as f64))
            / n;
        let avg_duration_ms = value.iter().fold(0.0, |a, s| sum(a, s.duration_ms)) / n;
        let throughput_nodes_per_ms = if avg_duration_ms > 0.0 {
            avg_nodes_explored / avg_duration_ms
        } else {
            0.0
        };

        Self {
            strategy: value.first().expect("No runs provided").strategy,
            runs: value.len(),
            avg_nodes_explored,
            avg_solution_moves,
            avg_max_frontier,
            avg_frontier,
            avg_generated_nodes,
            avg_enqueued_nodes,
            avg_duplicates_pruned,
            avg_max_depth_reached,
            avg_duration_ms,
            throughput_nodes_per_ms,
        }
    }
}

fn fmt_cell(width: usize, val: impl Into<String>) -> String {
    let s = val.into();
    if s.len() >= width {
        s
    } else {
        let pad = width - s.len();
        let mut out = String::with_capacity(width);
        out.push_str(&s);
        out.extend(std::iter::repeat(' ').take(pad));
        out
    }
}

fn fmt_num(n: f64) -> String {
    if n.is_finite() {
        if n.abs() >= 1000.0 {
            format!("{:.0}", n)
        } else {
            format!("{:.2}", n)
        }
    } else {
        "NaN".to_string()
    }
}

pub fn print_comparison_table(left: &StatsSummary, right: &StatsSummary) {
    let title = format!(
        "Comparativa de estrategias (runs: {}, {} vs {})",
        left.runs,
        format!("{:?}", left.strategy),
        format!("{:?}", right.strategy)
    );
    println!("\n{title}\n");

    let headers = vec![
        ("Métrica", 24usize),
        ("DFS (avg)", 16usize),
        ("BFS (avg)", 16usize),
        ("Mejor", 8usize),
    ];

    let sep: String = headers
        .iter()
        .map(|(h, w)| "-".repeat((*w).max(h.len())))
        .collect::<Vec<_>>()
        .join(" ");

    let header_line = headers
        .iter()
        .map(|(h, w)| fmt_cell(*w, (*h).to_string()))
        .collect::<Vec<_>>()
        .join(" ");
    println!("{}\n{}", header_line, sep);

    let row = |metric: &str, l: f64, r: f64, higher_is_better: bool| {
        let better = if (higher_is_better && l > r) || (!higher_is_better && l < r) {
            "DFS"
        } else if (higher_is_better && r > l) || (!higher_is_better && r < l) {
            "BFS"
        } else {
            "="
        };
        println!(
            "{} {} {} {}",
            fmt_cell(24, metric),
            fmt_cell(16, fmt_num(l)),
            fmt_cell(16, fmt_num(r)),
            fmt_cell(8, better),
        );
    };

    row(
        "Tiempo por run (ms)",
        left.avg_duration_ms,
        right.avg_duration_ms,
        false,
    );
    row(
        "Nodos explorados",
        left.avg_nodes_explored,
        right.avg_nodes_explored,
        false,
    );
    row(
        "Nodos generados",
        left.avg_generated_nodes,
        right.avg_generated_nodes,
        false,
    );
    row(
        "Encolados",
        left.avg_enqueued_nodes,
        right.avg_enqueued_nodes,
        false,
    );
    row(
        "Descartes (duplicados)",
        left.avg_duplicates_pruned,
        right.avg_duplicates_pruned,
        false,
    );
    row(
        "Longitud solución (movs)",
        left.avg_solution_moves,
        right.avg_solution_moves,
        false,
    );
    row(
        "Pico frontera",
        left.avg_max_frontier,
        right.avg_max_frontier,
        false,
    );
    row(
        "Frontera media",
        left.avg_frontier,
        right.avg_frontier,
        false,
    );
    row(
        "Profundidad máx.",
        left.avg_max_depth_reached,
        right.avg_max_depth_reached,
        false,
    );
    row(
        "Rendimiento (nodos/ms)",
        left.throughput_nodes_per_ms,
        right.throughput_nodes_per_ms,
        true,
    );
}
