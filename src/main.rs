#![feature(int_roundings)]
#![windows_subsystem = "windows"]

pub mod bitboard;
pub mod board;
pub mod r#move; 
pub mod ui;

const START_POS_CHESS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const RANDOM_GAME_POS: &str = "rnb1kbnr/pqpp3p/1p2ppp1/8/4P3/PPN5/2PPBPPP/R1BQ1RK1 w - - 3 12";

use board::BoardState;
use ggez::conf::WindowSetup;
use ggez::event;
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};
use ggez::glam::*;
use ui::MainState;

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("chess-r", "3500pts")
    .window_setup(WindowSetup {
        title: String::from("CHESSR"),
        samples: ggez::conf::NumSamples::Four,
        icon: String::from(""),//String::from("../assets/horsey/bp.svg"),
        srgb: false,
        vsync: true
    })
    ;
    let (ctx, event_loop) = cb.build().unwrap();
    let state = MainState::new()?;
    
    let board_full_test = BoardState::from_fen(String::from(START_POS_CHESS)).unwrap();
    board_full_test.render_piece_list();

    event::run(ctx, event_loop, state)
}
