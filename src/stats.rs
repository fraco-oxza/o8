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
            "explored={}, moves={}, max_frontier={}, gen={}, enq={}, pruned={}, max_depth={}, time={}ms",
            self.nodes_explored,
            self.solution_moves,
            self.max_frontier,
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
    /// Number of board states explored per run (mean ± std)
    pub nodes_explored: Metric,
    /// Number of moves in solutions found (mean ± std)
    pub solution_moves: Metric,
    /// Maximum frontier size per run (mean ± std)
    pub max_frontier: Metric,
    /// Successor states generated per run (mean ± std)
    pub generated_nodes: Metric,
    /// States enqueued per run (mean ± std)
    pub enqueued_nodes: Metric,
    /// Duplicate states pruned per run (mean ± std)
    pub duplicates_pruned: Metric,
    /// Maximum depth reached per run (mean ± std)
    pub max_depth_reached: Metric,
    /// Solve time per run in milliseconds (mean ± std)
    pub duration_ms: Metric,
}

/// A numeric metric summarized by common percentiles
#[derive(Clone, Copy, Debug, Default)]
pub struct Metric {
    pub p50: u64,
    pub p75: u64,
    pub p90: u64,
    pub p95: u64,
    pub p99: u64,
}

impl Metric {
    #[inline]
    fn new(p50: u64, p75: u64, p90: u64, p95: u64, p99: u64) -> Self {
        Self {
            p50,
            p75,
            p90,
            p95,
            p99,
        }
    }
}

