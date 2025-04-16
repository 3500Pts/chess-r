// bitboard.rs
use bitvec::prelude::*;

pub struct Bitboard {
    pub state: u64,
}

impl Bitboard {
    pub fn print(&self) {
        print!("  a b c d e f g h");
        // print each rank
        let state_slice = self.state.view_bits::<Lsb0>();
        let mut step: u32 = 1;
        let mut last_rank = 0;
        for rank in (0..8).rev() {
            print!("\n{} ", rank+1);

            for file in 0..8 {
                let bit_opt = state_slice.get((rank * 8) + file);
                if let Some(bit) = bit_opt {
                    let string = String::from("");
                    print!("{string}{} ", bit.then(|| { "X" }).unwrap_or("O"));
                }
            }
        }
    }
}
