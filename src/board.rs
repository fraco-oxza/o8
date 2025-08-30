//! # Board Module
//!
//! This module contains the implementation of the 8-puzzle board and related functionality.
//! The board is represented as a compact 32-bit integer where each tile position is encoded
//! using 4 bits, allowing for efficient storage and manipulation.
//!
//! ## Board Encoding Strategy
//!
//! The key insight is that instead of storing "what number is at each position",
//! we store "at what position is each number". This allows us to:
//!
//! 1. **Compact Representation**: Use only 32 bits for the entire board state
//! 2. **Fast Operations**: Bitwise operations for moves and comparisons  
//! 3. **Implicit Empty Space**: The missing position automatically represents the empty space
//!
//! ### Encoding Details
//!
//! - Each tile (1-8) gets 4 bits to store its position (0-8)
//! - Tile 1's position is stored in bits 0-3
//! - Tile 2's position is stored in bits 4-7
//! - And so on...
//! - The empty space position is found by determining which position (0-8) is not occupied
//!
//! ### Example Encoding
//!
//! For a solved board:
//! ```text
//! 1 2 3
//! 4 5 6  
//! 7 8  
//! ```
//!
//! The encoding would be:
//! - Tile 1 at position 0 → bits 0-3: 0000
//! - Tile 2 at position 1 → bits 4-7: 0001  
//! - Tile 3 at position 2 → bits 8-11: 0010
//! - Tile 4 at position 3 → bits 12-15: 0011
//! - Tile 5 at position 4 → bits 16-19: 0100
//! - Tile 6 at position 5 → bits 20-23: 0101
//! - Tile 7 at position 6 → bits 24-27: 0110
//! - Tile 8 at position 7 → bits 28-31: 0111
//! - Empty space at position 8 (implicit)
//!
//! This gives us the magic number: `SOLVED_BOARD = 1985229328`

use std::{cmp::Ordering, fmt::Display, sync::LazyLock};

use colored::Colorize;
use rand::{rng, seq::IndexedRandom};

use Direction::{Down, Left, Right, Up};

/// Array containing all possible movement directions
pub const ALL_DIRECTIONS: [Direction; 4] = [Up, Down, Left, Right];

#[rustfmt::skip]
const SOLVED_BOARD: [u8; BOARD_AREA as usize] = [
    1, 2, 3,
    8, 0, 4,
    7, 6, 5
];

/// The solved board state represented as a 32-bit integer
static SOLVED_BOARD_ENCODED: LazyLock<u32> = LazyLock::new(|| Board::from_arr(&SOLVED_BOARD).0);

/// The side length of the square board (3x3 grid)
const BOARD_SIDE: u8 = 3;

/// The total number of positions on the board (9 positions)
const BOARD_AREA: u8 = BOARD_SIDE * BOARD_SIDE;

