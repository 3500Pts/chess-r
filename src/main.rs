#![feature(int_roundings)]
#![feature(try_find)]
#![feature(mpmc_channel)]
#![feature(string_remove_matches)]
#![feature(iter_array_chunks)]
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
use tracing_subscriber::EnvFilter;
use ui::MainState;

#[tokio::main]
async fn main() {
    let player_team = if (random_range(0..=1)) == 0 {
        Team::White
    } else {
        Team::White
    };

    let board_full_test = BoardState::from_fen(String::from(START_POS_CHESS))
        .expect("Failed to create board from FEN");

    let filter = EnvFilter::builder()
        .from_env()
        .expect("Failed to build envfilter")
        .add_directive(
            "chess_r=warn"
                .parse()
                .expect("Failed to parse tracing directive"),
        );

    let sub_builder = tracing_subscriber::fmt().with_env_filter(filter);

    sub_builder.compact().init();

    let cb = ggez::ContextBuilder::new("chess-r", "3500pts")
        .window_setup(WindowSetup {
            title: String::from("CHESSR"),
            samples: ggez::conf::NumSamples::Four,
            icon: String::from("/horsey/bk.png"),
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
        ChessOpponent::Ada(Duration::from_millis(400)),
    )
    .unwrap();
    event::run(ctx, event_loop, state);
}
