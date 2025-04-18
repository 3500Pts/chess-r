use std::{
    fmt::{self},
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not},
};

// bitboard.rs
use bitvec::prelude::*;

#[derive(Debug, Hash, Clone, Copy, Eq, PartialEq)]
pub enum Team {
    White = 0,
    Black = 1,
    Both = 2,
    Red = 3,
    None = 4
}

#[derive(Debug, Hash, Clone, Copy, Eq, PartialEq)]
pub enum PieceType {
    None = 0,
    Pawn = 1,
    Rook = 2,
    Bishop = 3,
    Knight = 4,
    Queen = 5,
    King = 6,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum ChessFile {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
}
pub const CHESS_FILE_ARRAY: [ChessFile; 8] = [
    ChessFile::A,
    ChessFile::B,
    ChessFile::C,
    ChessFile::D,
    ChessFile::E,
    ChessFile::F,
    ChessFile::G,
    ChessFile::H,
];
pub const PIECE_TYPE_ARRAY: [PieceType; 7] = [
    PieceType::None,
    PieceType::Pawn,
    PieceType::Rook,
    PieceType::Bishop,
    PieceType::Knight,
    PieceType::Queen,
    PieceType::King,
];

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Bitboard {
    pub state: u64,
}
impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\n  a b c d e f g h")?;

        let state_slice = self.state.view_bits::<Lsb0>();
        for rank in (0..8).rev() {
            write!(f, "\n{} ", rank + 1)?;

            for file in 0..8 {
                let square_idx = (rank * 8) + file;
                let bit_opt = state_slice.get(square_idx);
                if let Some(bit) = bit_opt {
                    let string = String::from("");

                    // Leave the square_idx if statement for easy testing of what index maps to what position
                    write!(
                        f,
                        "{string}{} ",
                        bit.then(|| { if square_idx == 1 { "Z" } else { "X" } })
                            .unwrap_or("O")
                    )?;
                }
            }
        }
        write!(f, "")
    }
}
impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        return Bitboard {
            state: self.state | rhs.state,
        };
    }
}
impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.state |= rhs.state
    }
}
impl Not for Bitboard {
    type Output = Self;
    fn not(self) -> Self::Output {
        return Bitboard {
            state: !self.state 
        };
    }
}
impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        return Bitboard {
            state: self.state & rhs.state,
        };
    }
}
impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.state &= rhs.state
    }
}
impl Default for Bitboard {
    fn default() -> Self {
        Bitboard { state: 0 }
    }
}

impl Bitboard {
    pub fn al_notation_to_bit_idx(notation: &str) -> Option<usize> {
        let list = ["a", "b", "c", "d", "e", "f", "g", "h"];

        let split: Vec<char> = notation.chars().collect();

        let rank = list.iter().position(|n| *n == split[0].to_string());

        if let Some(rank_id) = rank {
            let file = split[1].to_digit(10);
            if let Some(file_id) = file {
                let result = ((rank_id as usize) * 8) + (file_id as usize);
                return Some(result);
            } else {
                None
            }
        } else {
            None
        }
    }
}
