use bitvec::{order::Lsb0, store::BitStore, view::BitView};

use crate::{
    bitboard::*,
    r#move::{Move, MoveError, Piece, *},
};
use std::{
    collections::HashMap,
    fmt::{self},
    io::Read,
};

const LIST_OF_PIECES: &str = "kqrbnpKQRBNP";
const SPLITTER: char = '/';

// Returns a table of the distance to the edges of the board for every square where index 0 of a square's table is the distance to the top, 1 is bottom, 2 is right, 3 is left, 4 is topright, 5 is bottomright, 6 is bottomleft, 7 is topleft.
pub fn compute_edges() -> [[usize; 8]; 64] {
    let mut square_list = [[0; 8]; 64];

    for square_pos in 0..square_list.len() {
        let rank = square_pos.div_floor(8);
        let file = square_pos % 8;

        let top_dist = 7 - rank;
        let bottom_dist = rank;
        let left_dist = file;
        let right_dist = 7 - file;

        square_list[square_pos] = [
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
    pub capture_bitboard: [Bitboard; 2],
    pub en_passant_turn: Option<i64>,
    pub active_team: Team,
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
            self.castling_rights.view_bits_mut::<Lsb0>().set(3, false);
        } else if r#move.start == 0 || r#move.target == 0 {
            // White queenside rook
            self.castling_rights.view_bits_mut::<Lsb0>().set(1, false);
        } else if r#move.start == 7 || r#move.target == 7 {
            // White kingside rook
            self.castling_rights.view_bits_mut::<Lsb0>().set(0, false);
        } else if r#move.start == 63 || r#move.target == 63 {
            // Black kingside rook
            self.castling_rights.view_bits_mut::<Lsb0>().set(2, false);
        } else if r#move.start == 4 || r#move.target == 4 {
            // Black king
            self.castling_rights.view_bits_mut::<Lsb0>().set(2, false);
            self.castling_rights.view_bits_mut::<Lsb0>().set(3, false);
        } else if r#move.start == 60 || r#move.target == 60 {
            // White king
            self.castling_rights.view_bits_mut::<Lsb0>().set(0, false);
            self.castling_rights.view_bits_mut::<Lsb0>().set(1, false);
        }

        self.piece_list[r#move.start] = PieceType::None;
        self.piece_list[r#move.target] = moving_piece_type;
    }
    fn update_capture_bitboards(&mut self) {
        for team_id in 0..Team::Black as usize {
            let mut capture_bitboard = Bitboard::default();
            let legals = self.get_psuedolegal_moves(); // TODO: Make these legal moves

            for square in 0..64 {
                capture_bitboard |= legals[square].0;
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
                    display_map.get(&bit_opt).expect("Exception while rendering piece list: slot doesn't exist")
                );
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
            let default_push = (Bitboard::default(), vec![Move::default()]);

            let team_opt = self.get_square_team(index);
            if let Some(team) = team_opt {
                let piece_obj = Piece {
                    piece_type: *piece_type,
                    position: index,
                    team: team,
                };
                let (psuedo_bitboard, psuedo_moves) = match piece_type {
                    PieceType::Bishop | PieceType::Rook | &PieceType::Queen | PieceType::King => {
                        compute_slider(self, piece_obj)
                    }
                    PieceType::Knight => compute_knight(self, piece_obj),
                    PieceType::Pawn => compute_pawn(self, piece_obj),
                    _ => default_push,
                };

                move_list.push((psuedo_bitboard, psuedo_moves));
            } else {
                move_list.push(default_push);
            }
        });

        // Add castling
        // K, Q, k q
        for castling_move in 0..4 {
            let castling_rights_bits = self.castling_rights.view_bits::<Lsb0>();

            if castling_rights_bits
                .get(castling_move)
                .expect("Attempted to access out-of-bounds castling bit")
                .then_some(1)
                .is_some()
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
    pub fn is_team_checked(&self, team: Team) -> bool {
        let enemy_capture_bitboard = (self.capture_bitboard[Team::White as usize]
            | self.capture_bitboard[Team::Black as usize]);

        let in_check =
            (enemy_capture_bitboard & self.board_pieces[team as usize][PieceType::King as usize]);

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
                let mut testing_board = self.clone(); // EXPENSIVE? TODO: Decide whether or not to keep this
                let team_moving = testing_board
                    .get_square_team(available_move.start)
                    .unwrap_or(Team::None);
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
                let team_moving = self
                    .get_square_team(available_move.start)
                    .unwrap_or(Team::None);
                if (team_moving == team) {
                    pruned_list.push(*available_move);
                }
            })
        });

        if pruned_list.len() == 0 && self.is_team_checked(self.active_team) {
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
                let team_moving = self
                    .get_square_team(available_move.start)
                    .unwrap_or(Team::None);
                if (team_moving == team) {
                    pruned_list.push(*available_move);
                }
            })
        });

        pruned_list
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
            } else {
                self.active_team = Team::Black // TODO: Account for three turn order with red before white
            }
        }

        return Ok(());
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

        let target_piece = if target_piece_type != PieceType::None {
            Some(Piece {
                team: self.get_square_team(pos).unwrap_or(Team::None),
                position: pos,
                piece_type: target_piece_type,
            })
        } else {
            None
        };

        target_piece
    }
}
