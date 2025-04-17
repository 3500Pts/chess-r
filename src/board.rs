use bitvec::{order::Lsb0, view::BitView};

use crate::bitboard::*;
use std::fmt::{self};

const FEN_NR_OF_PARTS: usize = 6;
const LIST_OF_PIECES: &str = "kqrbnpKQRBNP";
const WHITE_OR_BLACK: &str = "wb";
const SPLITTER: char = '/';
const DASH: char = '-';
const EM_DASH: char = '–';
const SPACE: char = ' ';

#[derive(Debug)]
pub enum FENErr {
    BadState,
    BadTeam,
    MalformedNumber
}
impl fmt::Display for FENErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BadState => {
                write!(
                    f,
                    "Bad character exists in the state section of FEN string\n"
                )
            }
            Self::BadTeam => {
                write!(
                    f,
                    "Team char is not either 'b' or 'w'\n"
                )
            }
            Self::MalformedNumber => {
                write!(
                    f,
                    "Turn/halfmove clock characters malformed\n"
                )
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CastlingRights {
    pub queen: bool,
    pub king: bool,
}

#[derive(Debug, Clone)]
pub struct BoardState {
    pub board_pieces: Vec<Vec<Bitboard>>,
    pub to_move: Team,
    pub castling_rights: Vec<CastlingRights>,
    pub fifty_move_clock: i64,
    pub en_passant_move: Option<u64>,
    pub turn_clock: i64,
}
impl Default for BoardState {
    fn default() -> Self {
        BoardState {
            board_pieces: vec![vec![Bitboard { state: 0 }; 7]; 3],
            to_move: Team::White,
            castling_rights: vec![
                CastlingRights {
                    queen: false,
                    king: false
                };
                2
            ],
            fifty_move_clock: 0,
            turn_clock: 1,
            en_passant_move: None
        }
    }
}
impl BoardState {
    pub fn from_fen(fen: String) -> Result<Self, FENErr> {
        let mut fen_part_idx = 0;

        let mut rank = 7;
        let mut file = ChessFile::A;

        let mut result_obj = BoardState::default();

        for fen_part in fen.split(" ") {
            fen_part_idx += 1;
            match fen_part_idx {
                1 => {
                    for char in fen_part.chars() {
    
                        let team = if char.is_lowercase() {
                            Team::Black
                        } else {
                            Team::White
                        };

                        let square: usize = ((rank as usize) * 8) + file as usize;
                        match char.to_ascii_lowercase() {
                            'k' => {
                                result_obj.board_pieces[team as usize][PieceType::King as usize].state.view_bits_mut::<Lsb0>().set(square, true);
                                result_obj.board_pieces[Team::Both as usize][PieceType::King as usize].state.view_bits_mut::<Lsb0>().set(square, true);
                            }
                            'q' => {
                                result_obj.board_pieces[team as usize][PieceType::Queen as usize].state.view_bits_mut::<Lsb0>().set(square, true);
                                result_obj.board_pieces[Team::Both as usize][PieceType::Queen as usize].state.view_bits_mut::<Lsb0>().set(square, true);
                            }
                            'p' => {
                                result_obj.board_pieces[team as usize][PieceType::Pawn as usize].state.view_bits_mut::<Lsb0>().set(square, true);
                                result_obj.board_pieces[Team::Both as usize][PieceType::Pawn as usize].state.view_bits_mut::<Lsb0>().set(square, true);
                            }
                            'b' => {
                                result_obj.board_pieces[team as usize][PieceType::Bishop as usize].state.view_bits_mut::<Lsb0>().set(square, true);
                                result_obj.board_pieces[Team::Both as usize][PieceType::Bishop as usize].state.view_bits_mut::<Lsb0>().set(square, true);
                            }
                            'r' => {
                                result_obj.board_pieces[team as usize][PieceType::Rook as usize].state.view_bits_mut::<Lsb0>().set(square, true);
                                result_obj.board_pieces[Team::Both as usize][PieceType::Rook as usize].state.view_bits_mut::<Lsb0>().set(square, true);
                            }
                            'n' => {
                                result_obj.board_pieces[team as usize][PieceType::Knight as usize].state.view_bits_mut::<Lsb0>().set(square, true);
                                result_obj.board_pieces[Team::Both as usize][PieceType::Knight as usize].state.view_bits_mut::<Lsb0>().set(square, true);
                            }
                            '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {
                                if let Some(empty_spaces) = char.to_digit(10) {
                                    if char != '8' && (file as usize + empty_spaces as usize) != 8 {
                                        file = CHESS_FILE_ARRAY[file as usize + empty_spaces as usize]
                                    } else {
                                        // do nothing and skip to the next rank
                                        file = ChessFile::H;
                                    }
                                }
                            }
                            SPLITTER => {
                                if file != ChessFile::H {
                                    return Err(FENErr::BadState);
                                }
                                rank -= 1;
                                file = ChessFile::A;
                            }
                            _ => { return Err(FENErr::BadState) },
                        };

                        if LIST_OF_PIECES.contains(char) && (file as i32 + 1) < 8{
                            if file as usize + 1 == 8 {
                                rank -= 1;
                                file = ChessFile::A;
                            } else {
                                file = CHESS_FILE_ARRAY[(file as usize) + 1]
                            }
                        }
                    }
                }
                2 => {
                    if fen_part.contains("b") {
                        result_obj.to_move = Team::Black
                    } else if fen_part.contains("w") {
                        result_obj.to_move = Team::White
                    } else {
                        return Err(FENErr::BadTeam)
                    }
                }
                3 => {
                    let mut black_rights = CastlingRights {
                        queen: false,
                        king: false,
                    };
                    let mut white_rights = CastlingRights {
                        queen: false,
                        king: false,
                    };
                    if fen_part.contains("K") {
                        white_rights.king = true
                    }
                    if fen_part.contains("Q") {
                        white_rights.queen = true
                    }
                    if fen_part.contains("k") {
                        black_rights.king = true
                    }
                    if fen_part.contains("q") {
                        black_rights.queen = true
                    }
                
                    if !fen_part.contains("-") {
                        result_obj.castling_rights[Team::White as usize] = white_rights;
                        result_obj.castling_rights[Team::Black as usize] = black_rights;
                    }
                }
                4 => {  
                    if fen_part.len() < 2 {
                        // Not enough to count this
                        result_obj.en_passant_move = None;
                    } else {
                        result_obj.en_passant_move = Bitboard::al_notation_to_bit_idx(fen_part)
                    }
                },
                5 => {
                    if let Ok(hm_turn_clk) = fen_part.parse::<i64>() {
                        result_obj.fifty_move_clock = hm_turn_clk
                    } else {
                        return Err(FENErr::MalformedNumber)
                    }
                },
                6 => {
                    if let Ok(turn_clk) = fen_part.parse::<i64>() {
                        result_obj.turn_clock = turn_clk
                    } else {
                        return Err(FENErr::MalformedNumber)
                    }
                },
                _ => unreachable!("FEN data has more than seven parts"),
            }
        }
        Ok(result_obj)
    }
}
