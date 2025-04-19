// TODO: Add an opponent module that just seeks a UCI protocol response from a websocket
// TODO: Add a timer that is passed to the opponent

use crate::{board::BoardState, r#move::Move};

trait ChessOpponent {
    fn get_move(board: BoardState) -> Move;
}

struct Randy {}
impl ChessOpponent for Randy  {
    fn get_move(board: BoardState) {
        let legals = board.get_legal_moves();
        return 
    }
}