/// Converts a slice of individual stats into an aggregated summary
impl From<&[Stats]> for StatsSummary {
    fn from(value: &[Stats]) -> Self {
        // Helper to compute integer percentiles (nearest-rank) for any projection
        fn summarize<T, F>(items: &[T], f: F) -> Metric
        where
            F: Fn(&T) -> u64,
        {
            let n = items.len();
            if n == 0 {
                return Metric::default();
            }

            let mut vals: Vec<u64> = items.iter().map(f).collect();
            vals.sort_unstable();
            let idx = |p: u32| -> usize {
                // nearest-rank: ceil(p/100 * n), 1-based -> to 0-based index
                let rank = (p as usize * n).div_ceil(100);
                rank.saturating_sub(1).min(n - 1)
            };
            let p50 = vals[idx(50)];
            let p75 = vals[idx(75)];
            let p90 = vals[idx(90)];
            let p95 = vals[idx(95)];
            let p99 = vals[idx(99)];

            Metric::new(p50, p75, p90, p95, p99)
        }

        Self {
            runs: value.len(),
            nodes_explored: summarize(value, |s| s.nodes_explored as u64),
            solution_moves: summarize(value, |s| s.solution_moves as u64),
            max_frontier: summarize(value, |s| s.max_frontier as u64),
            generated_nodes: summarize(value, |s| s.generated_nodes as u64),
            enqueued_nodes: summarize(value, |s| s.enqueued_nodes as u64),
            duplicates_pruned: summarize(value, |s| s.duplicates_pruned as u64),
            max_depth_reached: summarize(value, |s| s.max_depth_reached as u64),
            duration_ms: summarize(value, |s| u64::try_from(s.duration_ms).unwrap_or(u64::MAX)),
        }
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

    // Helper to print a single-metric table
    let print_metric_table = |metric_name: &str, l: &Metric, r: &Metric, o: &Metric| {
        let mut t = Table::new();
        t.load_preset(presets::UTF8_FULL_CONDENSED);
        t.apply_modifier(modifiers::UTF8_ROUND_CORNERS);
        t.set_content_arrangement(ContentArrangement::Dynamic);
        t.set_header([
            Cell::new(metric_name).add_attribute(Attribute::Bold),
            Cell::new("P50"),
            Cell::new("P75"),
            Cell::new("P90"),
            Cell::new("P95"),
            Cell::new("P99"),
        ]);

        // DFS row
        t.add_row([
            Cell::new("DFS").add_attribute(Attribute::Bold),
            Cell::new(l.p50).set_alignment(CellAlignment::Right),
            Cell::new(l.p75).set_alignment(CellAlignment::Right),
            Cell::new(l.p90).set_alignment(CellAlignment::Right),
            Cell::new(l.p95).set_alignment(CellAlignment::Right),
            Cell::new(l.p99).set_alignment(CellAlignment::Right),
        ]);
        // BFS row
        t.add_row([
            Cell::new("BFS").add_attribute(Attribute::Bold),
            Cell::new(r.p50).set_alignment(CellAlignment::Right),
            Cell::new(r.p75).set_alignment(CellAlignment::Right),
            Cell::new(r.p90).set_alignment(CellAlignment::Right),
            Cell::new(r.p95).set_alignment(CellAlignment::Right),
            Cell::new(r.p99).set_alignment(CellAlignment::Right),
        ]);
        // Heuristic row
        t.add_row([
            Cell::new("Heuristic").add_attribute(Attribute::Bold),
            Cell::new(o.p50).set_alignment(CellAlignment::Right),
            Cell::new(o.p75).set_alignment(CellAlignment::Right),
            Cell::new(o.p90).set_alignment(CellAlignment::Right),
            Cell::new(o.p95).set_alignment(CellAlignment::Right),
            Cell::new(o.p99).set_alignment(CellAlignment::Right),
        ]);

        println!("{t}\n");
    };

    print_metric_table(
        "Time per run (ms)",
        &left.duration_ms,
        &right.duration_ms,
        &other.duration_ms,
    );
    print_metric_table(
        "Nodes explored",
        &left.nodes_explored,
        &right.nodes_explored,
        &other.nodes_explored,
    );
    print_metric_table(
        "Nodes generated",
        &left.generated_nodes,
        &right.generated_nodes,
        &other.generated_nodes,
    );
    print_metric_table(
        "Enqueued",
        &left.enqueued_nodes,
        &right.enqueued_nodes,
        &other.enqueued_nodes,
    );
    print_metric_table(
        "Discards (duplicates)",
        &left.duplicates_pruned,
        &right.duplicates_pruned,
        &other.duplicates_pruned,
    );
    print_metric_table(
        "Solution length (moves)",
        &left.solution_moves,
        &right.solution_moves,
        &other.solution_moves,
    );
    print_metric_table(
        "Peak frontier",
        &left.max_frontier,
        &right.max_frontier,
        &other.max_frontier,
    );
    // Average frontier removed
    print_metric_table(
        "Max depth",
        &left.max_depth_reached,
        &right.max_depth_reached,
        &other.max_depth_reached,
    );

    println!("Legend:");
    println!("- Columns are percentiles: P50 (median), P75, P90, P95, P99.");
}

/// Prints a formatted table for a single run's statistics
///
/// Mirrors the labels used in the comparison table so outputs feel consistent
/// between `benchmark` and `solve-random` commands.
pub fn print_run_stats(stats: &Stats) {
    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL_CONDENSED);
    table.apply_modifier(modifiers::UTF8_ROUND_CORNERS);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(["Metric", "Value"]);

    let mut row = |metric: &str, v: String| {
        table.add_row([
            Cell::new(metric).add_attribute(Attribute::Bold),
            Cell::new(v).set_alignment(CellAlignment::Right),
        ]);
    };

    row("Time (ms)", format!("{}", stats.duration_ms));
    row("Nodes explored", format!("{}", stats.nodes_explored));
    row("Nodes generated", format!("{}", stats.generated_nodes));
    row("Enqueued", format!("{}", stats.enqueued_nodes));
    row(
        "Discards (duplicates)",
        format!("{}", stats.duplicates_pruned),
    );
    row(
        "Solution length (moves)",
        format!("{}", stats.solution_moves),
    );
    row("Peak frontier", format!("{}", stats.max_frontier));
    // Average frontier removed
    row("Max depth", format!("{}", stats.max_depth_reached));

    println!("\nRun statistics\n\n{table}");
}
