//! # Statistics Module
//!
//! This module provides comprehensive statistics collection and reporting
//! for the 8-puzzle solver performance analysis. It tracks various metrics
//! during the search process and offers formatted output to compare
//! different search strategies side-by-side.

use std::fmt::{self, Display};

use comfy_table::{Attribute, Cell, CellAlignment, ContentArrangement, Table, modifiers, presets};

// Type aliases to keep signatures readable when describing comparison sections
type SectionAccessor = fn(&StatsSummary) -> &Metric;
type SectionDesc = (&'static str, &'static str, SectionAccessor);

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

    /// Build a Metric from a slice and a projection function.
    /// Uses nearest-rank percentile on sorted values.
    #[inline]
    fn from_slice<T, F>(items: &[T], f: F) -> Self
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
        Metric::new(
            vals[idx(50)],
            vals[idx(75)],
            vals[idx(90)],
            vals[idx(95)],
            vals[idx(99)],
        )
    }
}

/// Converts a slice of individual stats into an aggregated summary
impl From<&[Stats]> for StatsSummary {
    fn from(value: &[Stats]) -> Self {
        Self {
            runs: value.len(),
            nodes_explored: Metric::from_slice(value, |s| s.nodes_explored as u64),
            solution_moves: Metric::from_slice(value, |s| s.solution_moves as u64),
            max_frontier: Metric::from_slice(value, |s| s.max_frontier as u64),
            generated_nodes: Metric::from_slice(value, |s| s.generated_nodes as u64),
            enqueued_nodes: Metric::from_slice(value, |s| s.enqueued_nodes as u64),
            duplicates_pruned: Metric::from_slice(value, |s| s.duplicates_pruned as u64),
            max_depth_reached: Metric::from_slice(value, |s| s.max_depth_reached as u64),
            duration_ms: Metric::from_slice(value, |s| {
                u64::try_from(s.duration_ms).unwrap_or(u64::MAX)
            }),
        }
    }
}

// ---------- Rendering helpers (SRP: isolate table rendering) ----------

fn new_base_table() -> Table {
    let mut t = Table::new();
    t.load_preset(presets::UTF8_FULL_CONDENSED);
    t.apply_modifier(modifiers::UTF8_ROUND_CORNERS);
    t.set_content_arrangement(ContentArrangement::Dynamic);
    t
}

fn add_percentile_row(t: &mut Table, label: &str, m: &Metric) {
    t.add_row([
        Cell::new(label).add_attribute(Attribute::Bold),
        Cell::new(m.p50).set_alignment(CellAlignment::Right),
        Cell::new(m.p75).set_alignment(CellAlignment::Right),
        Cell::new(m.p90).set_alignment(CellAlignment::Right),
        Cell::new(m.p95).set_alignment(CellAlignment::Right),
        Cell::new(m.p99).set_alignment(CellAlignment::Right),
    ]);
}

fn add_value_row(t: &mut Table, metric: &str, value: &dyn Display) {
    t.add_row([
        Cell::new(metric).add_attribute(Attribute::Bold),
        Cell::new(format!("{value}")).set_alignment(CellAlignment::Right),
    ]);
}

fn print_percentile_section<'a>(
    title: &str,
    desc: &str,
    rows: impl IntoIterator<Item = (&'a str, &'a Metric)>,
) {
    println!("{title} – {desc}");

    let mut t = new_base_table();
    t.set_header([
        Cell::new(title).add_attribute(Attribute::Bold),
        Cell::new("P50"),
        Cell::new("P75"),
        Cell::new("P90"),
        Cell::new("P95"),
        Cell::new("P99"),
    ]);

    for (label, metric) in rows {
        add_percentile_row(&mut t, label, metric);
    }

    println!("{t}\n");
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

    let strategies: [(&str, &StatsSummary); 3] =
        [("DFS", left), ("BFS", right), ("Heuristic", other)];

    // Descriptor: label, description, accessor to metric in a StatsSummary
    let sections: [SectionDesc; 8] = [
        (
            "Time per run (ms)",
            "Wall-clock time to solve one instance (milliseconds).",
            |s| &s.duration_ms,
        ),
        (
            "Nodes explored",
            "Unique states that were expanded (visited).",
            |s| &s.nodes_explored,
        ),
        (
            "Nodes generated",
            "Total successors produced before filtering (may include duplicates).",
            |s| &s.generated_nodes,
        ),
        (
            "Enqueued",
            "Generated states accepted into the frontier after filtering.",
            |s| &s.enqueued_nodes,
        ),
        (
            "Discards (duplicates)",
            "Generated states dropped because they were duplicates or already seen.",
            |s| &s.duplicates_pruned,
        ),
        (
            "Solution length (moves)",
            "Number of moves in the solution path found.",
            |s| &s.solution_moves,
        ),
        (
            "Peak frontier",
            "Maximum size of the frontier observed (proxy for peak memory).",
            |s| &s.max_frontier,
        ),
        (
            "Max depth",
            "Deepest depth reached in the search tree.",
            |s| &s.max_depth_reached,
        ),
    ];

    for (label, desc, accessor) in sections {
        let rows = strategies
            .into_iter()
            .map(|(name, ss)| (name, accessor(ss)));
        print_percentile_section(label, desc, rows);
    }

    println!("Legend:");
    println!("- Columns are percentiles: P50 (median), P75, P90, P95, P99.");
}

/// Prints a formatted table for a single run's statistics
///
/// Mirrors the labels used in the comparison table so outputs feel consistent
/// between `benchmark` and `solve-random` commands.
pub fn print_run_stats(stats: &Stats) {
    let mut table = new_base_table();
    table.set_header(["Metric", "Value"]);

    add_value_row(&mut table, "Time (ms)", &stats.duration_ms);
    add_value_row(&mut table, "Nodes explored", &stats.nodes_explored);
    add_value_row(&mut table, "Nodes generated", &stats.generated_nodes);
    add_value_row(&mut table, "Enqueued", &stats.enqueued_nodes);
    add_value_row(
        &mut table,
        "Discards (duplicates)",
        &stats.duplicates_pruned,
    );
    add_value_row(&mut table, "Solution length (moves)", &stats.solution_moves);
    add_value_row(&mut table, "Peak frontier", &stats.max_frontier);
    add_value_row(&mut table, "Max depth", &stats.max_depth_reached);

    println!("\nRun statistics\n\n{table}");
}
