
pub mod bitboard;
use crate::bitboard::*;

fn main() {
    println!("Hello, world!");
    
    let board_test = Bitboard {
        state: 9123 as u64
    };

    board_test.print();
}
