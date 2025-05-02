mod tests {
    // Tests against all the rules

    use crate::bitboard;
    use crate::bitboard::Bitboard;
    use crate::BoardState;
    use crate::Team;
    use crate::r#move::Move;
    use bitvec::order::Lsb0;
    use bitvec::view::BitView;

    #[test]
    fn en_passant() {
        let mut test_board = BoardState::from_fen(String::from(
            "rnbqkbnr/4pppp/3p4/2p5/pp6/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1",
        ))
        .expect("Invalid FEN used in testing");
        BoardState::render_piece_list(test_board.piece_list.to_vec());

        // White to move. Do c2c4 to allow black en passant
        let _ = test_board
            .make_move(Move {
                start: 10,
                target: 26,
                captures: None,
                is_pawn_double: true,
                is_castle: false,
            })
            .unwrap();
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
        test_board
            .make_move(Move {
                start: 10,
                target: 26,
                captures: None,
                is_pawn_double: true,
                is_castle: false,
            })
            .unwrap();

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
        test_board
            .make_move(Move {
                start: 58,
                target: 32,
                captures: None,
                is_pawn_double: false,
                is_castle: false,
            })
            .unwrap();

        // White does something else (h2h3), meaning the opportunity for black to take 24 with a pawn should be lost by this next move
        test_board
            .make_move(Move {
                start: 14,
                target: 21,
                captures: None,
                is_pawn_double: false,
                is_castle: false,
            })
            .unwrap();

        // Black to move. Check if they can attack square 24 where we just en passanted to
        // There is no other legal captures on the square besides en passant
        assert_eq!(
            test_board.capture_bitboard[Team::Black as usize]
                .state
                .view_bits::<Lsb0>()
                .get(26)
                .expect("Piece Bitboard did not extend to 25 bits")
                .then_some(1),
            None,
            "En passant test failed - you can still capture after a turn"
        );
    }

    #[test]

    // No castling in check
    fn check_castling() {
        let test_board = BoardState::from_fen(String::from(
            "rnb1kbnr/ppp2ppp/8/3p4/3pP3/3B1N2/PPP2qPP/RNBQK2R w KQkq - 0 1",
        ))
        .expect("Invalid FEN used in testing");

        let ept = test_board.capture_bitboard[Team::White as usize]
            .state
            .view_bits::<Lsb0>()
            .get(6)
            .expect("Piece Bitboard did not extend to 25 bits")
            .then_some(1);

        println!("{}", test_board.capture_bitboard[Team::White as usize]);

        assert_eq!(ept, None, "Castled while in check");
    }

    // Check
    #[test]
    fn check() {
        let test_board = BoardState::from_fen(String::from(
            "rnb1kbnr/ppp2ppp/8/3p4/3pP3/3B1N2/PPP2qPP/RNBQK2R w KQkq - 0 1",
        ))
        .expect("Invalid FEN used in testing");
        assert_eq!(
            test_board.is_team_checked(Team::White),
            true,
            "Check doesn't work"
        );
    }

    #[test]
    fn al_notation() {
        assert_eq!(Bitboard::bit_idx_to_al_notation(37), Some(String::from("f5")), "Bit index to al notation returned incorrectly");
        assert_eq!(Bitboard::al_notation_to_bit_idx("f5"), Some(37), "Algorithmic notation to bit index returned incorrectly");
    }
    #[test]
    fn fen() {
        let fen = String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        let test_board = BoardState::from_fen(fen.clone()).expect("Invalid FEN string used");

        assert_eq!(fen, test_board.as_fen(), "Fen conversion failed")
    }
    #[test]
    fn unmake_move() {
        let mut start_board = BoardState::from_fen(String::from(
            "8/7p/8/5r2/P3K2k/1P4p1/2P5/8 w - - 0 40",
        )).expect("Invalid FEN used in testing");

        let compare_board = start_board.clone();

        let move_to_reverse = Move {
            start: Bitboard::al_notation_to_bit_idx("e4").unwrap(),
            target: Bitboard::al_notation_to_bit_idx("f5").unwrap(),
            captures: start_board.get_piece_at_pos(Bitboard::al_notation_to_bit_idx("f5").unwrap()), 
            is_pawn_double: false,
            is_castle: false,
        };

        start_board.dump_positions();
        start_board.make_move(move_to_reverse).unwrap();
        println!("{} COMP {}", start_board.as_fen(), compare_board.as_fen());
        start_board.unmake_move(move_to_reverse).unwrap();
        assert_eq!(
            start_board.as_fen(),
            compare_board.as_fen(),
            "Unmaking one move created a different board state than the initial board"
        );
    }
}
