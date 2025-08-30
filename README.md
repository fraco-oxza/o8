# O8 - 8-Puzzle Solver

A high-performance 8-puzzle solver in Rust that compares multiple search strategies
in parallel and reports rich statistics. It supports Depth-First Search (DFS),
Breadth-First Search (BFS), and a heuristic best-first/A*-style strategy.

## Overview

The 8-puzzle is a classic sliding puzzle consisting of a 3×3 grid with 8 numbered tiles and one empty space. The goal is to arrange the tiles in numerical order by sliding them into the empty space. This solver provides a comprehensive analysis of three search algorithms:

- **Depth-First Search (DFS)**: Explores as far as possible along each branch before backtracking
- **Breadth-First Search (BFS)**: Explores all neighbors at the current depth before moving deeper
- **Heuristic (A\*-style)**: Expands states by increasing f(n) = g(n) + h(n), where h(n) is Manhattan distance

## Features

- 🚀 **High Performance**: Efficient board representation using compact 32-bit encoding
- 🔄 **Parallel Processing**: Uses Rayon for concurrent puzzle solving
- 📊 **Comprehensive Statistics**: Detailed performance metrics and comparison tables
- 🛠️ **Configurable**: Customizable number of runs and scramble complexity
- 📝 **Well Documented**: Extensive rustdoc documentation throughout

## Installation

### Prerequisites

- Rust 1.70 or later (uses 2024 edition)

### Building from Source

```bash
git clone https://github.com/fraco-oxza/o8.git
cd o8
cargo build --release
```

### Running

```bash
cargo run --release
```

## Usage

### Basic Usage

Run a benchmark with default settings (200 runs, 200 scramble steps):

```bash
cargo run --release
```

### Subcommands

This binary provides two subcommands: `benchmark` and `solve-random`.

1) Benchmark strategies in parallel and print comparison table:

```bash
# Run 50 tests with 100 scramble moves each
cargo run --release -- benchmark --runs 50 --scramble-steps 100

# Quick test with minimal complexity
cargo run --release -- benchmark --runs 10 --scramble-steps 10

Optional: Fix the number of threads used by Rayon:

```bash
cargo run --release -- benchmark --runs 200 --scramble-steps 200 --threads 8
```

2) Solve a single random board and print the path using a selected algorithm:

```bash
# Heuristic (default)
cargo run --release -- solve-random --scramble-steps 40

# Force DFS or BFS
cargo run --release -- solve-random --algorithm dfs --scramble-steps 40
cargo run --release -- solve-random --algorithm bfs --scramble-steps 40
```
```

### Command Line Options

- `-r, --runs <RUNS>`: Number of test runs to perform for each algorithm (default: 200) [benchmark]
- `-s, --scramble-steps <STEPS>`: Number of scramble steps to generate random puzzle boards (default: 200) [benchmark, solve-random]
- `-t, --threads <N>`: Number of worker threads to use (defaults to Rayon automatic) [benchmark]
- `-a, --algorithm <dfs|bfs|heuristic>`: Algorithm for solve-random (default: heuristic)
- `-h, --help`: Display help information

## Example Output

```
Generating 200 random boards with 200 moves and comparing strategies...

Strategy Comparison (runs: 200, Dfs vs Bfs vs Heuristic)

Metric                   DFS (avg)        BFS (avg)        Heuristic (avg)
------------------------ ---------------- ---------------- ----------------
Time per run (ms)        45.23            12.67            8.41
Nodes explored           15234.50         892.34           650.10
Nodes generated          42156.78         2134.89          1620.33
Enqueued                 26922.28         1242.55          970.23
Discards (duplicates)    15234.50         892.34           650.10
Solution length (moves)  187.45           23.12            23.12
Peak frontier            12453.67         456.78           310.21
Average frontier         6226.84          228.39           156.59
Max depth                187.45           23.67            23.67
```

## Architecture

### Core Components

#### Board Representation
- **Compact Storage**: 32-bit integer encoding for efficient memory usage and fast operations
- **Implicit Empty Space**: Empty space represented as the missing position in the encoding
- **Hash-Friendly**: Optimized for use in hash tables and sets