/// Number of bits used to represent each tile position
const TILE_BIT_SIZE: u8 = 4;

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
/// ## Compact 32-bit Encoding
///
/// The board is stored as a compact 32-bit integer where each tile's position
/// is encoded using 4 bits. This representation is based on the insight that
/// instead of storing "what tile is at each position", we store "at what position
/// is each tile".
///
/// ### Why This Encoding?
///
/// 1. **Memory Efficient**: Only 32 bits instead of 36+ bytes for arrays
/// 2. **Copy Efficient**: Single integer copy instead of array copy
/// 3. **Hash Friendly**: Perfect for HashMap/HashSet keys
/// 4. **Cache Friendly**: Fits in a single cache line
/// 5. **Implicit Empty Space**: No need to explicitly track empty position
///
/// ### Bit Layout
///
/// ```text
/// Bits:  31-28  27-24  23-20  19-16  15-12  11-8   7-4    3-0
/// Tile:    8      7      6      5      4     3      2      1
/// Value: pos8   pos7   pos6   pos5   pos4  pos3   pos2   pos1
/// ```
///
/// Each 4-bit field stores the position (0-8) where that tile is located.
///
/// ### Example: Solved State
///
/// ```text
/// Board Layout:     Binary Encoding:
/// 1 2 3            Tile 1 at pos 0: 0000
/// 4 5 6            Tile 2 at pos 1: 0001
/// 7 8               Tile 3 at pos 2: 0010
///                  Tile 4 at pos 3: 0011
/// Positions:       Tile 5 at pos 4: 0100
/// 0 1 2            Tile 6 at pos 5: 0101
/// 3 4 5            Tile 7 at pos 6: 0110
/// 6 7 8            Tile 8 at pos 7: 0111
///                  Empty at pos 8:  (implicit)
/// ```
///
/// This produces: `0111_0110_0101_0100_0011_0010_0001_0000` = 1985229328
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

        for _ in 0..steps {
            let direction = *ALL_DIRECTIONS
                .choose(&mut rng)
                .expect("This should never happen");

            if let Ok(b) = board.move_space(direction) {
                board = b;
            }
        }

        board
    }

    /// Converts the compact board representation to a 2D array format
    ///
    /// This function reverses the encoding process: instead of storing where each
    /// tile is located, it reconstructs what tile is at each position.
    ///
    /// ## Algorithm
    ///
    /// 1. Initialize array with zeros (empty positions)
    /// 2. For each tile (1-8):
    ///    - Extract its position from the 4-bit field
    ///    - Place the tile number at that position in the array
    /// 3. Position 8 remains 0 (empty space) if not occupied
    ///
    /// ## Example
    ///
    /// For encoded value `1985229328`:
    /// ```text
    /// Tile 1 → Extract bits 0-3:   0000 = position 0
    /// Tile 2 → Extract bits 4-7:   0001 = position 1  
    /// Tile 3 → Extract bits 8-11:  0010 = position 2
    /// ...
    /// Result: [1, 2, 3, 4, 5, 6, 7, 8, 0]
    /// ```
    ///
    /// # Returns
    ///
    /// A 9-element array where each position contains the tile number,
    /// with 0 representing the empty space
    fn into_arr(self) -> [u8; BOARD_AREA as usize] {
        let mut arr = [0; BOARD_AREA as usize];

        // For each tile (0-7 representing tiles 1-8)
        for val in 0..(BOARD_AREA - 1) {
            // Extract the 4-bit position field for this tile
            let pos = self.get_pos(val);
            // Place the tile number (val + 1) at its encoded position
            arr[pos as usize] = val + 1;
        }

        arr
    }

    /// Creates a board from a 2D array representation
    ///
    /// This function encodes the array format back into the compact 32-bit representation.
    /// It assumes that the input array is valid (contains numbers 0-8 with no duplicates).
    ///
    /// # Arguments
    ///
    /// * `arr` - A reference to a 9-element array representing the board state
    ///
    /// # Returns
    ///
    /// The encoded `Board` instance
    pub fn from_arr(arr: &[u8; BOARD_AREA as usize]) -> Self {
        let mut board = Board(0);

        for (pos, &val) in arr.iter().enumerate() {
            if val != 0 {
                board.set_value(pos.try_into().expect("Should be less than 256"), val - 1);
            }
        }

        board
    }

    /// Checks if the board is in the solved state
    ///
    /// # Returns
    ///
    /// `true` if the board is solved (tiles are in numerical order), `false` otherwise
    pub fn is_solved(self) -> bool {
        self.0 == *SOLVED_BOARD_ENCODED
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
    fn is_valid_movement(position: u8, direction: Direction) -> bool {
        match direction {
            Up => (position / BOARD_SIDE) != 0,
            Down => (position / BOARD_SIDE) != BOARD_SIDE - 1,
            Left => (position % BOARD_SIDE) != 0,
            Right => (position % BOARD_SIDE) != BOARD_SIDE - 1,
        }
    }

    /// Finds the current position of the empty space on the board
    ///
    /// Since we only store positions for tiles 1-8, the empty space is implicitly
    /// the position that's NOT occupied by any tile.
    ///
    /// ## Algorithm
    ///
    /// 1. Create a bitmask representing all occupied positions
    /// 2. For each tile, set the bit corresponding to its position
    /// 3. Find the first unset bit (0-8) - this is the empty space
    ///
    /// ## Example
    ///
    /// If tiles occupy positions [0,1,2,3,4,5,6,7]:
    /// ```text
    /// Bitmask: 011111111 (positions 0-7 occupied)
    /// Missing: position 8 → empty space at position 8
    /// ```
    ///
    /// ## Why This Works
    ///
    /// - We have 9 positions (0-8) and 8 tiles
    /// - Each tile occupies exactly one position  
    /// - The unoccupied position is where the empty space is
    /// - `trailing_ones()` finds the first 0 bit, which is our answer
    ///
    /// # Returns
    ///
    /// The position (0-8) of the empty space
    fn find_space_position(self) -> u8 {
        let mut idx: u32 = 0;

        // Build bitmask of occupied positions
        for val in 0..(BOARD_AREA - 1) {
            let pos = self.get_pos(val);
            idx |= 1 << pos; // Set bit at position 'pos'
        }

        // Find first unset bit (empty position)
        idx.trailing_ones()
            .try_into()
            .expect("Should be less than 256")
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
    fn calculate_new_position(from: u8, direction: Direction) -> Result<u8, &'static str> {
        if !Board::is_valid_movement(from, direction) {
            return Err("Invalid move: cannot move space in that direction");
        }

        Ok(match direction {
            Up => from - BOARD_SIDE,
            Down => from + BOARD_SIDE,
            Left => from - 1,
            Right => from + 1,
        })
    }

    /// Gets the tile value at a specific position
    ///
    /// This function searches through all tiles to find which one is located
    /// at the specified position. It's essentially the inverse of the encoding.
    ///
    /// ## Algorithm
    ///
    /// 1. Iterate through all tiles (1-8, represented as 0-7 internally)
    /// 2. For each tile, extract its encoded position
    /// 3. If the position matches what we're looking for, return the tile number
    /// 4. If no tile is found at that position, it means the position is empty
    ///
    /// ## Example
    ///
    /// To find what's at position 2:
    /// ```text
    /// Check tile 1: position = bits 0-3   → if == 2, return 0 (tile 1)
    /// Check tile 2: position = bits 4-7   → if == 2, return 1 (tile 2)  
    /// Check tile 3: position = bits 8-11  → if == 2, return 2 (tile 3) ✓
    /// ```
    ///
    /// # Arguments
    ///
    /// * `p` - The position to query (0-8)
    ///
    /// # Returns
    ///
    /// The tile number (0-7 representing tiles 1-8) at the specified position
    ///
    /// # Panics
    ///
    /// Panics if the position doesn't contain a valid tile (i.e., it's the empty space)
    fn get_value(self, p: u8) -> u8 {
        let mut target_val = 0;
        let mut target_pos = 0;

        // Search through all tiles to find which one is at position 'p'
        for val in 0..(BOARD_AREA - 1) {
            target_val = val;
            target_pos = self.get_pos(val);
            if target_pos == p {
                break; // Found the tile at position 'p'
            }
        }

        assert!(
            target_pos == p,
            "Invalid move: cannot move space in that direction"
        );

        target_val // Return tile number (0-7)
    }

    /// Sets a tile value at a specific position in the compact representation
    ///
    /// This function updates the encoding to move a tile to a new position.
    /// It modifies the 4-bit field corresponding to the specified tile.
    ///
    /// ## Algorithm
    ///
    /// 1. Create a mask to clear the old position for tile `val`
    /// 2. Clear the 4-bit field for tile `val` using bitwise AND with inverted mask
    /// 3. Set the new position using bitwise OR
    ///
    /// ## Example
    ///
    /// To move tile 3 (val=2) to position 5 (p=5):
    /// ```text
    /// 1. Create mask for bits 8-11:  mask = 1111_0000_0000
    /// 2. Clear old position:         board &= !mask
    /// 3. Set new position:           board |= (5 << 8)
    /// ```
    ///
    /// ## Bit Operations Breakdown
    ///
    /// - `(1 << TILE_BIT_SIZE) - 1` creates mask `1111`
    /// - `mask << (TILE_BIT_SIZE * val)` positions the mask at tile's field
    /// - `&= !mask` clears the old value
    /// - `|= p << (TILE_BIT_SIZE * val)` sets the new position
    ///
    /// # Arguments
    ///
    /// * `p` - The position to place the tile at (0-8)
    /// * `val` - The tile number (0-7 representing tiles 1-8) to place
    fn set_value(&mut self, p: u8, val: u8) {
        // Create 4-bit mask: 0000_1111
        let ones = (1 << TILE_BIT_SIZE) - 1;
        // Position mask at tile's bit field
        let mask = ones << (TILE_BIT_SIZE * val);
        // Clear old position
        self.0 &= !mask;
        // Set new position
        self.0 |= u32::from(p) << (TILE_BIT_SIZE * val);
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

    fn get_pos(self, value: u8) -> u8 {
        let offset = TILE_BIT_SIZE * value;
        (self.0.unbounded_shr(u32::from(offset)) % (1 << TILE_BIT_SIZE))
            .try_into()
            .expect("TILE_BIT_SIZE should be less than 8")
    }

    pub fn heuristic_distance_to_solution(self) -> u8 {
        let solution = Self::default();
        let mut distance = 0;

        for val in 0..(BOARD_AREA - 1) {
            distance += Self::manhattan_distance(solution.get_pos(val), self.get_pos(val));
        }

        distance
        // + Self::manhattan_distance(solution.find_space_position(), self.find_space_position())
    }

    fn manhattan_distance(pos1: u8, pos2: u8) -> u8 {
        let hdis = (pos2 % BOARD_SIDE).abs_diff(pos1 % BOARD_SIDE);
        let vdis = (pos2 / BOARD_SIDE).abs_diff(pos1 / BOARD_SIDE);

        hdis + vdis
    }
}

impl PartialOrd for Board {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Board {
    fn cmp(&self, other: &Self) -> Ordering {
        self.heuristic_distance_to_solution()
            .cmp(&other.heuristic_distance_to_solution())
    }
}

/// Default implementation creates a solved board state
impl Default for Board {
    fn default() -> Self {
        Board(*SOLVED_BOARD_ENCODED)
    }
}

/// Display implementation for pretty-printing the board
///
/// Displays the board as a 3x3 grid with numbers 1-8 and empty space
/// represented by three spaces.
impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let arr = self.into_arr().into_iter();
        let target = Board::default().into_arr().into_iter();

        for (i, (val, is_in_position)) in arr.zip(target).map(|(a, t)| (a, a == t)).enumerate() {
            if i % BOARD_SIDE as usize == 0 && i != 0 {
                writeln!(f)?;
            }

            if val != 0 {
                let s = format!("{val:2} ");
                if is_in_position {
                    write!(f, "{}", s.green().bold())?;
                } else {
                    write!(f, "{}", s.red())?;
                }
            } else {
                write!(f, "   ")?;
            }
        }

        Ok(())
    }
}

/// Board annotated with the number of steps taken to reach it (g-cost).
///
/// When ordered, it uses `heuristic_distance_to_solution() + steps` which
/// allows a priority queue to behave like A* with an admissible heuristic.
#[derive(PartialEq, Eq, Default, Clone)]
pub struct BoardWithSteps(pub Board, pub usize);

impl PartialOrd for BoardWithSteps {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BoardWithSteps {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.0.heuristic_distance_to_solution() as usize + self.1)
            .cmp(&(other.0.heuristic_distance_to_solution() as usize + other.1))
    }
}
