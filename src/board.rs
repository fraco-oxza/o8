use std::fmt::Display;

use rand::{rng, seq::IndexedRandom};

use Direction::*;

pub const ALL_DIRECTIONS: [Direction; 4] = [Up, Down, Left, Right];
const SOLVED_BOARD: u32 = 1985229328;

#[derive(Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Board(u32);

impl Board {
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

    fn into_arr(self) -> [u8; 9] {
        let bits = self.0;
        let mut arr = [0b0; 9];

        for val in 0..8 {
            let pos = (bits.unbounded_shr(4 * val)) % 16;
            arr[pos as usize] = (val + 1) as u8;
        }

        arr
    }

    pub fn is_solved(&self) -> bool {
        self.0 == SOLVED_BOARD
    }

    fn is_valid_movement(position: u32, direction: Direction) -> bool {
        match direction {
            Up => (position / 3) != 0,
            Down => (position / 3) != 2,
            Left => (position % 3) != 0,
            Right => (position % 3) != 2,
        }
    }

    fn find_space_position(&self) -> u32 {
        let mut idx: u32 = 0;

        for val in 0..8 {
            let pos = (self.0.unbounded_shr(4 * val)) % 16;
            idx |= 1 << pos;
        }

        idx.trailing_ones()
    }

    fn calculate_new_position(from: u32, direction: Direction) -> Result<u32, &'static str> {
        if !Board::is_valid_movement(from, direction) {
            return Err("Invalid move: cannot move space in that direction");
        }

        Ok(match direction {
            Up => from - 3,
            Down => from + 3,
            Left => from - 1,
            Right => from + 1,
        })
    }

    fn get_value(&self, p: u32) -> u32 {
        let mut target_val = 0;
        let mut target_pos = 0;
        for val in 0..8 {
            target_val = val;
            target_pos = (self.0.unbounded_shr(4 * val)) % 16;
            if target_pos == p {
                break;
            }
        }

        if target_pos != p {
            panic!("Invalid move: cannot move space in that direction");
        }

        target_val
    }

    fn set_value(&mut self, p: u32, val: u32) {
        let mask = 0b1111 << (4 * val);
        self.0 &= !mask;
        self.0 |= p << (4 * val);
    }

    pub fn move_space(mut self, direction: Direction) -> Result<Self, &'static str> {
        let space_position = self.find_space_position();
        let space_new_position = Self::calculate_new_position(space_position, direction)?;
        let digit_to_move = self.get_value(space_new_position);

        self.set_value(space_position, digit_to_move);

        Ok(self)
    }
}

impl Default for Board {
    fn default() -> Self {
        Board(SOLVED_BOARD)
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let arr = self.into_arr();
        for (i, val) in arr.iter().enumerate() {
            if i % 3 == 0 && i != 0 {
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
