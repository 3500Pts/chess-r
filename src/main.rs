#![feature(int_roundings)]

pub mod bitboard;
pub mod board;
pub mod r#move; 

use board::BoardState;

use crate::bitboard::*;

const START_POS_CHESS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const RANDOM_GAME_POS: &str = "rnb1kbnr/pqpp3p/1p2ppp1/8/4P3/PPN5/2PPBPPP/R1BQ1RK1 w - - 3 12";
fn main() {
    println!("Hello, world!");

    let board_full_test = BoardState::from_fen(String::from(START_POS_CHESS)).unwrap();
    board_full_test.render_piece_list();
}
