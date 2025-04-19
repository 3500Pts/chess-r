#![feature(int_roundings)]
#![feature(try_find)]
//#![windows_subsystem = "windows"]

pub mod bitboard;
pub mod board;
pub mod r#move; 
pub mod ui;
pub mod opponents;

const START_POS_CHESS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const RANDOM_GAME_POS: &str = "rnb1kbnr/pqpp3p/1p2ppp1/8/4P3/PPN5/2PPBPPP/R1BQ1RK1 w - - 3 12";

use bitboard::Team;
use board::BoardState;
use ggez::conf::WindowSetup;
use ggez::event;
use rand::random_range;
use ui::MainState;

pub fn main() {


    let player_team = if (random_range(0..=1)) == 0 {Team::Black} else {Team::White};
    
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
    
    /*let chess_filter = EnvFilter::builder()
    .with_default_directive(LevelFilter::INFO.into())
    .from_env().unwrap()
    .add_directive("chess-r=debug".parse().unwrap());

    tracing_subscriber::fmt()
    .with_level(true)
    .with_env_filter(chess_filter)
    .compact()
    .init();*/

    let (mut ctx, event_loop) = cb.build().expect("Failed to build ggez context");
    
    let state = MainState::new(board_full_test, &mut ctx, player_team).unwrap();
    event::run(ctx, event_loop, state);
}
