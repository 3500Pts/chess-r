
pub mod bitboard;
use crate::bitboard::*;

fn main() {
    println!("Hello, world!");

    let board_test = Bitboard {
        state: 0x8040201008040201 as u64
    };

    println!("{board_test}")
}
