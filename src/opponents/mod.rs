// TODO: Add an opponent module that just seeks a UCI protocol response from a websocket
// TODO: Add a timer that is passed to the opponent

use rand::{rng, seq::IndexedRandom};

use crate::{board::BoardState, r#move::Move};

pub trait ChessOpponent {
    fn get_move(&mut self, board: BoardState) -> Option<Move>;
}

pub struct Randy {}
impl ChessOpponent for Randy  {
    fn get_move(&mut self, board: BoardState) -> Option<Move> {
        let legals = board.prune_moves_for_team(board.get_legal_moves(), board.active_team);
        legals.choose(&mut rand::rng()).copied()
    }
}
