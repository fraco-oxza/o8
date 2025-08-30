#!/usr/bin/env python3
"""
o8 (Python) — 8-puzzle solver with CLI compatible output to the Rust version.

Subcommands:
  - benchmark: runs random boards and prints a comparison table (DFS, BFS, Heuristic)
  - solve-random: scrambles a solved board and prints the path with per-step distances

Notes:
  - Internals are independent, but outputs aim to match the Rust project's formatting.
"""
from __future__ import annotations

import argparse
import heapq
import random
import sys
import time
from collections import deque
from dataclasses import dataclass
from typing import Deque, Dict, Iterable, List, Optional, Sequence, Tuple

from tqdm import tqdm


# ---------------------------- Board ---------------------------------

BOARD_SIDE = 3
BOARD_AREA = BOARD_SIDE * BOARD_SIDE
TILE_BIT_SIZE = 4
SOLVED_BOARD = 1_985_229_328  # matches Rust constant


class Direction:
    Up, Down, Left, Right = range(4)


ALL_DIRECTIONS = (Direction.Up, Direction.Down, Direction.Left, Direction.Right)


class Board:
    __slots__ = ("_v",)

    def __init__(self, v: int = SOLVED_BOARD):
        self._v = int(v)

    def copy(self) -> "Board":
        return Board(self._v)

    # Encoding helpers
    def _get_pos(self, value: int) -> int:
        offset = TILE_BIT_SIZE * value
        return (self._v >> offset) & ((1 << TILE_BIT_SIZE) - 1)

    def _set_value(self, p: int, val: int) -> None:
        ones = (1 << TILE_BIT_SIZE) - 1
        mask = ones << (TILE_BIT_SIZE * val)
        self._v &= ~mask
        self._v |= (p & ones) << (TILE_BIT_SIZE * val)

    def _get_value(self, p: int) -> int:
        # returns tile index (0..7) for position p, raises if p is empty
        for val in range(BOARD_AREA - 1):
            if self._get_pos(val) == p:
                return val
        raise ValueError("Invalid position (empty)")

    def _find_space_position(self) -> int:
        idx = 0
        for val in range(BOARD_AREA - 1):
            pos = self._get_pos(val)
            idx |= 1 << pos
        # find first 0 bit from LSB
        # there are 9 positions (0..8), exactly one is zero
        for i in range(BOARD_AREA):
            if ((idx >> i) & 1) == 0:
                return i
        return 0  # should not happen

    @staticmethod
    def _is_valid_movement(position: int, direction: int) -> bool:
        if direction == Direction.Up:
            return (position // BOARD_SIDE) != 0
        if direction == Direction.Down:
            return (position // BOARD_SIDE) != BOARD_SIDE - 1
        if direction == Direction.Left:
            return (position % BOARD_SIDE) != 0
        if direction == Direction.Right:
            return (position % BOARD_SIDE) != BOARD_SIDE - 1
        return False

    @staticmethod
    def _new_position(from_pos: int, direction: int) -> int:
        if not Board._is_valid_movement(from_pos, direction):
            raise ValueError("Invalid move: cannot move space in that direction")
        if direction == Direction.Up:
            return from_pos - BOARD_SIDE
        if direction == Direction.Down:
            return from_pos + BOARD_SIDE
        if direction == Direction.Left:
            return from_pos - 1
        return from_pos + 1

    def move_space(self, direction: int) -> "Board":
        space_position = self._find_space_position()
        space_new_position = self._new_position(space_position, direction)
        digit_to_move = self._get_value(space_new_position)
        b = self.copy()
        b._set_value(space_position, digit_to_move)
        return b

    def heuristic_distance_to_solution(self) -> int:
        distance = 0
        solved = Board()
        for val in range(BOARD_AREA - 1):
            distance += Board._manhattan(solved._get_pos(val), self._get_pos(val))
        distance += Board._manhattan(solved._find_space_position(), self._find_space_position())
        return distance

    @staticmethod
    def _manhattan(pos1: int, pos2: int) -> int:
        h = abs((pos2 % BOARD_SIDE) - (pos1 % BOARD_SIDE))
        v = abs((pos2 // BOARD_SIDE) - (pos1 // BOARD_SIDE))
        return h + v

    def into_arr(self) -> List[int]:
        arr = [0] * BOARD_AREA
        for val in range(BOARD_AREA - 1):
            pos = self._get_pos(val)
            arr[pos] = val + 1
        return arr

    def is_solved(self) -> bool:
        return self._v == SOLVED_BOARD

    @staticmethod
    def random_with_solution(steps: int) -> "Board":
        b = Board()
        for _ in range(steps):
            direction = random.choice(ALL_DIRECTIONS)
            try:
                b = b.move_space(direction)
            except ValueError:
                # ignore invalid move and try another in next iteration
                pass
        return b

    # Ordering by heuristic (like Rust Ord impl)
    def __lt__(self, other: "Board") -> bool:
        return self.heuristic_distance_to_solution() < other.heuristic_distance_to_solution()

    def __hash__(self) -> int:  # to use as dict/set key
        return hash(self._v)

    def __eq__(self, other: object) -> bool:
        return isinstance(other, Board) and self._v == other._v

    def __str__(self) -> str:
        arr = self.into_arr()
        lines = []
        for i in range(BOARD_AREA):
            if i % BOARD_SIDE == 0:
                lines.append("")
            if arr[i] != 0:
                lines[-1] += f"{arr[i]:2} "
            else:
                lines[-1] += "   "
        return "\n".join(lines)


@dataclass(order=True)
class BoardWithSteps:
    cost: int
    steps: int
    board: Board

    @staticmethod
    def make(board: Board, steps: int) -> "BoardWithSteps":
        return BoardWithSteps(board.heuristic_distance_to_solution() + steps, steps, board)


# ---------------------------- Stats ---------------------------------

@dataclass
class Stats:
    nodes_explored: int = 0
    solution_moves: int = 0
    max_frontier: int = 0
    avg_frontier: float = 0.0
    generated_nodes: int = 0
    enqueued_nodes: int = 0
    duplicates_pruned: int = 0
    max_depth_reached: int = 0
    duration_ms: float = 0.0

    def __str__(self) -> str:
        return (
            f"explored={self.nodes_explored}, moves={self.solution_moves}, "
            f"max_frontier={self.max_frontier}, avg_frontier={self.avg_frontier:.2f}, "
            f"gen={self.generated_nodes}, enq={self.enqueued_nodes}, "
            f"pruned={self.duplicates_pruned}, max_depth={self.max_depth_reached}, "
            f"time={self.duration_ms:.3f}ms"
        )


@dataclass
class StatsSummary:
    runs: int
    avg_nodes_explored: float
    avg_solution_moves: float
    avg_max_frontier: float
    avg_frontier: float
    avg_generated_nodes: float
    avg_enqueued_nodes: float
    avg_duplicates_pruned: float
    avg_max_depth_reached: float
    avg_duration_ms: float

    @staticmethod
    def from_runs(stats: Sequence[Stats]) -> "StatsSummary":
        n = max(1, len(stats))
        def avg(f):
            return sum(f(s) for s in stats) / n
        return StatsSummary(
            runs=len(stats),
            avg_nodes_explored=avg(lambda s: s.nodes_explored),
            avg_solution_moves=avg(lambda s: s.solution_moves),
            avg_max_frontier=avg(lambda s: s.max_frontier),
            avg_frontier=avg(lambda s: s.avg_frontier),
            avg_generated_nodes=avg(lambda s: s.generated_nodes),
            avg_enqueued_nodes=avg(lambda s: s.enqueued_nodes),
            avg_duplicates_pruned=avg(lambda s: s.duplicates_pruned),
            avg_max_depth_reached=avg(lambda s: s.max_depth_reached),
            avg_duration_ms=avg(lambda s: s.duration_ms),
        )


def fmt_num(n: float) -> str:
    if n == n and n not in (float("inf"), float("-inf")):
        if abs(n) >= 1000.0:
            return f"{n:.0f}"
        return f"{n:.2f}"
    return "NaN"


def print_comparison_table(left: StatsSummary, right: StatsSummary, other: StatsSummary) -> None:
    title = f"Strategy Comparison (runs: {left.runs}, Dfs vs Bfs vs Heuristic)"
    print(f"\n{title}\n")

    # Minimal comfy-like table with UTF-8 borders
    headers = ["Metric", "DFS (avg)", "BFS (avg)", "Heuristic (avg)"]
    rows = [
        ("Time per run (ms)", left.avg_duration_ms, right.avg_duration_ms, other.avg_duration_ms),
        ("Nodes explored", left.avg_nodes_explored, right.avg_nodes_explored, other.avg_nodes_explored),
        ("Nodes generated", left.avg_generated_nodes, right.avg_generated_nodes, other.avg_generated_nodes),
        ("Enqueued", left.avg_enqueued_nodes, right.avg_enqueued_nodes, other.avg_enqueued_nodes),
        ("Discards (duplicates)", left.avg_duplicates_pruned, right.avg_duplicates_pruned, other.avg_duplicates_pruned),
        ("Solution length (moves)", left.avg_solution_moves, right.avg_solution_moves, other.avg_solution_moves),
        ("Peak frontier", left.avg_max_frontier, right.avg_max_frontier, other.avg_max_frontier),
        ("Average frontier", left.avg_frontier, right.avg_frontier, other.avg_frontier),
        ("Max depth", left.avg_max_depth_reached, right.avg_max_depth_reached, other.avg_max_depth_reached),
    ]

    # Compute column widths
    def col_strs():
        yield headers
        for m, l, r, o in rows:
            yield [m, fmt_num(l), fmt_num(r), fmt_num(o)]

    cols = list(zip(*list(col_strs())))
    widths = [max(len(x) for x in col) for col in cols]

    def pad(s: str, w: int) -> str:
        return s + " " * (w - len(s))

    # Borders
    def top_border():
        return "╭" + "┄" * (widths[0] + 2) + "┬" + "┄" * (widths[1] + 2) + "┬" + "┄" * (widths[2] + 2) + "┬" + "┄" * (widths[3] + 2) + "╮"

    def mid_border():
        return "├" + "┄" * (widths[0] + 2) + "┼" + "┄" * (widths[1] + 2) + "┼" + "┄" * (widths[2] + 2) + "┼" + "┄" * (widths[3] + 2) + "┤"

    def bot_border():
        return "╰" + "┄" * (widths[0] + 2) + "┴" + "┄" * (widths[1] + 2) + "┴" + "┄" * (widths[2] + 2) + "┴" + "┄" * (widths[3] + 2) + "╯"

    def row(vals: List[str]) -> str:
        return "│ " + pad(vals[0], widths[0]) + " │ " + pad(vals[1], widths[1]) + " │ " + pad(vals[2], widths[2]) + " │ " + pad(vals[3], widths[3]) + " │"

    print(top_border())
    print(row(headers))
    print(mid_border())
    for m, l, r, o in rows:
        print(row([m, fmt_num(l), fmt_num(r), fmt_num(o)]))
    print(bot_border())


# ------------------------ Solver and strategies ----------------------

class ExplorerStrategy:
    Dfs = "dfs"
    Bfs = "bfs"
    Heuristic = "heuristic"


class Frontier:
    def __len__(self) -> int:  # pragma: no cover
        return 0

    def enqueue(self, item: BoardWithSteps) -> None:  # pragma: no cover
        pass

    def get_next(self) -> Optional[BoardWithSteps]:  # pragma: no cover
        return None


class SimpleFrontier(Frontier):
    def __init__(self, algo: str) -> None:
        self._algo = algo
        self._dq: Deque[BoardWithSteps] = deque()

    def __len__(self) -> int:
        return len(self._dq)

    def enqueue(self, item: BoardWithSteps) -> None:
        self._dq.append(item)

    def get_next(self) -> Optional[BoardWithSteps]:
        if not self._dq:
            return None
        if self._algo == ExplorerStrategy.Bfs:
            return self._dq.popleft()
        return self._dq.pop()


class HeuristicFrontier(Frontier):
    def __init__(self) -> None:
        self._heap: List[BoardWithSteps] = []

    def __len__(self) -> int:
        return len(self._heap)

    def enqueue(self, item: BoardWithSteps) -> None:
        heapq.heappush(self._heap, item)

    def get_next(self) -> Optional[BoardWithSteps]:
        if not self._heap:
            return None
        return heapq.heappop(self._heap)


class Solver:
    def __init__(self, frontier: Frontier) -> None:
        self.parents: Dict[Board, Board] = {}
        self.boards_checked: set[Board] = set()
        self.frontier = frontier
        self.to_check_size: List[int] = []
        self.depth_by_board: Dict[Board, int] = {}
        self.generated_nodes = 0
        self.enqueued_nodes = 0
        self.duplicates_pruned = 0
        self.max_depth_reached = 0
        self.solve_duration_ms = 0.0

    def solve(self, start: Board) -> Optional[Board]:
        self.parents.clear()
        self.boards_checked.clear()
        self.to_check_size.clear()
        self.depth_by_board.clear()
        self.generated_nodes = 0
        self.enqueued_nodes = 0
        self.duplicates_pruned = 0
        self.max_depth_reached = 0
        self.solve_duration_ms = 0.0

        self.frontier.enqueue(BoardWithSteps.make(start, 0))
        self.depth_by_board[start] = 0

        t0 = time.perf_counter()

        while True:
            node = self.frontier.get_next()
            if node is None:
                break

            self.boards_checked.add(node.board)
            self.to_check_size.append(len(self.frontier))

            if node.board.is_solved():
                self.solve_duration_ms = (time.perf_counter() - t0) * 1000.0
                return node.board

            for direction in ALL_DIRECTIONS:
                try:
                    child_board = node.board.move_space(direction)
                except ValueError:
                    continue
                self.generated_nodes += 1
                if child_board in self.boards_checked:
                    self.duplicates_pruned += 1
                else:
                    child = BoardWithSteps.make(child_board, node.steps + 1)
                    self.enqueued_nodes += 1
                    self.parents[child_board] = node.board
                    depth = self.depth_by_board.get(node.board, 0) + 1
                    self.depth_by_board[child_board] = depth
                    if depth > self.max_depth_reached:
                        self.max_depth_reached = depth
                    self.frontier.enqueue(child)

        self.solve_duration_ms = (time.perf_counter() - t0) * 1000.0
        return None

    def get_solution_stats(self) -> Stats:
        size = max(1, len(self.to_check_size))
        avg_frontier = sum(self.to_check_size) / size
        max_frontier = max(self.to_check_size) if self.to_check_size else 0
        solution_moves = max(0, len(self.step_by_step_solution()) - 1)

        return Stats(
            nodes_explored=len(self.boards_checked),
            solution_moves=solution_moves,
            max_frontier=max_frontier,
            avg_frontier=avg_frontier,
            generated_nodes=self.generated_nodes,
            enqueued_nodes=self.enqueued_nodes,
            duplicates_pruned=self.duplicates_pruned,
            max_depth_reached=self.max_depth_reached,
            duration_ms=self.solve_duration_ms,
        )

    def step_by_step_solution(self) -> List[Board]:
        c = Board()  # solved
        solution = [c]
        while c in self.parents:
            nc = self.parents[c]
            solution.append(nc)
            c = nc
        solution.reverse()
        return solution


# ----------------------------- CLI ----------------------------------

def run_benchmark(runs: int, scramble_steps: int, threads: Optional[int]) -> None:
    print(f"Generating {runs} random boards with {scramble_steps} moves and comparing strategies...")
    if threads is not None:
        print(f"Using {threads} threads for parallel execution.")

    boards: List[Board] = [Board.random_with_solution(scramble_steps) for _ in range(runs)]

    print("Running DFS...")
    dfs_stats: List[Stats] = []
    for b in tqdm(boards):
        s = Solver(SimpleFrontier(ExplorerStrategy.Dfs))
        s.solve(b)
        dfs_stats.append(s.get_solution_stats())

    print("Running BFS...")
    bfs_stats: List[Stats] = []
    for b in tqdm(boards):
        s = Solver(SimpleFrontier(ExplorerStrategy.Bfs))
        s.solve(b)
        bfs_stats.append(s.get_solution_stats())

    print("Running Heuristic Search (A*-style) ...")
    heu_stats: List[Stats] = []
    for b in tqdm(boards):
        s = Solver(HeuristicFrontier())
        s.solve(b)
        heu_stats.append(s.get_solution_stats())

    print_comparison_table(
        StatsSummary.from_runs(dfs_stats),
        StatsSummary.from_runs(bfs_stats),
        StatsSummary.from_runs(heu_stats),
    )


def _print_solve_steps(solution: List[Board], stats: Stats) -> None:
    for idx, step in enumerate(solution):
        print("-" * 6)
        print(step)
        print("distance to solution")
        print(f"  estimated : {step.heuristic_distance_to_solution()}")
        print(f"  real      : {len(solution) - idx - 1}")
        print("-" * 6)

    # Emulate Rust's pretty Debug for Stats
    # Field order matches the dataclass definition above.
    print("Stats {")
    print(f"    nodes_explored: {stats.nodes_explored},")
    print(f"    solution_moves: {stats.solution_moves},")
    print(f"    max_frontier: {stats.max_frontier},")
    print(f"    avg_frontier: {stats.avg_frontier},")
    print(f"    generated_nodes: {stats.generated_nodes},")
    print(f"    enqueued_nodes: {stats.enqueued_nodes},")
    print(f"    duplicates_pruned: {stats.duplicates_pruned},")
    print(f"    max_depth_reached: {stats.max_depth_reached},")
    # Rust prints u128 as integer; we keep ms as float but print full precision similar to Python default
    print(f"    duration_ms: {stats.duration_ms},")
    print("}")


def run_solve_random(scramble_steps: int, algorithm: str) -> None:
    board = Board.random_with_solution(scramble_steps)
    if algorithm == ExplorerStrategy.Dfs:
        solver = Solver(SimpleFrontier(ExplorerStrategy.Dfs))
    elif algorithm == ExplorerStrategy.Bfs:
        solver = Solver(SimpleFrontier(ExplorerStrategy.Bfs))
    else:
        solver = Solver(HeuristicFrontier())

    solver.solve(board)
    solution = solver.step_by_step_solution()
    _print_solve_steps(solution, solver.get_solution_stats())


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="Command-line arguments for the 8-puzzle solver")
    sub = p.add_subparsers(dest="command", required=True)

    b = sub.add_parser("benchmark", help="Run many random boards and compare strategies with aggregate stats")
    b.add_argument("-r", "--runs", type=int, default=200, dest="runs", help="Number of test runs to perform for each algorithm")
    b.add_argument("-s", "--scramble-steps", type=int, default=200, dest="scramble_steps", help="Number of scramble steps to generate random puzzle boards")
    b.add_argument("-t", "--threads", type=int, default=None, dest="threads", help="Number of worker threads to use (display only)")

    s = sub.add_parser("solve-random", help="Solve a single random board and print the path")
    s.add_argument("-a", "--algorithm", choices=["dfs", "bfs", "heuristic"], default="heuristic", help="Algorithm to use (defaults to heuristic)")
    s.add_argument("-s", "--scramble-steps", type=int, default=200, dest="scramble_steps", help="Number of scramble steps to generate random puzzle boards")
    return p


def main(argv: Optional[List[str]] = None) -> int:
    args = build_parser().parse_args(argv)
    if args.command == "benchmark":
        run_benchmark(args.runs, args.scramble_steps, args.threads)
        return 0
    if args.command == "solve-random":
        run_solve_random(args.scramble_steps, args.algorithm)
        return 0
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
