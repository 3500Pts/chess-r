use bitvec::{order::Lsb0, view::BitView};

use crate::{
    bitboard::*,
    r#move::{Move, MoveError, Piece, *},
};
use std::{
    collections::HashMap,
    fmt::{self},
};

const LIST_OF_PIECES: &str = "kqrbnpKQRBNP";
const SPLITTER: char = '/';

// Returns a table of the distance to the edges of the board for every square where index 0 of a square's table is the distance to the top, 1 is bottom, 2 is right, 3 is left, 4 is topright, 5 is bottomright, 6 is bottomleft, 7 is topleft.
pub fn compute_edges() -> [[usize; 8]; 64] {
    let mut square_list = [[0; 8]; 64];

    for (square_pos, entry) in square_list.iter_mut().enumerate() {
        let rank = square_pos.div_floor(8);
        let file = square_pos % 8;

        let top_dist = 7 - rank;
        let bottom_dist = rank;
        let left_dist = file;
        let right_dist = 7 - file;

        *entry = [
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
                writeln!(f, "Bad character exists in the state section of FEN string")
            }
            Self::BadTeam => {
                writeln!(f, "Team char is not either 'b' or 'w'")
            }
            Self::MalformedNumber => {
                writeln!(f, "Turn/halfmove clock characters malformed")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct BoardState {
    pub board_pieces: [[Bitboard; 7]; 3],
    pub castling_rights: u8, // Using queen, king, and each side as booleans, there are 4 bits of castling rights that can be expressed as a number
    pub fifty_move_clock: i64,
    pub en_passant_square: Option<usize>,
    pub turn_clock: i64,
    pub ply_clock: i64,
    pub active_team_checkmate: bool,
    pub piece_list: [PieceType; 64],
    pub edge_compute: [[usize; 8]; 64],
    pub king_compute: [[Bitboard; 64]; 2],
    pub capture_bitboard: [Bitboard; 2],
    pub en_passant_turn: Option<i64>,
    pub active_team: Team,
    pub pawn_attack_compute: [[Bitboard; 64]; 2],
    pub pawn_push_compute: [[Bitboard; 64]; 2],
    pub knight_compute: [Bitboard; 64],
}
impl Default for BoardState {
    fn default() -> Self {
        BoardState {
            board_pieces: [[Bitboard { state: 0 }; 7]; 3],
            castling_rights: 0,
            fifty_move_clock: 0,
            ply_clock: 0,
            turn_clock: 1,
            en_passant_square: None,
            en_passant_turn: None,
            active_team_checkmate: false,
            piece_list: [PieceType::None; 64], // TODO: Make this compatible with any amount of squares/any size of map. Maybe as a type argument to the board state?
            edge_compute: compute_edges(),
            king_compute: precalc_king_attack::<64>(),
            knight_compute: precalc_knight_attack::<64>(),
            pawn_attack_compute: precalc_pawn_attack::<64>(),
            pawn_push_compute: precalc_pawn_push::<64>(),
            capture_bitboard: [Bitboard { state: 0 }; 2],
            active_team: Team::White,
        }
    }
}
impl BoardState {
    /*
        Constructs a board state from a FEN string
    */
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
                        result_obj.active_team = Team::Black
                    } else if fen_part.contains("w") {
                        result_obj.active_team = Team::White
                    } else {
                        return Err(FENErr::BadTeam);
                    }
                }
                3 => {
                    let mut rights: u8 = 0;
                    if fen_part.contains('K') {
                        rights.view_bits_mut::<Lsb0>().set(0, true);
                    }
                    if fen_part.contains('Q') {
                        rights.view_bits_mut::<Lsb0>().set(1, true);
                    }
                    if fen_part.contains('k') {
                        rights.view_bits_mut::<Lsb0>().set(2, true);
                    }
                    if fen_part.contains('q') {
                        rights.view_bits_mut::<Lsb0>().set(3, true);
                    }

                    if fen_part.contains('-') && rights > 0 { // TODO: Throw an error if we hit the 'else' arm and rights is not 0
                    }
                    result_obj.castling_rights = rights;
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

        // Update castling rights
        if r#move.start == 56 || r#move.target == 56 {
            // Black queenside rook
            tracing::debug!("Lost queenside castling (black) through rook movement");
            self.castling_rights.view_bits_mut::<Lsb0>().set(3, false);
        } else if r#move.start == 0 || r#move.target == 0 {
            // White queenside rook
            tracing::debug!("Lost queenside castling (white) through rook movement");
            self.castling_rights.view_bits_mut::<Lsb0>().set(1, false);
        } else if r#move.start == 7 || r#move.target == 7 {
            // White kingside rook
            tracing::debug!("Lost kingside castling (white) through rook movement");
            self.castling_rights.view_bits_mut::<Lsb0>().set(0, false);
        } else if r#move.start == 63 || r#move.target == 63 {
            // Black kingside rook
            tracing::debug!("Lost kingside castling (black) through rook movement");
            self.castling_rights.view_bits_mut::<Lsb0>().set(2, false);
        } else if r#move.start == 4 || r#move.target == 4 {
            // Black king
            tracing::debug!("Lost castling (black) through king movement");
            self.castling_rights.view_bits_mut::<Lsb0>().set(2, false);
            self.castling_rights.view_bits_mut::<Lsb0>().set(3, false);
        } else if r#move.start == 60 || r#move.target == 60 {
            // White king
            tracing::debug!("Lost castling (black) through king movement");
            self.castling_rights.view_bits_mut::<Lsb0>().set(0, false);
            self.castling_rights.view_bits_mut::<Lsb0>().set(1, false);
        }

        self.piece_list[r#move.start] = PieceType::None;
        self.piece_list[r#move.target] = moving_piece_type;
    }
    fn update_capture_bitboards(&mut self) {
        for team_id in 0..=Team::Black as usize {
            let mut capture_bitboard = Bitboard::default();
            let legals = self.get_psuedolegal_moves();

            for (square, (bitboard, _legal_moves)) in legals.iter().enumerate().take(64) {
                if self.get_square_team(square) as usize == team_id {
                    let piece_type = self.piece_list[square];
                    if piece_type == PieceType::Pawn {
                        capture_bitboard |= *bitboard & !self.pawn_push_compute[team_id][square]
                    } else {
                        capture_bitboard |= *bitboard;
                    }
                }
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
                print!(
                    "{} ",
                    display_map
                        .get(&bit_opt)
                        .expect("Exception while rendering piece list: slot doesn't exist")
                );
            }
        }
        println!();
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
        let pl = self.piece_list;
        let mut move_list: Vec<(Bitboard, Vec<Move>)> = Vec::new(); // The bitboard is used for highlighting moves the selected square has

        pl.iter().enumerate().for_each(|(index, piece_type)| {
            let default_push = (Bitboard::default(), vec![Move::default()]);

            let team = self.get_square_team(index);
            if team != Team::None {
                let piece_obj = Piece {
                    piece_type: *piece_type,
                    position: index,
                    team,
                };
                let (psuedo_bitboard, psuedo_moves) = match piece_type {
                    PieceType::Bishop | PieceType::Rook | &PieceType::Queen => {
                        compute_slider(self, piece_obj)
                    }
                    PieceType::King => get_precomputed_king(self, piece_obj),
                    PieceType::Knight => get_precomputed_knight(self, piece_obj),
                    PieceType::Pawn => get_precomputed_pawn(self, piece_obj),
                    _ => default_push,
                };

                move_list.push((psuedo_bitboard, psuedo_moves));
            } else {
                move_list.push(default_push);
            }
        });

        // Add castling
        // K, Q, k q
        let white_check = self.is_team_checked(Team::White);
        let black_check = self.is_team_checked(Team::Black);

        for castling_move in 0..4 {
            let castling_rights_bits = self.castling_rights.view_bits::<Lsb0>();
            if castling_rights_bits
                .get(castling_move)
                .expect("Attempted to access out-of-bounds castling bit")
                .then_some(1)
                .is_some()
                && !(if castling_move < 2 {
                    white_check
                } else {
                    black_check
                })
            {
                // Update bitboard for this square
                let king_square = if castling_move < 2 { 4 } else { 60 };
                if self.piece_list[king_square] != PieceType::King {
                    continue;
                };
                let (bitboard, move_vec) = &mut move_list[king_square];

                if (castling_move == 0 || castling_move == 2)
                    && pl[king_square + 2] == PieceType::None
                    && pl[king_square + 1] == PieceType::None
                {
                    bitboard
                        .state
                        .view_bits_mut::<Lsb0>()
                        .set(king_square + 2, true);
                    move_vec.push(Move {
                        start: king_square,
                        target: king_square + 2,
                        captures: None,
                        is_pawn_double: false,
                        is_castle: true,
                    });
                } else if pl[king_square - 2] == PieceType::None
                    && pl[king_square - 1] == PieceType::None
                    && pl[king_square - 3] == PieceType::None
                {
                    bitboard
                        .state
                        .view_bits_mut::<Lsb0>()
                        .set(king_square - 2, true);
                    move_vec.push(Move {
                        start: king_square,
                        target: king_square - 2,
                        captures: None,
                        is_pawn_double: false,
                        is_castle: true,
                    });
                }
            }
        }

        move_list
    }
    pub fn dump_positions(&self) {
        for (square, _) in self.piece_list.iter().enumerate() {
            if let Some(piece) = self.get_piece_at_pos(square) {
                println!(
                    "{:?} {:?} @ {:?} ({})",
                    piece.team,
                    piece.piece_type,
                    Bitboard::bit_idx_to_al_notation(square),
                    square
                )
            }
        }
    }
    pub fn is_team_checked(&self, team: Team) -> bool {
        let enemy_capture_bitboard = self.capture_bitboard[Team::White as usize]
            | self.capture_bitboard[Team::Black as usize];

        let in_check =
            enemy_capture_bitboard & self.board_pieces[team as usize][PieceType::King as usize];

        in_check.state > 0
    }
    pub fn get_legal_moves(&self) -> Vec<(Bitboard, Vec<Move>)> {
        let pl_moves = self.get_psuedolegal_moves();
        let mut legal_moves: Vec<(Bitboard, Vec<Move>)> = Vec::new();

        // This is a list of what moves are available from what square, let's cut that down by active team
        for (mut bitboard, move_vector) in pl_moves {
            // Check
            let mut lm_vector: Vec<Move> = Vec::new();

            move_vector.iter().for_each(|available_move| {
                let mut testing_board = *self; // EXPENSIVE? TODO: Decide whether or not to keep this
                let team_moving = testing_board.get_square_team(available_move.start);
                let move_att = testing_board.make_move(*available_move);

                if move_att.is_ok() {
                    if testing_board.is_team_checked(team_moving) {
                        bitboard
                            .state
                            .view_bits_mut::<Lsb0>()
                            .set(available_move.target, false);
                    } else {
                        lm_vector.push(*available_move);
                    }
                }
            });

            legal_moves.push((bitboard, lm_vector))
        }

        legal_moves
    }
    pub fn prune_moves_for_team_mut(
        &mut self,
        move_list: Vec<(Bitboard, Vec<Move>)>,
        team: Team,
    ) -> Vec<Move> {
        let mut pruned_list: Vec<Move> = Vec::new();
        move_list.iter().for_each(|(_, move_vector)| {
            move_vector.iter().for_each(|available_move| {
                let team_moving = self.get_square_team(available_move.start);
                if team_moving == team {
                    pruned_list.push(*available_move);
                }
            })
        });

        if pruned_list.is_empty() && self.is_team_checked(self.active_team) {
            self.active_team_checkmate = true;
        }
        pruned_list
    }
    pub fn prune_moves_for_team(
        &self,
        move_list: Vec<(Bitboard, Vec<Move>)>,
        team: Team,
    ) -> Vec<Move> {
        let mut pruned_list: Vec<Move> = Vec::new();
        move_list.iter().for_each(|(_, move_vector)| {
            move_vector.iter().for_each(|available_move| {
                let team_moving = self.get_square_team(available_move.start);
                if team_moving == team {
                    pruned_list.push(*available_move);
                }
            })
        });

        pruned_list
    }
    pub fn get_square_team(&self, square_idx: usize) -> Team {
        let white_check = self.get_team_coverage(Team::White);
        let black_check = self.get_team_coverage(Team::Black);

        {
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
                    Team::Black
                } else {
                    // There is not a piece here.
                    Team::None
                }
            } else {
                Team::White
            }
        }
    }
    pub fn make_move(&mut self, r#move: Move) -> Result<(), MoveError> {
        // Update out of the target positions
        let moving_piece_type = self.piece_list[r#move.start];
        let square_team = self.get_square_team(r#move.start);
        let target_team = self.get_square_team(r#move.target);

        if r#move.start == r#move.target {
            return Err(MoveError::NotAMove);
        }
        if square_team != Team::None {
            if target_team == square_team {
                return Err(MoveError::AttackedAlly);
            }
            tracing::debug!("{square_team:?} {moving_piece_type:?} {move:?}");

            self.move_piece(square_team, moving_piece_type, r#move);

            // Move the rook for castlings
            // White

            if r#move.is_castle && r#move.target == 6 {
                // Rook to a5
                self.move_piece(square_team, PieceType::Rook, {
                    Move {
                        start: 7,
                        target: 5,
                        captures: None,
                        is_pawn_double: false,
                        is_castle: true,
                    }
                });
            } else if r#move.is_castle && r#move.target == 2 {
                // Rook to a3
                self.move_piece(square_team, PieceType::Rook, {
                    Move {
                        start: 0,
                        target: 3,
                        captures: None,
                        is_pawn_double: false,
                        is_castle: true,
                    }
                });
            } else if r#move.is_castle && r#move.target == 58 {
                // Rook to a3
                self.move_piece(square_team, PieceType::Rook, {
                    Move {
                        start: 56,
                        target: 59,
                        captures: None,
                        is_pawn_double: false,
                        is_castle: true,
                    }
                });
            } else if r#move.is_castle && r#move.target == 62 {
                // Rook to a3
                self.move_piece(square_team, PieceType::Rook, {
                    Move {
                        start: 63,
                        target: 61,
                        captures: None,
                        is_pawn_double: false,
                        is_castle: true,
                    }
                });
            }

            self.en_passant_square = if r#move.is_pawn_double {
                Some(r#move.target)
            } else {
                self.en_passant_square
            };
            self.en_passant_turn = Some(self.turn_clock);

            // Crudely handle promotions by queening any pawns that finished

            self.update_capture_bitboards();

            if self.active_team == Team::Black {
                self.active_team = Team::White;
                self.turn_clock += 1;
                self.en_passant_square = None;
            } else {
                self.active_team = Team::Black // TODO: Account for three turn order with red before white
            }
            self.ply_clock += 1;
        } else {
            return Err(MoveError::NoUnit);
        }

        Ok(())
    }
    pub fn as_fen(&self) -> String {
        let mut castling_rights = String::from(if self.castling_rights > 0 { "" } else { "-" });
        let en_passant_square = {
            if let Some(eps) = self.en_passant_square {
                if let Some(eps_str) = Bitboard::bit_idx_to_al_notation(eps) {
                    eps_str
                } else {
                    String::from("-")
                }
            } else {
                String::from("-")
            }
        };
        let half_move_clock = "0"; // TODO 
        let full_move_clock = self.turn_clock;
        let active_color = if self.active_team == Team::White {
            "w"
        } else {
            "b"
        };
        let mut piece_placement = String::default();

        for castling_move in 0..4 {
            let castling_rights_bits = self.castling_rights.view_bits::<Lsb0>();

            if castling_rights_bits
                .get(castling_move)
                .expect("Attempted to access out-of-bounds castling bit")
                .then_some("k")
                .is_some()
            {
                let mut additional_string = if castling_move % 2 == 0 { 'k' } else { 'q' };
                if castling_move < 2 {
                    additional_string = additional_string.to_ascii_uppercase()
                }

                castling_rights.push(additional_string);
            }
        }

        let mut empty_square_head = 0; // Add to this for every empty square, reset on every filled square

        let mut rank = 0;
        // Write pieces
        for rank_of_pieces in self.piece_list.iter().enumerate().rev().array_chunks::<8>() {
            for (square, piece_type) in rank_of_pieces.iter().rev() {
                let team = self.get_square_team(*square);

                let mut piece_char = match *(*piece_type) {
                    PieceType::None => '0',
                    PieceType::Pawn => 'p',
                    PieceType::Rook => 'r',
                    PieceType::Bishop => 'b',
                    PieceType::Knight => 'n',
                    PieceType::Queen => 'q',
                    PieceType::King => 'k',
                };

                if team == Team::White {
                    piece_char = piece_char.to_ascii_uppercase()
                }
                if *piece_type == &PieceType::None || team == Team::None {
                    empty_square_head += 1;
                } else {
                    if empty_square_head != 0 {
                        // Append empty squares
                        piece_placement.push_str(&(empty_square_head).to_string())
                    }
                    empty_square_head = 0;
                    piece_placement.push(piece_char)
                }
            }

            rank += 1;
            if empty_square_head != 0 {
                // Append empty squares if there is nothing here
                piece_placement.push_str(&(empty_square_head).to_string())
            }
            empty_square_head = 0;
            if rank != 8 {
                // Append a splitter
                piece_placement.push('/')
            }
        }
        format!(
            "{piece_placement} {active_color} {castling_rights} {en_passant_square} {half_move_clock} {full_move_clock}"
        )
    }
    pub fn unmake_move(&mut self, r#move: Move) -> Result<(), MoveError> {
        let moving_piece_type = self.piece_list[r#move.target];
        let square_team = self.get_square_team(r#move.target);
        let target_team = self.get_square_team(r#move.start);

        if r#move.start == r#move.target {
            return Err(MoveError::NotAMove);
        }
        if square_team != Team::None {
            if target_team == square_team {
                return Err(MoveError::AttackedAlly);
            }

            self.move_piece(
                square_team,
                moving_piece_type,
                Move {
                    start: r#move.target,
                    target: r#move.start,
                    captures: r#move.captures,
                    is_pawn_double: false,
                    is_castle: false,
                },
            );

            if let Some(fallen_piece) = r#move.captures {
                self.piece_list[r#move.target] = fallen_piece.piece_type;
                self.board_pieces[fallen_piece.team as usize][fallen_piece.piece_type as usize]
                    .state
                    .view_bits_mut::<Lsb0>()
                    .set(r#move.target, true);
            }
            if r#move.is_castle {
                let is_queenside = r#move.target < r#move.start;
                let is_kingside = r#move.target > r#move.start;
                let rights_index = square_team as usize;
                if is_kingside {
                    self.castling_rights
                        .view_bits_mut::<Lsb0>()
                        .set(rights_index, true);
                } else if is_queenside {
                    self.castling_rights
                        .view_bits_mut::<Lsb0>()
                        .set(rights_index + 1, true);
                }

                // Unmove rooks

                if r#move.is_castle && r#move.target == 6 {
                    // Rook to a5
                    self.move_piece(square_team, PieceType::Rook, {
                        Move {
                            start: 5,
                            target: 7,
                            captures: None,
                            is_pawn_double: false,
                            is_castle: true,
                        }
                    });
                } else if r#move.is_castle && r#move.target == 2 {
                    // Rook to a3
                    self.move_piece(square_team, PieceType::Rook, {
                        Move {
                            start: 3,
                            target: 0,
                            captures: None,
                            is_pawn_double: false,
                            is_castle: true,
                        }
                    });
                } else if r#move.is_castle && r#move.target == 58 {
                    // Rook to a3
                    self.move_piece(square_team, PieceType::Rook, {
                        Move {
                            start: 59,
                            target: 56,
                            captures: None,
                            is_pawn_double: false,
                            is_castle: true,
                        }
                    });
                } else if r#move.is_castle && r#move.target == 62 {
                    // Rook to a3
                    self.move_piece(square_team, PieceType::Rook, {
                        Move {
                            start: 61,
                            target: 63,
                            captures: None,
                            is_pawn_double: false,
                            is_castle: true,
                        }
                    });
                }
            }

            if self.active_team == Team::Black {
                self.active_team = Team::White;
            } else {
                self.turn_clock -= 1;
                self.en_passant_square = None;
                self.active_team = Team::Black // TODO: Account for three turn order with red before white
            }
            self.ply_clock -= 1;
            self.update_capture_bitboards();
        } else {
            return Err(MoveError::NoUnit);
        }

        Ok(())
    }
    pub fn opponent_attacking_square(&self, pos: usize) -> bool {
        let enemy_capture_bitboard = (self.capture_bitboard[Team::White as usize]
            | self.capture_bitboard[Team::Black as usize])
            & !self.capture_bitboard[self.active_team as usize];

        enemy_capture_bitboard
            .state
            .view_bits::<Lsb0>()
            .get(pos)
            .unwrap()
            .then_some(true)
            .is_some()
    }
    pub fn get_piece_at_pos(&self, pos: usize) -> Option<Piece> {
        let target_piece_type = self.piece_list[pos];

        if target_piece_type != PieceType::None {
            Some(Piece {
                team: self.get_square_team(pos),
                position: pos,
                piece_type: target_piece_type,
            })
        } else {
            None
        }
    }
}