#### Search Algorithms
- **Unified Interface**: Common solver interface supporting multiple strategies
- **State Tracking**: Comprehensive statistics collection during search
- **Duplicate Detection**: Efficient pruning of already-visited states

#### Statistics Engine
- **Detailed Metrics**: Tracks 10+ performance indicators
- **Aggregation**: Statistical summaries across multiple runs
- **Formatted Output**: Clean, readable comparison tables

### Performance Characteristics

| Metric | DFS | BFS |
|--------|-----|-----|
| **Solution Quality** | Suboptimal (longer paths) | Optimal (shortest paths) |
| **Memory Usage** | Lower (stack-based) | Higher (queue-based) |
| **Time Complexity** | Variable (can be very slow) | Predictable |
| **Space Complexity** | O(d) where d is depth | O(b^d) where b is branching factor |

## Technical Details

### Board Encoding

The board state is encoded in a compact 32-bit representation where each tile's position is stored using 4 bits. This allows for:
- Fast copying and comparison operations
- Efficient hash table storage
- Minimal memory footprint

#### Encoding Strategy

The key insight is that instead of storing "what tile is at each position", we store "at what position is each tile". This clever approach:

1. **Uses only 32 bits total** (8 tiles × 4 bits each = 32 bits)
2. **Makes the empty space implicit** (the position not occupied by any tile)
3. **Enables fast bitwise operations** for moves and comparisons

#### Bit Layout

```text
Bits:  31-28  27-24  23-20  19-16  15-12  11-8   7-4    3-0
Tile:    8      7      6      5      4     3      2      1
Value: pos8   pos7   pos6   pos5   pos4  pos3   pos2   pos1
```

#### Example: Solved State

```text
Board Layout:         Encoding:
1 2 3                Tile 1 at pos 0: 0000
4 5 6                Tile 2 at pos 1: 0001
7 8                  Tile 3 at pos 2: 0010
                     Tile 4 at pos 3: 0011
Positions:           Tile 5 at pos 4: 0100
0 1 2                Tile 6 at pos 5: 0101
3 4 5                Tile 7 at pos 6: 0110
6 7 8                Tile 8 at pos 7: 0111
                     Empty at pos 8:  (implicit)
```

Binary: `01110110010101000011001000010000` = 1985229328

#### Testing the Encoding

You can explore the encoding with the built-in debug utilities:

```bash
cargo test test_solved_board_encoding -- --nocapture
cargo test test_encoding_explanation -- --nocapture
```

This will show you exactly how the bits are arranged and how they change when moves are made.

### Parallel Processing

The solver leverages Rayon for parallel execution:
- Each puzzle instance is solved independently
- Results are collected and aggregated efficiently
- Scales with available CPU cores

### Search Implementation

All strategies share a common framework:
1. Initialize with starting board state
2. Maintain frontier of unexplored states
3. Track parent relationships for solution reconstruction
4. Collect comprehensive statistics during search

## Dependencies

- [`clap`](https://crates.io/crates/clap) - Command-line argument parsing
- [`rand`](https://crates.io/crates/rand) - Random board generation
- [`rayon`](https://crates.io/crates/rayon) - Parallel processing
- [`comfy-table`](https://crates.io/crates/comfy-table) - Nicely formatted comparison table
- [`indicatif`](https://crates.io/crates/indicatif) - Parallel progress reporting

## Documentation

Generate and view the full API documentation:

```bash
cargo doc --open
```

## Disclaimer

Documentation in this repository (README and rustdoc comments) was created with AI support under the author's guidance. The author provided the intent and direction in most cases. The core algorithms, design, and implementation of the solver were written by hand.

## Performance Tips

1. **Release Mode**: Always use `--release` for performance testing
2. **Scramble Steps**: Higher values create more complex puzzles but longer solve times
3. **Run Count**: More runs provide better statistical significance
4. **System Resources**: Performance scales with available CPU cores

### Development Setup

```bash
git clone https://github.com/fraco-oxza/o8.git
cd o8
cargo test
cargo clippy
cargo fmt
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

**Francisco C. Ossa** - [francisco@oza.cl](mailto:francisco@oza.cl)

## Acknowledgments

- Classic 8-puzzle problem from artificial intelligence literature
- Rust community for excellent tooling and libraries
- Algorithm design patterns from computer science fundamentals
