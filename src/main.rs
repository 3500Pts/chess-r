#![feature(int_roundings)]
#![feature(try_find)]
//#![windows_subsystem = "windows"]

pub mod bitboard;
pub mod board;
pub mod r#move; 
pub mod ui;

const START_POS_CHESS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const RANDOM_GAME_POS: &str = "rnb1kbnr/pqpp3p/1p2ppp1/8/4P3/PPN5/2PPBPPP/R1BQ1RK1 w - - 3 12";

use bitboard::Team;
use board::BoardState;
use ggez::conf::WindowSetup;
use ggez::event;
use ui::MainState;

pub fn main() {
    let board_full_test = BoardState::from_fen(String::from(START_POS_CHESS)).expect("Failed to create board from FEN");
    //board_full_test.render_piece_list();
    println!("{}", board_full_test.get_team_coverage(Team::White));
    let cb = ggez::ContextBuilder::new("chess-r", "3500pts")
    .window_setup(WindowSetup {
        title: String::from("CHESSR"),
        samples: ggez::conf::NumSamples::Four,
        icon: String::from("/horsey/bp.png"),
        srgb: false,
        vsync: true
    });
    
    let (mut ctx, event_loop) = cb.build().expect("Failed to build ggez context");
    
    let state = MainState::new(board_full_test, &mut ctx).unwrap();
    event::run(ctx, event_loop, state);
}
