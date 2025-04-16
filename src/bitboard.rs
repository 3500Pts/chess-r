use std::fmt::{self, write};

// bitboard.rs
use bitvec::prelude::*;

#[derive(Debug)]
pub struct Bitboard {
    pub state: u64,
}
impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "  a b c d e f g h")?;

        let state_slice = self.state.view_bits::<Lsb0>();
        for rank in (0..8).rev() {
            write!(f, "\n{} ", rank+1)?;

            for file in 0..8 {
                let bit_opt = state_slice.get((rank * 8) + file);
                if let Some(bit) = bit_opt {
                    let string = String::from("");
                    write!(f, "{string}{} ", bit.then(|| { "X" }).unwrap_or("O"))?;
                }
            }
        }
        write!(f, "")
    }
}
impl Bitboard {
}
