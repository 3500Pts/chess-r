use std::{
    fmt::{self},
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not},
};

// bitboard.rs
use bitvec::prelude::*;

use crate::{
    board::BoardState,
    r#move::{Move, Piece},
};

#[derive(Debug, Hash, Clone, Copy, Eq, PartialEq)]
pub enum Team {
    White = 0,
    Black = 1,
    Both = 2,
    Red = 3,
    None = 4,
}
impl Team {
    pub fn opponent(&self) -> Self {
        if self == &Self::Black {
            Team::White
        } else {
            Team::Black
        }
    }
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
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
                        bit.then(|| {
                            if square_idx == 1 {
                                "Z"
                            } else {
                                "X"
                            }
                        })
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
        Bitboard {
            state: self.state | rhs.state,
        }
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
        Bitboard { state: !self.state }
    }
}
impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard {
            state: self.state & rhs.state,
        }
    }
}
impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.state &= rhs.state
    }
}

impl Bitboard {
    pub fn al_notation_to_bit_idx(notation: &str) -> Option<usize> {
        let list = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

        let split: Vec<char> = notation.chars().collect();

        let file = list.iter().position(|n| n == &split[0]);

        if let Some(file_id) = file {
            let rank = split[1].to_digit(10);
            if let Some(rank_id) = rank {
                let result = ((rank_id - 1) * 8) + file_id as u32;
                Some(result as usize)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn bit_idx_to_al_notation(bit: usize) -> Option<String> {
        let list = ["a", "b", "c", "d", "e", "f", "g", "h"];

        if !(0..64).contains(&bit) {
            return None;
        }
        let rank_num = bit.div_floor(8);
        let file_num = bit % 8;

        let file_str = list[file_num];

        Some(format!("{}{}", file_str, rank_num + 1))
    }

    pub fn set_bit<O: BitOrder>(&mut self, index: usize, value: bool) {
        let bit_slice = self.state.view_bits_mut::<O>();

        let bits = 64;
        if index < bits {
            bit_slice.set(index, value);
        }
    }

    pub fn get_bit<Order: BitOrder>(&self, index: usize) -> bool {
        let bit_slice = self.state.view_bits::<Order>();

        let bits = 64;
        if index < bits {
            let bit_ref_option = bit_slice.get(index);
            if let Some(bit_ref) = bit_ref_option {
                return bit_ref.then_some(1).is_some();
            }
        }

        false
    }

    pub fn get_move_from_bit(
        bitboard: &Bitboard,
        enemy_bitboard: Bitboard,
        friendly_bitboard: Bitboard,
        square: usize,
        attacking_piece: &Piece,
        team: Team,
        board_state: &BoardState,
    ) -> Option<Move> {
        if bitboard.get_bit::<Lsb0>(square) {
            let capture = (enemy_bitboard & !friendly_bitboard).get_bit::<Lsb0>(square).then(|| {board_state.get_piece_at_pos(square).expect("Enemy bitboard should not have a positive bit where a piece does not exist")});
            let start = attacking_piece.position;

            return Some(Move {
                start: start,
                target: square,
                captures: capture,
                is_castle: attacking_piece.piece_type == PieceType::King
                    && square.abs_diff(start) == 2,
                is_pawn_double: attacking_piece.piece_type == PieceType::Pawn
                    && square.abs_diff(start) == 16,
            });
        } else {
            return None;
        }
    }
    pub fn as_moves(
        &self,
        attacking_piece: &Piece,
        team: Team,
        board_state: &BoardState,
    ) -> Vec<Move> {
        // Create move objs from all parts of the board
        let mut move_list: Vec<Move> = Vec::new();

        let range_end: usize = 64;
        let range_start: usize = 0;

        let friendly_bitboard = board_state.get_team_coverage(team);
        let enemy_bitboard = board_state.get_team_coverage(team.opponent());
        for square in range_start..range_end {
            if let Some(move_result) = Self::get_move_from_bit(
                self,
                enemy_bitboard,
                friendly_bitboard,
                square,
                attacking_piece,
                team,
                board_state,
            ) {
                move_list.push(move_result)
            }
        }
        move_list
    }
}

pub struct BitboardIterator {
    head: usize,
    board: Bitboard,
}
impl Iterator for BitboardIterator {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        self.head += 1;

        if self.head >= 64 {
            return None;
        } else {
            return Some(self.board.get_bit::<Lsb0>(self.head));
        }
    }
}
impl IntoIterator for Bitboard {
    type Item = bool;

    type IntoIter = BitboardIterator;

    fn into_iter(self) -> Self::IntoIter {
        return BitboardIterator {
            head: 0,
            board: self,
        };
    }
}
