# O8 - 8-Puzzle Solver

A high-performance 8-puzzle solver written in Rust that compares the effectiveness of different search strategies (Depth-First Search vs Breadth-First Search) using parallel processing for performance analysis.

## Overview

The 8-puzzle is a classic sliding puzzle consisting of a 3×3 grid with 8 numbered tiles and one empty space. The goal is to arrange the tiles in numerical order by sliding them into the empty space. This solver provides a comprehensive analysis of two fundamental search algorithms:

- **Depth-First Search (DFS)**: Explores as far as possible along each branch before backtracking
- **Breadth-First Search (BFS)**: Explores all neighbors at the current depth before moving deeper

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

Run with default settings (200 runs, 200 scramble steps):

```bash
cargo run --release
```

### Custom Configuration

```bash
# Run 50 tests with 100 scramble moves each
cargo run --release -- --runs 50 --scramble-steps 100

# Quick test with minimal complexity
cargo run --release -- --runs 10 --scramble-steps 10
```

### Command Line Options

- `-r, --runs <RUNS>`: Number of test runs to perform for each algorithm (default: 200)
- `-s, --scramble-steps <STEPS>`: Number of scramble steps to generate random puzzle boards (default: 200)
- `-h, --help`: Display help information

## Example Output

```
Generating 200 random boards with 200 moves and comparing DFS vs BFS...

Strategy Comparison (runs: 200, Dfs vs Bfs)

Metric                   DFS (avg)        BFS (avg)       
------------------------ ---------------- ----------------
Time per run (ms)        45.23            12.67           
Nodes explored           15234.50         892.34          
Nodes generated          42156.78         2134.89         
Enqueued                 26922.28         1242.55         
Discards (duplicates)    15234.50         892.34          
Solution length (moves)  187.45           23.12           
Peak frontier            12453.67         456.78          
Average frontier         6226.84          228.39          
Max depth                187.45           23.67           
Throughput (nodes/ms)    336.78           70.45           
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

### Parallel Processing

The solver leverages Rayon for parallel execution:
- Each puzzle instance is solved independently
- Results are collected and aggregated efficiently
- Scales with available CPU cores

### Search Implementation

Both algorithms share a common framework:
1. Initialize with starting board state
2. Maintain frontier of unexplored states
3. Track parent relationships for solution reconstruction
4. Collect comprehensive statistics during search

## Dependencies

- [`clap`](https://crates.io/crates/clap) - Command-line argument parsing
- [`rand`](https://crates.io/crates/rand) - Random board generation
- [`rayon`](https://crates.io/crates/rayon) - Parallel processing

## Documentation

Generate and view the full API documentation:

```bash
cargo doc --open
```

## Performance Tips

1. **Release Mode**: Always use `--release` for performance testing
2. **Scramble Steps**: Higher values create more complex puzzles but longer solve times
3. **Run Count**: More runs provide better statistical significance
4. **System Resources**: Performance scales with available CPU cores

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

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
