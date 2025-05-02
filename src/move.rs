use std::fmt::{Display, Formatter};

use bitvec::{order::Lsb0, view::BitView};

use crate::{
    bitboard::{Bitboard, PieceType, Team},
    board::BoardState,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Piece {
    pub piece_type: PieceType,
    pub team: Team,
    pub position: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Move {
    pub start: usize,
    pub target: usize,
    pub captures: Option<Piece>,
    pub is_pawn_double: bool, // en passant tracker
    pub is_castle: bool,
}
impl Move {
    fn set_start(&self, pos: usize) -> Self {
        let mut clone = *self;
        clone.start = pos;
        clone
    }
}
impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(alno) = Bitboard::bit_idx_to_al_notation(self.start) {
            f.write_str(&alno)?;
            f.write_str("->")?;
        }
        if let Some(alno) = Bitboard::bit_idx_to_al_notation(self.target) {
            f.write_str(&alno)?;
        }
        Ok(())
    }
    
}
impl Default for Move {
    fn default() -> Self {
        Move {
            start: 0,
            target: 0,
            captures: None, 
            is_pawn_double: false,
            is_castle: false
        }
    }
    
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MoveError {
    AttackedAlly,
    NoUnit,
    NotAMove,
}
// Should match [compute_edges] from board.rs exactly in direction
const DIRECTION_OFFSETS: [i32; 8] = [
    // Rook moves are 0-4
    8,  // n
    -8, // s
    1,  // e
    -1, // w
    // Bishop moves are 4-7
    9,  // ne
    -7, // se
    7,  // nw
    -9, // sw
];

fn is_square_attackable(board: &BoardState, piece: Piece, possible_target: usize) -> bool {
    let target_team = board.get_square_team(possible_target);
    let target_piece_type = board.piece_list[possible_target];

    if target_piece_type == PieceType::None {
        // Nothing is there, do not process based on team
        true
    } else {
        // You can never attack your teammates
        if target_team != piece.team {
            true
        } else {
            tracing::debug!(
                "[MOVEGEN] {:?}{:?} Blocked by friendly piece",
                piece.team,
                piece.piece_type
            );
            false
        }
    }
}

fn psuedolegalize_move(
    move_list: &mut Vec<Move>,
    bitboard: &mut Bitboard,
    cmove: Move,
    condition: bool,
) {
    if condition {
        move_list.push(cmove);
    }
    bitboard
        .state
        .view_bits_mut::<Lsb0>()
        .set(cmove.target, condition);
}
/*
    Gets psuedolegal moves for the pawns.
*/
pub fn compute_pawn(board: &BoardState, piece: Piece) -> (Bitboard, Vec<Move>) {
    let mut bitboard = Bitboard::default();
    let mut computed_moves: Vec<Move> = Vec::new();
    let forward_direction: i32 = match piece.team {
        Team::Black => -8,
        Team::White => 8, // making this 7 makes for an interesting diagonal pawn...
        _ => {
            panic!("Pawn movements for unconventional teams are unhandled"); // TODO: Dont forget to fix this if you add other teams
        }
    };

    let pawn_view_range = forward_direction.signum();

    let far_edge_dist = match piece.team {
        Team::Black => board.edge_compute[piece.position][1],
        Team::White => board.edge_compute[piece.position][0],
        _ => {
            unreachable!()
        }
    };

    if far_edge_dist == 0 {
        return (bitboard, computed_moves); // Promotable
    }

    let mut offset_index = 0;
    let step_length = if far_edge_dist == 6 { 2 } else { 1 }; // Do we award initial advances from any start position? It is an nteresting question, but for now we just assume normal start

    let of_start = (forward_direction - pawn_view_range).min(forward_direction + pawn_view_range);
    let of_end = (forward_direction - pawn_view_range).max(forward_direction + pawn_view_range);

    for offset in of_start..=of_end {
        'step_ray: for step in 1..=step_length {
            let possible_target = (piece.position as i32 + (offset * step)) as usize;
            if !(0..board.piece_list.len()).contains(&possible_target) {
                continue;
            };
            let target_file = possible_target % 8;
            let start_file = piece.position % 8;

            if target_file.abs_diff(start_file) > 3 {
                continue;
            };

            let target_piece_type = board.piece_list[possible_target];

            let target_piece = board.get_piece_at_pos(possible_target);
            let resulting_move = Move {
                start: piece.position,
                target: possible_target,
                is_pawn_double: step == 2,
                captures: target_piece,
                is_castle: false,
            };
            if target_piece_type == PieceType::None {
                psuedolegalize_move(
                    &mut computed_moves,
                    &mut bitboard,
                    resulting_move,
                    is_square_attackable(board, piece, possible_target) && offset_index == 1,
                );
            } else {
                psuedolegalize_move(
                    &mut computed_moves,
                    &mut bitboard,
                    resulting_move,
                    is_square_attackable(board, piece, possible_target)
                        && (offset_index != 1 && step == 1),
                );
                // Can't jump over it
                break 'step_ray;
            }
        }
        offset_index += 1;
    }

