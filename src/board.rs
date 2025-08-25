//! # Board Module
//!
//! This module contains the implementation of the 8-puzzle board and related functionality.
//! The board is represented as a compact 32-bit integer where each tile position is encoded
//! using 4 bits, allowing for efficient storage and manipulation.

use std::fmt::Display;

use rand::{rng, seq::IndexedRandom};

use Direction::*;

/// Array containing all possible movement directions
pub const ALL_DIRECTIONS: [Direction; 4] = [Up, Down, Left, Right];

/// The solved board state represented as a 32-bit integer
const SOLVED_BOARD: u32 = 1985229328;

/// The side length of the square board (3x3 grid)
const BOARD_SIDE: usize = 3;

/// The total number of positions on the board (9 positions)
const BOARD_AREA: usize = BOARD_SIDE * BOARD_SIDE;

/// Number of bits used to represent each tile position
const TILE_BIT_SIZE: usize = 4;

/// Represents the four possible directions for moving tiles in the puzzle
#[derive(Clone, Copy)]
pub enum Direction {
    /// Move a tile upward (space moves down)
    Up,
    /// Move a tile downward (space moves up)
    Down,
    /// Move a tile leftward (space moves right)
    Left,
    /// Move a tile rightward (space moves left)
    Right,
}

/// Represents an 8-puzzle board state
///
/// The board is stored as a compact 32-bit integer where each tile's position
/// is encoded using 4 bits. This allows for efficient storage, copying, and
/// hashing operations. The empty space is represented implicitly as the missing
/// position in the encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Board(u32);

impl Board {
    /// Generates a random board by performing random moves from the solved state
    ///
    /// This ensures that the generated board is always solvable, as it's created
    /// by scrambling the solved state with valid moves.
    ///
    /// # Arguments
    ///
    /// * `steps` - Number of random moves to perform for scrambling
    ///
    /// # Returns
    ///
    /// A randomly scrambled but solvable board
    pub fn random_with_solution(steps: usize) -> Board {
        let mut board = Board::default();
        let mut rng = rng();

        (0..steps).for_each(|_| {
            let _ = board
                .move_space(
                    *ALL_DIRECTIONS
                        .choose(&mut rng)
                        .expect("This should never happen"),
                )
                .map(|b| board = b);
        });

        board
    }

    /// Converts the compact board representation to a 2D array format
    ///
    /// # Returns
    ///
    /// A 9-element array where each position contains the tile number,
    /// with 0 representing the empty space
    fn into_arr(self) -> [u8; BOARD_AREA] {
        let bits = self.0;
        let mut arr = [0b0; BOARD_AREA];

        for val in 0..(BOARD_AREA - 1) {
            let pos = (bits.unbounded_shr((val * TILE_BIT_SIZE) as u32)) % (1 << TILE_BIT_SIZE);
            arr[pos as usize] = (val + 1) as u8;
        }

        arr
    }

    /// Checks if the board is in the solved state
    ///
    /// # Returns
    ///
    /// `true` if the board is solved (tiles are in numerical order), `false` otherwise
    pub fn is_solved(&self) -> bool {
        self.0 == SOLVED_BOARD
    }

    /// Validates if a movement is possible from a given position
    ///
    /// # Arguments
    ///
    /// * `position` - The current position of the empty space (0-8)
    /// * `direction` - The direction to move
    ///
    /// # Returns
    ///
    /// `true` if the movement is valid, `false` if it would move outside the board
    fn is_valid_movement(position: u32, direction: Direction) -> bool {
        let position = position as usize;
        match direction {
            Up => (position / BOARD_SIDE) != 0,
            Down => (position / BOARD_SIDE) != BOARD_SIDE - 1,
            Left => (position % BOARD_SIDE) != 0,
            Right => (position % BOARD_SIDE) != BOARD_SIDE - 1,
        }
    }

