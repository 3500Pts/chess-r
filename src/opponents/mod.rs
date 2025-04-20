// TODO: Add an opponent module that just seeks a UCI protocol response from a websocket
// TODO: Add a timer that is passed to the opponent

use std::{cmp::Ordering, collections::HashMap};

use rand::{Rng, rng, seq::IndexedRandom};

use crate::{
    bitboard::{PieceType, Team},
    board::BoardState,
    r#move::{self, Move},
};

#[derive(Debug, Copy, Clone)]
pub struct NegamaxEval {
    eval: i32,
    legal_move: Move,
}

pub trait ChessOpponent {
    fn get_move(&mut self, board: BoardState) -> Option<Move>;
}

#[derive(Debug, Copy, Clone)]
pub struct Randy {}
impl ChessOpponent for Randy {
    fn get_move(&mut self, board: BoardState) -> Option<Move> {
        let legals = board.prune_moves_for_team(board.get_legal_moves(), board.active_team);
        legals.choose(&mut rand::rng()).copied()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Matt {
    pub search_budget: i32,
}
impl ChessOpponent for Matt {
    fn get_move(&mut self, board: BoardState) -> Option<Move> {
        let mut legals = board.prune_moves_for_team(board.get_legal_moves(), board.active_team);
        let mut mapped_legals = Vec::new();
        // expensive...
        for legal_move in legals {
            let eval = Matt::evaluate_move(&board, legal_move, self.search_budget - 1)
                * if board.active_team == Team::White {
                    1
                } else {
                    -1
                };

            mapped_legals.push(NegamaxEval {
                eval: eval,
                legal_move: legal_move,
            })
        }

        mapped_legals.sort_by(|a, b| a.eval.cmp(&b.eval));
        println!("Evaled to: {}", mapped_legals[0].eval);
        if mapped_legals.len() > 1 {
            Some(mapped_legals[mapped_legals.len() - 1].legal_move)
        } else {
            None
        }
    }
}
impl Matt {
    fn evaluate_move(board: &BoardState, ava_move: Move, search_budget: i32) -> i32 {
        // SUPER EXPENSIVE to recurse over it
        let mut virtual_board = board.clone();
        let move_res = virtual_board.make_move(ava_move);

        let mut eval_score = Matt::evaluate(virtual_board);

        if search_budget == 0 {
        } else {
            let legals = board.prune_moves_for_team(
                (&virtual_board).get_legal_moves(),
                (&virtual_board).active_team,
            );

            legals.iter().for_each(|legal_move| {
                eval_score += Matt::evaluate_move(&virtual_board, *legal_move, search_budget - 1)
            });
        }
        eval_score
    }
    fn evaluate_team(board: &BoardState, team: Team) -> i32 {
        let scores: HashMap<PieceType, i32> = HashMap::from([
            (PieceType::None, 0),
            (PieceType::Pawn, 100),
            (PieceType::Knight, 300),
            (PieceType::Bishop, 300),
            (PieceType::Rook, 500),
            (PieceType::Queen, 900),
            (PieceType::King, 1000000),
        ]);

        let mut material = 0;
        for (idx, piece) in board.piece_list.iter().enumerate() {
            if board.get_square_team(idx).unwrap_or(Team::None) == team {
                material += scores.get(piece).unwrap_or(&0);
            }
        }

        material += board
            .prune_moves_for_team(board.get_legal_moves(), board.active_team)
            .len() as i32
            * 25; 
        material
    }
    fn evaluate(board: BoardState) -> i32 {
        let white_eval = Matt::evaluate_team(&board, Team::White);
        let black_eval = Matt::evaluate_team(&board, Team::Black);

        let jiggle = rng().random_range(-100..100);

        return (white_eval - black_eval) + jiggle;
    }
}
