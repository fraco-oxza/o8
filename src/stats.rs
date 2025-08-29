//! # Statistics Module
//!
//! This module provides comprehensive statistics collection and reporting
//! for the 8-puzzle solver performance analysis. It tracks various metrics
//! during the search process and provides formatted output for comparison
//! between different search strategies.

use std::fmt::{self, Display};

/// Width for metric column in comparison table
const METRIC_WIDTH: usize = 24;

/// Width for algorithm columns in comparison table
const ALGO_WIDTH: usize = 16;

/// Individual statistics for a single puzzle solve
///
/// Contains detailed metrics about the search process for one puzzle instance,
/// including performance data, search space exploration, and solution quality.
#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    /// Total number of board states explored
    pub nodes_explored: usize,
    /// Number of moves in the optimal solution found
    pub solution_moves: usize,
    /// Maximum size of the frontier during search
    pub max_frontier: usize,
    /// Average size of the frontier throughout the search
    pub avg_frontier: f64,
    /// Total number of successor states generated
    pub generated_nodes: usize,
    /// Total number of states added to the frontier
    pub enqueued_nodes: usize,
    /// Number of duplicate states that were pruned
    pub duplicates_pruned: usize,
    /// Maximum depth reached in the search tree
    pub max_depth_reached: usize,
    /// Time taken to solve the puzzle in milliseconds
    pub duration_ms: u128,
}

impl Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "explored={}, moves={}, max_frontier={}, avg_frontier={:.2}, gen={}, enq={}, pruned={}, max_depth={}, time={:.3}ms",
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

/// Aggregated statistics summary for multiple puzzle runs
///
/// Provides averaged metrics across multiple puzzle solves for comparing
/// the overall performance characteristics of different search strategies.
#[derive(Clone, Debug, Default)]
pub struct StatsSummary {
    /// Number of puzzle instances included in this summary
    pub runs: usize,
    /// Average number of board states explored per run
    pub avg_nodes_explored: f64,
    /// Average number of moves in solutions found
    pub avg_solution_moves: f64,
    /// Average maximum frontier size per run
    pub avg_max_frontier: f64,
    /// Average frontier size throughout all runs
    pub avg_frontier: f64,
    /// Average number of successor states generated per run
    pub avg_generated_nodes: f64,
    /// Average number of states enqueued per run
    pub avg_enqueued_nodes: f64,
    /// Average number of duplicate states pruned per run
    pub avg_duplicates_pruned: f64,
    /// Average maximum depth reached per run
    pub avg_max_depth_reached: f64,
    /// Average solve time per run in milliseconds
    pub avg_duration_ms: f64,
    /// Throughput metric: nodes explored per millisecond
    pub throughput_nodes_per_ms: f64,
}

/// Converts a slice of individual stats into an aggregated summary
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
        let avg_duration_ms = value.iter().fold(0.0, |a, s| sum(a, s.duration_ms as f64)) / n;
        let throughput_nodes_per_ms = if avg_duration_ms > 0.0 {
            avg_nodes_explored / avg_duration_ms
        } else {
            0.0
        };

        Self {
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

/// Formats a cell in the comparison table with proper padding
///
/// # Arguments
///
/// * `width` - The desired width of the cell
/// * `val` - The value to display in the cell
///
/// # Returns
///
/// A string padded to the specified width
fn fmt_cell(width: usize, val: impl Into<String>) -> String {
    let s = val.into();
    if s.len() >= width {
        s
    } else {
        let pad = width - s.len();
        let mut out = String::with_capacity(width);
        out.push_str(&s);
        out.extend(std::iter::repeat_n(" ", pad));
        out
    }
}

/// Formats a numeric value for display in the comparison table
///
/// # Arguments
///
/// * `n` - The numeric value to format
///
/// # Returns
///
/// A formatted string with appropriate precision
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

/// Prints a formatted comparison table of two search strategies
///
/// Displays a comprehensive side-by-side comparison of performance metrics
/// for two different search strategies (typically DFS vs BFS).
///
/// # Arguments
///
/// * `left` - Statistics summary for the first strategy
/// * `right` - Statistics summary for the second strategy
pub fn print_comparison_table(left: &StatsSummary, right: &StatsSummary) {
    let title = format!("Strategy Comparison (runs: {})", left.runs);
    println!("\n{title}\n");

    let headers = [
        ("Metric", METRIC_WIDTH),
        ("DFS (avg)", ALGO_WIDTH),
        ("BFS (avg)", ALGO_WIDTH),
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

    let row = |metric: &str, l: f64, r: f64| {
        println!(
            "{} {} {}",
            fmt_cell(METRIC_WIDTH, metric),
            fmt_cell(ALGO_WIDTH, fmt_num(l)),
            fmt_cell(ALGO_WIDTH, fmt_num(r)),
        );
    };

    row(
        "Time per run (ms)",
        left.avg_duration_ms,
        right.avg_duration_ms,
    );
    row(
        "Nodes explored",
        left.avg_nodes_explored,
        right.avg_nodes_explored,
    );
    row(
        "Nodes generated",
        left.avg_generated_nodes,
        right.avg_generated_nodes,
    );
    row(
        "Enqueued",
        left.avg_enqueued_nodes,
        right.avg_enqueued_nodes,
    );
    row(
        "Discards (duplicates)",
        left.avg_duplicates_pruned,
        right.avg_duplicates_pruned,
    );
    row(
        "Solution length (moves)",
        left.avg_solution_moves,
        right.avg_solution_moves,
    );
    row(
        "Peak frontier",
        left.avg_max_frontier,
        right.avg_max_frontier,
    );
    row("Average frontier", left.avg_frontier, right.avg_frontier);
    row(
        "Max depth",
        left.avg_max_depth_reached,
        right.avg_max_depth_reached,
    );
    row(
        "Throughput (nodes/ms)",
        left.throughput_nodes_per_ms,
        right.throughput_nodes_per_ms,
    );
}
