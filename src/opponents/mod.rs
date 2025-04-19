// TODO: Add an opponent module that just seeks a UCI protocol response from a websocket
// TODO: Add a timer that is passed to the opponent

use crate::board::BoardState;

trait ChessOpponent {
    fn get_move(board: BoardState);
}