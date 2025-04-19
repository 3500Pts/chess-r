use crate::bitboard::{PieceType, Team};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Piece {
    pub piece_type: PieceType,
    pub team: Team,
    pub position: usize
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Move {
    pub start: usize,
    pub target: usize,
    pub captures: Option<PieceType>
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MoveError {
    AttackedAlly,
    NoUnit,
}
// Compute path for rooks, queens, bishops. Requires pre-computed edges
pub fn compute_rider(square_bit_index: usize, piece: Piece, edge_compute: Vec<Vec<usize>>) {
    let index_start = if piece.piece_type == PieceType::Bishop { 4 } else { 0 };
    let index_end = if piece.piece_type == PieceType::Rook { 4 } else { 8 };

    let mut computed_moves: Vec<Move> = Vec::new();

    for index in index_start..index_end {
        let indexed_direction = edge_compute[square_bit_index][index];
        
        let target = square_bit_index + indexed_direction;

        computed_moves.push(
            Move {
                start: square_bit_index,
                target: target,
                captures: None
            }
        );
    }
}