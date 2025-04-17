use bitvec::{order::Lsb0, view::BitView};

use crate::bitboard::*;
use std::{
    collections::HashMap,
    fmt::{self},
    ops::BitOr,
};

const FEN_NR_OF_PARTS: usize = 6;
const LIST_OF_PIECES: &str = "kqrbnpKQRBNP";
const WHITE_OR_BLACK: &str = "wb";
const SPLITTER: char = '/';
const DASH: char = '-';
const EM_DASH: char = '–';
const SPACE: char = ' ';

// Returns a table of the distance to the edges of the board for every square where index 0 of a square's table is the distance to the top, 1 is bottom, 2 is right, 3 is left, 4 is topright, 5 is bottomright, 6 is bottomleft, 7 is topleft.
pub fn compute_edges() -> Vec<Vec<usize>> {
    let mut square_list: Vec<Vec<usize>> = vec![vec![0; 8]; 64];

    for square_pos in 0..square_list.len() {
        let rank = square_pos.div_floor(8);
        let file = square_pos % 8;

        let top_dist = 7 - rank;
        let bottom_dist = rank;
        let left_dist = 7 - file;
        let right_dist = file;

        square_list[square_pos] = vec![
            top_dist,
            bottom_dist,
            right_dist,
            left_dist,
            top_dist.min(right_dist),
            bottom_dist.min(right_dist),
            top_dist.min(left_dist),
            bottom_dist.min(left_dist),
        ];
    }

    square_list
}

#[derive(Debug)]
pub enum FENErr {
    BadState,
    BadTeam,
    MalformedNumber,
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
                write!(f, "Team char is not either 'b' or 'w'\n")
            }
            Self::MalformedNumber => {
                write!(f, "Turn/halfmove clock characters malformed\n")
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
    pub piece_list: Vec<PieceType>,
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
            en_passant_move: None,
            piece_list: vec![PieceType::None; 64], // TODO: Make this compatible with any amount of squares/any size of map. Maybe as a type argument to the board state?
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
                                result_obj.board_pieces[team as usize][PieceType::King as usize]
                                    .state
                                    .view_bits_mut::<Lsb0>()
                                    .set(square, true);
                                result_obj.board_pieces[Team::Both as usize]
                                    [PieceType::King as usize]
                                    .state
                                    .view_bits_mut::<Lsb0>()
                                    .set(square, true);
                            }
                            'q' => {
                                result_obj.board_pieces[team as usize][PieceType::Queen as usize]
                                    .state
                                    .view_bits_mut::<Lsb0>()
                                    .set(square, true);
                                result_obj.board_pieces[Team::Both as usize]
                                    [PieceType::Queen as usize]
                                    .state
                                    .view_bits_mut::<Lsb0>()
                                    .set(square, true);
                            }
                            'p' => {
                                result_obj.board_pieces[team as usize][PieceType::Pawn as usize]
                                    .state
                                    .view_bits_mut::<Lsb0>()
                                    .set(square, true);
                                result_obj.board_pieces[Team::Both as usize]
                                    [PieceType::Pawn as usize]
                                    .state
                                    .view_bits_mut::<Lsb0>()
                                    .set(square, true);
                            }
                            'b' => {
                                result_obj.board_pieces[team as usize][PieceType::Bishop as usize]
                                    .state
                                    .view_bits_mut::<Lsb0>()
                                    .set(square, true);
                                result_obj.board_pieces[Team::Both as usize]
                                    [PieceType::Bishop as usize]
                                    .state
                                    .view_bits_mut::<Lsb0>()
                                    .set(square, true);
                            }
                            'r' => {
                                result_obj.board_pieces[team as usize][PieceType::Rook as usize]
                                    .state
                                    .view_bits_mut::<Lsb0>()
                                    .set(square, true);
                                result_obj.board_pieces[Team::Both as usize]
                                    [PieceType::Rook as usize]
                                    .state
                                    .view_bits_mut::<Lsb0>()
                                    .set(square, true);
                            }
                            'n' => {
                                result_obj.board_pieces[team as usize][PieceType::Knight as usize]
                                    .state
                                    .view_bits_mut::<Lsb0>()
                                    .set(square, true);
                                result_obj.board_pieces[Team::Both as usize]
                                    [PieceType::Knight as usize]
                                    .state
                                    .view_bits_mut::<Lsb0>()
                                    .set(square, true);
                            }
                            '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {
                                if let Some(empty_spaces) = char.to_digit(10) {
                                    if char != '8' && (file as usize + empty_spaces as usize) != 8 {
                                        file =
                                            CHESS_FILE_ARRAY[file as usize + empty_spaces as usize]
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
                            _ => return Err(FENErr::BadState),
                        };

                        if LIST_OF_PIECES.contains(char) && (file as i32 + 1) < 8 {
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
                        return Err(FENErr::BadTeam);
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
                }
                5 => {
                    if let Ok(hm_turn_clk) = fen_part.parse::<i64>() {
                        result_obj.fifty_move_clock = hm_turn_clk
                    } else {
                        return Err(FENErr::MalformedNumber);
                    }
                }
                6 => {
                    if let Ok(turn_clk) = fen_part.parse::<i64>() {
                        result_obj.turn_clock = turn_clk
                    } else {
                        return Err(FENErr::MalformedNumber);
                    }
                }
                _ => unreachable!("FEN data has more than seven parts"),
            }
        }

        result_obj.init_piece_list();
        Ok(result_obj)
    }
    fn init_piece_list(&mut self) {
        let white_bits = &self.board_pieces[Team::White as usize];
        let black_bits = &self.board_pieces[Team::Black as usize];

        for (piece_type, (blackboard, whiteboard)) in
            white_bits.iter().zip(black_bits.iter()).enumerate()
        {
            let piece_type = PIECE_TYPE_ARRAY[piece_type];

            // Iterate over each bit and table it
            let mut bit_index = 0;
            for black_bit in blackboard.state.view_bits::<Lsb0>() {
                let square = black_bit.then(|| piece_type);
                if let Some(square_piece) = square {
                    self.piece_list[bit_index] = square_piece;
                }
                bit_index += 1;
            }

            let mut bit_index = 0;
            for white_bit in whiteboard.state.view_bits::<Lsb0>() {
                let square = white_bit.then(|| piece_type);
                if let Some(square_piece) = square {
                    self.piece_list[bit_index] = square_piece;
                }
                bit_index += 1;
            }
        }
    }
    pub fn render_piece_list(&self) {
        print!("  a b c d e f g h");

        let display_map = HashMap::from([
            (PieceType::None, "O"),
            (PieceType::Pawn, "♙"),
            (PieceType::Bishop, "♗"),
            (PieceType::Knight, "♘"),
            (PieceType::Rook, "♖"),
            (PieceType::Queen, "♕"),
            (PieceType::King, "♔"),
        ]);
        let pl = &self.piece_list;
        
        for rank in (0..8).rev() {
            print!("\n{} ", rank + 1);

            for file in 0..8 {
                let bit_opt = pl[
                    rank * 8 + file
                ];
                print!("{} ", display_map.get(&bit_opt).unwrap());
            }
        }
        print!("\n");
    }

    pub fn get_team_coverage(&self, team: Team) -> Bitboard {
        let mut result = Bitboard::default();

        for piece_board in &self.board_pieces[team as usize] {
            // Apply all the piece tables to the base bitboard
            result |= *piece_board
        }

        result
    }
}
