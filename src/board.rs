use bitvec::{order::Lsb0, view::BitView};

use crate::{
    bitboard::*,
    r#move::{compute_knight, compute_pawn, compute_slider, Move, MoveError, Piece},
};
use std::{
    collections::HashMap,
    fmt::{self},
};

const LIST_OF_PIECES: &str = "kqrbnpKQRBNP";
const SPLITTER: char = '/';

// Returns a table of the distance to the edges of the board for every square where index 0 of a square's table is the distance to the top, 1 is bottom, 2 is right, 3 is left, 4 is topright, 5 is bottomright, 6 is bottomleft, 7 is topleft.
pub fn compute_edges() -> Vec<Vec<usize>> {
    let mut square_list: Vec<Vec<usize>> = vec![vec![0; 8]; 64];

    for square_pos in 0..square_list.len() {
        let rank = square_pos.div_floor(8);
        let file = square_pos % 8;

        let top_dist = 7 - rank;
        let bottom_dist = rank;
        let left_dist = file;
        let right_dist = 7 - file;

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

#[derive(Debug, Clone)]
pub struct BoardState {
    pub board_pieces: Vec<Vec<Bitboard>>,
    pub to_move: Team,
    pub castling_rights: u8, // Using queen, king, and each side as booleans, there are 4 bits of castling rights that can be expressed as a number
    pub fifty_move_clock: i64,
    pub en_passant_square: Option<usize>,
    pub turn_clock: i64,
    pub piece_list: Vec<PieceType>,
    pub edge_compute: Vec<Vec<usize>>,
    pub capture_bitboard: Vec<Bitboard>
}
impl Default for BoardState {
    fn default() -> Self {
        BoardState {
            board_pieces: vec![vec![Bitboard { state: 0 }; 7]; 3],
            to_move: Team::White,
            castling_rights: 0,
            fifty_move_clock: 0,
            turn_clock: 1,
            en_passant_square: None,
            piece_list: vec![PieceType::None; 64], // TODO: Make this compatible with any amount of squares/any size of map. Maybe as a type argument to the board state?
            edge_compute: compute_edges(),
            capture_bitboard: vec![Bitboard { state: 0 }; 2],
        }
    }
}
impl BoardState {
    // Constructs a board state from a FEN string
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
                        let team = if char.is_ascii_uppercase() {
                            Team::White
                        } else {
                            Team::Black
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
                    let mut rights: u8 = 0;
                    if fen_part.contains("K") {
                        rights.view_bits_mut::<Lsb0>().set(0, true);
                    }
                    if fen_part.contains("Q") {
                        rights.view_bits_mut::<Lsb0>().set(1, true);
                    }
                    if fen_part.contains("k") {
                        rights.view_bits_mut::<Lsb0>().set(2, true);
                    }
                    if fen_part.contains("q") {
                        rights.view_bits_mut::<Lsb0>().set(3, true);
                    }

                    if fen_part.contains("-") && rights > 0 { // TODO: Throw an error if we hit the 'else' arm and rights is not 0
                    }
                }
                4 => {
                    if fen_part.len() < 2 {
                        // Not enough to count this
                        result_obj.en_passant_square = None;
                    } else {
                        result_obj.en_passant_square = Bitboard::al_notation_to_bit_idx(fen_part)
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
        result_obj.update_capture_bitboards();
        Ok(result_obj)
    }

    // initializes piece lists based on the bitboards
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
       
    fn move_piece(&mut self, square_team: Team, moving_piece_type: PieceType, r#move: Move) {
        let board_pieces = &mut self.board_pieces;

        if square_team == Team::White {
            board_pieces[Team::White as usize][moving_piece_type as usize]
                .state
                .view_bits_mut::<Lsb0>()
                .set(r#move.start, false);
            board_pieces[Team::White as usize][moving_piece_type as usize]
                .state
                .view_bits_mut::<Lsb0>()
                .set(r#move.target, true);

            board_pieces[Team::Black as usize]
                .iter_mut()
                .for_each(|bb| {
                    bb.state.view_bits_mut::<Lsb0>().set(r#move.target, false);
                });
        } else {
            board_pieces[Team::Black as usize][moving_piece_type as usize]
                .state
                .view_bits_mut::<Lsb0>()
                .set(r#move.start, false);
            board_pieces[Team::Black as usize][moving_piece_type as usize]
                .state
                .view_bits_mut::<Lsb0>()
                .set(r#move.target, true);

            // Clear the slot for the piece - this resembles a capture
            board_pieces[Team::White as usize]
                .iter_mut()
                .for_each(|bb| {
                    bb.state.view_bits_mut::<Lsb0>().set(r#move.target, false);
                });
        }

        self.piece_list[r#move.start] = PieceType::None;
        self.piece_list[r#move.target] = moving_piece_type;
    }
    fn update_capture_bitboards(&mut self) {
        for team_id in 0..Team::Black as usize {
        let mut capture_bitboard = Bitboard::default();
        let legals = self.get_psuedolegal_moves(); // TODO: Make these legal moves

        for square in 0..64 {
            capture_bitboard &= legals[square].0;
        }
        self.capture_bitboard[team_id] = capture_bitboard;
    }
    }
    pub fn render_piece_list(pl: Vec<PieceType>) {
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
        for rank in (0..8).rev() {
            print!("\n{} ", rank + 1);

            for file in 0..8 {
                let bit_opt = pl[rank * 8 + file];
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
    pub fn get_psuedolegal_moves(&self) -> Vec<(Bitboard, Vec<Move>)> {
        let pl = self.piece_list.clone();
        let mut move_list: Vec<(Bitboard, Vec<Move>)> = Vec::new(); // The bitboard is used for highlighting moves the selected square has
        pl.iter().enumerate().for_each(|(index, piece_type)| {
            let default_push = (Bitboard::default(),
            vec![Move::default()]);

            let team_opt = self.get_square_team(index);
            if let Some(team) = team_opt {
                let piece_obj =  Piece {
                    piece_type: *piece_type,
                    position: index,
                    team: team,
                };
                let (psuedo_bitboard, psuedo_moves) = match piece_type {
                    PieceType::Bishop | PieceType::Rook | &PieceType::Queen | PieceType::King => compute_slider(
                        self,
                        piece_obj
                    ),
                    PieceType::Knight => compute_knight(
                        self, 
                        piece_obj
                    ),
                    PieceType::Pawn => compute_pawn(
                        self,
                        piece_obj
                    ),
                    _ => default_push
                };

                move_list.push((psuedo_bitboard, psuedo_moves));
            } else {
                move_list.push(default_push);
            }
        });

        move_list
    }
    pub fn get_square_team(&self, square_idx: usize) -> Option<Team> {
        let white_check = self.get_team_coverage(Team::White);
        let black_check = self.get_team_coverage(Team::Black);

        let square_team = {
            let white_bitcheck = white_check
                .state
                .view_bits::<Lsb0>()
                .get(square_idx)
                .expect("Index was not within bitboard")
                .then(|| Team::White);

            if white_bitcheck.is_none() {
                let black_bitcheck = black_check
                    .state
                    .view_bits::<Lsb0>()
                    .get(square_idx)
                    .expect("Index was not within bitboard")
                    .then(|| Team::Black);

                if let Some(_bbc) = black_bitcheck {
                    black_bitcheck
                } else {
                    // There is not a piece here.
                    None
                }
            } else {
                white_bitcheck
            }
        };
        square_team
    }
    pub fn make_move(&mut self, r#move: Move) -> Result<(), MoveError> {
        
        // Update out of the target positions
        let moving_piece_type = self.piece_list[r#move.start];
        let square_team_opt = self.get_square_team(r#move.start);
        let target_team_opt = self.get_square_team(r#move.target);

        if r#move.start == r#move.target {
            return Err(MoveError::NotAMove);
        }
        if square_team_opt == None {
            return Err(MoveError::NoUnit);
        }
        if target_team_opt == square_team_opt {
            return Err(MoveError::AttackedAlly);
        }
        if let Some(square_team) = square_team_opt {
            tracing::debug!("{square_team:?} {moving_piece_type:?} {move:?}");

            self.move_piece(square_team, moving_piece_type, r#move);
            self.en_passant_square = if (r#move.is_pawn_double) {Some(r#move.target)} else {self.en_passant_square};
            self.update_capture_bitboards();
        }
        return Ok(());
    }
    pub fn get_piece_at_pos(&self, pos: usize) -> Option<Piece> {
        
        let target_piece_type = self.piece_list[pos];

        let target_piece = if target_piece_type == PieceType::None { 
            Some(Piece {
                team: self.get_square_team(pos).unwrap_or(Team::None),
                position: pos,
                piece_type: target_piece_type
            })
        } else {
            None
        };

        target_piece
    }
}
