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
    pub captures: Option<PieceType>,
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
    let mut bitboard = Bitboard::default();

    if target_piece_type == PieceType::None {
        // Nothing is there, do not process based on team
        true
    } else {
        // You can never attack your teammates
        if target_team != Some(piece.team) {
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
/*
    Gets psuedolegal moves for the pawns.
*/
pub fn compute_pawn(board: &BoardState, piece: Piece) -> (Bitboard, Vec<Move>) {
    let mut bitboard = Bitboard::default();
    let mut computed_moves: Vec<Move> = Vec::new();
    let forward_direction: i32 = match piece.team {
        Team::Black => -8,
        Team::White => 8,
        _ => {
            // TODO: Dont forget to fix this if you add other teams
            panic!("Pawn movements for unconventional teams are unhandled");
        }
    };
    let far_edge_dist = match piece.team {
        Team::Black => board.edge_compute[piece.position][1],
        Team::White => board.edge_compute[piece.position][0],
        _ => {
            // TODO: Dont forget to fix this if you add other teams
            panic!("Pawn movements for unconventional teams are unhandled");
        }
    };

    if far_edge_dist == 0 {
        // Promoted
        return (bitboard, computed_moves);
    }

    // Check for pawn in front

    let possible_target = (piece.position as i32 + forward_direction) as usize;
    let target_piece_type = board.piece_list[possible_target];

    let target_team = board.get_square_team(possible_target);

    if target_piece_type == PieceType::None {
        // Nothing is there, do not process based on team
        // Register a capture
        computed_moves.push(Move {
            start: piece.position,
            target: possible_target,
            captures: None,
        });
        bitboard
            .state
            .view_bits_mut::<Lsb0>()
            .set(possible_target, true);
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
    for index in index_start..index_end {
        let edges_from_here = board.edge_compute[square_bit_index].clone();
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

        if square_bit_index == 0 {}

        if piece.piece_type == PieceType::King {
            indexed_direction = 1;
        }

        'raycast_check: for raycast in 1..=indexed_direction {
            let possible_target =
                (square_bit_index as i32 + (raycast as i32 * DIRECTION_OFFSETS[index])) as usize;

            if !(0..board.piece_list.len()).contains(&possible_target) {
                break 'raycast_check;
            };

            let target_piece_type = board.piece_list[possible_target];

            let target_team = board.get_square_team(possible_target);

            bitboard.state.view_bits_mut::<Lsb0>().set(
                possible_target,
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
pub fn compute_knight(board: &BoardState, piece: Piece) -> (Bitboard, Vec<Move>) {
    let knight_moves: [i32; 8] = [10, 17, -10, -17, 15, -15, 6, -6];
    let computed_moves: Vec<Move> = Vec::new();
    let mut bitboard = Bitboard::default();

    for knight_square in knight_moves {
        let possible_target = ((piece.position as i32) + knight_square) as usize;
        if !(0..board.piece_list.len()).contains(&possible_target) {
            continue;
        };
        let target_piece_type = board.piece_list[possible_target];

        bitboard.state.view_bits_mut::<Lsb0>().set(
            possible_target,
            is_square_attackable(board, piece, possible_target),
        );
    }

    (bitboard, computed_moves)
}
