//! # Statistics Module
//!
//! This module provides comprehensive statistics collection and reporting
//! for the 8-puzzle solver performance analysis. It tracks various metrics
//! during the search process and offers formatted output to compare
//! different search strategies side-by-side.

use std::fmt::{self, Display};

use comfy_table::{Attribute, Cell, CellAlignment, ContentArrangement, Table, modifiers, presets};

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
        }
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
            format!("{n:.0}")
        } else {
            format!("{n:.2}")
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
pub fn print_comparison_table(left: &StatsSummary, right: &StatsSummary, other: &StatsSummary) {
    let title = format!(
        "Strategy Comparison (runs: {}, Dfs vs Bfs vs Heuristic)",
        left.runs
    );
    println!("\n{title}\n");

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL_CONDENSED);
    table.apply_modifier(modifiers::UTF8_ROUND_CORNERS);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(["Metric", "DFS (avg)", "BFS (avg)", "Heuristic (avg)"]);

    let mut row = |metric: &str, l: f64, r: f64, o: f64| {
        table.add_row([
            Cell::new(metric).add_attribute(Attribute::Bold),
            Cell::new(fmt_num(l)).set_alignment(CellAlignment::Right),
            Cell::new(fmt_num(r)).set_alignment(CellAlignment::Right),
            Cell::new(fmt_num(o)).set_alignment(CellAlignment::Right),
        ]);
    };

    row(
        "Time per run (ms)",
        left.avg_duration_ms,
        right.avg_duration_ms,
        other.avg_duration_ms,
    );
    row(
        "Nodes explored",
        left.avg_nodes_explored,
        right.avg_nodes_explored,
        other.avg_nodes_explored,
    );
    row(
        "Nodes generated",
        left.avg_generated_nodes,
        right.avg_generated_nodes,
        other.avg_generated_nodes,
    );
    row(
        "Enqueued",
        left.avg_enqueued_nodes,
        right.avg_enqueued_nodes,
        other.avg_enqueued_nodes,
    );
    row(
        "Discards (duplicates)",
        left.avg_duplicates_pruned,
        right.avg_duplicates_pruned,
        other.avg_duplicates_pruned,
    );
    row(
        "Solution length (moves)",
        left.avg_solution_moves,
        right.avg_solution_moves,
        other.avg_solution_moves,
    );
    row(
        "Peak frontier",
        left.avg_max_frontier,
        right.avg_max_frontier,
        other.avg_max_frontier,
    );
    row(
        "Average frontier",
        left.avg_frontier,
        right.avg_frontier,
        other.avg_frontier,
    );
    row(
        "Max depth",
        left.avg_max_depth_reached,
        right.avg_max_depth_reached,
        other.avg_max_depth_reached,
    );

    println!("{table}");
}
