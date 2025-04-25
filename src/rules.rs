mod tests {
    // Tests against all the rules

    use bitvec::order::Lsb0;
    use bitvec::view::BitView;
    use crate::BoardState;
    use crate::r#move::Move;
    use crate::Team;
    
    #[test]
    fn en_passant() {
        let mut test_board = BoardState::from_fen(String::from(
            "rnbqkbnr/4pppp/3p4/2p5/pp6/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1",
        ))
        .expect("Invalid FEN used in testing");
        BoardState::render_piece_list(test_board.piece_list.to_vec());

        // White to move. Do c2c4 to allow black en passant
        let _ = test_board.make_move(Move {
            start: 10,
            target: 26,
            captures: None,
            is_pawn_double: true,
            is_castle: false,
        }).unwrap();
        BoardState::render_piece_list(test_board.piece_list.to_vec());

        println!("{}", test_board.capture_bitboard[Team::Black as usize]);

        assert_eq!(
            test_board.capture_bitboard[Team::Black as usize]
                .state
                .view_bits::<Lsb0>()
                .get(26)
                .expect("Piece Bitboard did not extend to 25 bits")
                .then_some(1),
            Some(1),
            "En passant test failed"
        );
    }

    #[test]
    // Test that you can't en passant after the next turn
    fn en_passant_deferred() {
        let mut test_board = BoardState::from_fen(String::from(
            "rnbqkbnr/4pppp/3p4/2p5/pp6/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1",
        ))
        .expect("Invalid FEN used in testing");

        // White to move. Do c2c4 to allow black en passant
        test_board.make_move(Move {
            start: 10,
            target: 26,
            captures: None,
            is_pawn_double: true,
            is_castle: false,
        }).unwrap();

        let ept = test_board.capture_bitboard[Team::Black as usize]
            .state
            .view_bits::<Lsb0>()
            .get(26)
            .expect("Piece Bitboard did not extend to 25 bits")
            .then_some(1);
        if ept.is_none() {
            panic!("Prerequisite test failed")
        }
        // Black does bishop to a6 instead
        test_board.make_move(Move {
            start: 58,
            target: 32,
            captures: None,
            is_pawn_double: false,
            is_castle: false,
        }).unwrap();

        // White does something else (h2h3), meaning the opportunity for black to take 24 with a pawn should be lost by this next move
        test_board.make_move(Move {
            start: 14,
            target: 21,
            captures: None,
            is_pawn_double: false,
            is_castle: false,
        }).unwrap();

        // Black to move. Check if they can attack square 24 where we just en passanted to
        // There is no other legal captures on the square besides en passant
        assert_eq!(
            test_board.capture_bitboard[Team::Black as usize]
                .state
                .view_bits::<Lsb0>()
                .get(24)
                .expect("Piece Bitboard did not extend to 25 bits")
                .then_some(1),
            None,
            "En passant test failed - you can still capture after a turn"
        );
    }
}
