#![feature(int_roundings)]
#![feature(try_find)]
#![feature(mpmc_channel)]
#![feature(string_remove_matches)]
//#![windows_subsystem = "windows"]

pub mod bitboard;
pub mod board;
pub mod r#move;
pub mod opponents;
pub mod rules;
pub mod ui;
const START_POS_CHESS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

use std::time::Duration;

use bitboard::Team;
use board::BoardState;
use ggez::conf::{WindowMode, WindowSetup};
use ggez::event;
use opponents::*;
use rand::random_range;
use ui::MainState;

#[tokio::main]
async fn main() {
    let player_team = if (random_range(0..=1)) == 0 {
        Team::Black
    } else {
        Team::White
    };

    let board_full_test = BoardState::from_fen(String::from(START_POS_CHESS))
        .expect("Failed to create board from FEN");

    println!("{}", board_full_test.get_team_coverage(Team::White));
    let cb = ggez::ContextBuilder::new("chess-r", "3500pts")
        .window_setup(WindowSetup {
            title: String::from("CHESSR"),
            samples: ggez::conf::NumSamples::Four,
            icon: String::from("/horsey/bp.png"),
            srgb: false,
            vsync: true,
        })
        .window_mode(
            WindowMode::default()
                .resizable(false)
                .max_dimensions(800.0, 800.0),
        );

    let (mut ctx, event_loop) = cb.build().unwrap();

    let state: MainState = MainState::new(
        board_full_test,
        &mut ctx,
        player_team,
        ChessOpponent::Ada(Duration::from_millis(1050)),
    )
    .unwrap();
    event::run(ctx, event_loop, state);
}