    /// Finds the current position of the empty space on the board
    ///
    /// The empty space is identified as the position not occupied by any tile.
    /// This is determined by creating a bitmask of all occupied positions and
    /// finding the first unset bit.
    ///
    /// # Returns
    ///
    /// The position (0-8) of the empty space
    fn find_space_position(&self) -> u32 {
        let mut idx: u32 = 0;

        for val in 0..(BOARD_AREA - 1) {
            let pos = (self.0.unbounded_shr((val * TILE_BIT_SIZE) as u32)) % (1 << TILE_BIT_SIZE);
            idx |= 1 << pos;
        }

        idx.trailing_ones()
    }

    /// Calculates the new position after moving in a specific direction
    ///
    /// # Arguments
    ///
    /// * `from` - The current position (0-8)
    /// * `direction` - The direction to move
    ///
    /// # Returns
    ///
    /// `Ok(new_position)` if the move is valid, or an error message if invalid
    fn calculate_new_position(from: u32, direction: Direction) -> Result<u32, &'static str> {
        if !Board::is_valid_movement(from, direction) {
            return Err("Invalid move: cannot move space in that direction");
        }

        Ok(match direction {
            Up => from - BOARD_SIDE as u32,
            Down => from + BOARD_SIDE as u32,
            Left => from - 1,
            Right => from + 1,
        })
    }

    /// Gets the tile value at a specific position
    ///
    /// # Arguments
    ///
    /// * `p` - The position to query (0-8)
    ///
    /// # Returns
    ///
    /// The tile number (0-7) at the specified position
    ///
    /// # Panics
    ///
    /// Panics if the position doesn't contain a valid tile
    fn get_value(&self, p: u32) -> u32 {
        let mut target_val = 0;
        let mut target_pos = 0;

        for val in 0..(BOARD_AREA - 1) {
            target_val = val as u32;
            target_pos =
                (self.0.unbounded_shr((TILE_BIT_SIZE * val) as u32)) % (1 << TILE_BIT_SIZE);
            if target_pos == p {
                break;
            }
        }

        if target_pos != p {
            panic!("Invalid move: cannot move space in that direction");
        }

        target_val
    }

    /// Sets a tile value at a specific position in the compact representation
    ///
    /// # Arguments
    ///
    /// * `p` - The position to place the tile at (0-8)
    /// * `val` - The tile number (0-7) to place
    fn set_value(&mut self, p: u32, val: u32) {
        let ones = (1 << TILE_BIT_SIZE) - 1;
        let mask = ones << (TILE_BIT_SIZE as u32 * val);
        self.0 &= !mask;
        self.0 |= p << (TILE_BIT_SIZE as u32 * val);
    }

    /// Moves the empty space in the specified direction
    ///
    /// This effectively moves a tile into the empty space, creating a new board state.
    /// The operation is performed on a copy of the board, leaving the original unchanged.
    ///
    /// # Arguments
    ///
    /// * `direction` - The direction to move the empty space
    ///
    /// # Returns
    ///
    /// `Ok(new_board)` if the move is valid, or an error message if the move is invalid
    pub fn move_space(mut self, direction: Direction) -> Result<Self, &'static str> {
        let space_position = self.find_space_position();
        let space_new_position = Self::calculate_new_position(space_position, direction)?;
        let digit_to_move = self.get_value(space_new_position);

        self.set_value(space_position, digit_to_move);

        Ok(self)
    }
}

/// Default implementation creates a solved board state
impl Default for Board {
    fn default() -> Self {
        Board(SOLVED_BOARD)
    }
}

/// Display implementation for pretty-printing the board
///
/// Displays the board as a 3x3 grid with numbers 1-8 and empty space
/// represented by three spaces.
impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let arr = self.into_arr();
        for (i, val) in arr.iter().enumerate() {
            if i % BOARD_SIDE == 0 && i != 0 {
                writeln!(f)?;
            }

            if *val != 0 {
                write!(f, "{:2} ", val)?;
            } else {
                write!(f, "   ")?;
            }
        }
        Ok(())
    }
}
