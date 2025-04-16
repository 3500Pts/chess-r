// bitboard.rs
use bitvec::prelude::*;

pub struct Bitboard {
    pub state: u64,
}

impl Bitboard {
    pub fn print(&self) {
        print!("  a b c d e f g h");
        // print each rank
        let state_slice = self.state.view_bits::<Msb0>();
        let mut index: u32 = 64;
        let mut last_rank = 9;
        for bit in state_slice {
       
            let rank = index.div_ceil(8);
            let rank_as_string = if last_rank > rank {
                let rank_as = rank.to_string();
                format!("\n{} ", rank_as)
            } else {
                String::from("")
            };

            let string = String::from("") + rank_as_string.as_str();

            print!("{string}{} ", bit.then(||{1}).unwrap_or(0));

            index -= 1;
            last_rank = rank;
        }
    }
}