    // en passant
    if let Some(en_pass) = board.en_passant_square {
        if en_pass.abs_diff(piece.position) == 1 {
            let target_piece_type = board.piece_list[en_pass];
            let target_piece = board.get_piece_at_pos(en_pass);

            let resulting_move = Move {
                start: piece.position,
                target: en_pass,
                is_pawn_double: false,
                captures: target_piece,
                is_castle: false,
            };

            psuedolegalize_move(
                &mut computed_moves,
                &mut bitboard,
                resulting_move,
                is_square_attackable(board, piece, en_pass)
                    && board.en_passant_turn.unwrap() == board.turn_clock
                    && target_piece_type == PieceType::Pawn,
            );
        }
    }

    (bitboard, computed_moves)
}

/*
Computes psuedolegals for rooks, queens, bishops, kings. Requires pre-computed edges.
*/
pub fn compute_slider(board: &BoardState, piece: Piece) -> (Bitboard, Vec<Move>) {
    let index_start = if piece.piece_type == PieceType::Bishop {
        4
    } else {
        0
    };
    let index_end = if piece.piece_type == PieceType::Rook {
        4
    } else {
        8
    };

    // Queen/KING will do 0-8

    let mut computed_moves: Vec<Move> = Vec::new();
    let mut bitboard = Bitboard::default();

    // Push in each available direction for the rider until it hits an edge or an occupied spot.
    let square_bit_index = piece.position;

    // index is a direction
    for (index, direction_add) in DIRECTION_OFFSETS.iter().enumerate().take(index_end).skip(index_start)  {
        let mut indexed_direction = board.edge_compute[square_bit_index][index];

        if indexed_direction >= 1 {
            tracing::debug!(
                "[MOVEGEN] {:?}{:?} {} has {indexed_direction} squares of depth in direction {index}, seeking to add {}",
                piece.team,
                piece.piece_type,
                piece.position,
                DIRECTION_OFFSETS[index]
            );
        } else {
            // We are against the edge in this direction
            tracing::debug!(
                "[MOVEGEN] {:?}{:?} Blocked by wall in direction {index}",
                piece.team,
                piece.piece_type
            );
            continue;
        }

        if piece.piece_type == PieceType::King {
            indexed_direction = 1;
        }

        'raycast_check: for raycast in 1..=indexed_direction {
            let possible_target =
                (square_bit_index as i32 + (raycast as i32 * direction_add)) as usize;

            if !(0..board.piece_list.len()).contains(&possible_target) {
                break 'raycast_check;
            };

            let target_piece_type = board.piece_list[possible_target];

            let target_piece = board.get_piece_at_pos(possible_target);
            let resulting_move = Move {
                start: piece.position,
                target: possible_target,
                is_pawn_double: false,
                captures: target_piece,
                is_castle: false,
            };
            psuedolegalize_move(
                &mut computed_moves,
                &mut bitboard,
                resulting_move,
                is_square_attackable(board, piece, possible_target),
            );

            if target_piece_type != PieceType::None {
                // Piece blocks further movement in this direction
                break 'raycast_check;
            }
        }
    }
    (bitboard, computed_moves)
}

// For nightrider, we could do this recursively until we get 0 results
// compute_knight
pub fn compute_knight(board: &BoardState, piece: Piece) -> (Bitboard, Vec<Move>) {
    let knight_moves: [i32; 8] = [10, 17, -10, -17, 15, -15, 6, -6];
    let mut computed_moves: Vec<Move> = Vec::new();
    let mut bitboard = Bitboard::default();

    for knight_square in knight_moves {
        let possible_target = ((piece.position as i32) + knight_square) as usize;
        if !(0..board.piece_list.len()).contains(&possible_target) {
            continue;
        };
        let target_piece = board.get_piece_at_pos(possible_target);
        let resulting_move = Move {
            start: piece.position,
            target: possible_target,
            is_pawn_double: false,
            captures: target_piece,
            is_castle: false,
        };

        let target_file = possible_target % 8;
        let start_file = piece.position % 8;

        // Disable stuff that lets you loop around the board, which seems to only happen laterally.
        // Do this by ignoring anything that is on file A/B if you're on H/G and vice versa
        psuedolegalize_move(
            &mut computed_moves,
            &mut bitboard,
            resulting_move,
            is_square_attackable(board, piece, possible_target)
                && target_file.abs_diff(start_file) <= 2,
        );
    }

    (bitboard, computed_moves)
}
