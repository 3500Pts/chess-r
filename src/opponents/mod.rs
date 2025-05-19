// TODO: Add an opponent module that just seeks a UCI protocol response from a websocket
// TODO: Add a timer that is passed to the opponent

use std::{
    fmt::{self, Display, Formatter},
    time::{Duration, Instant},
};

use bitvec::order::Lsb0;
use rand::{seq::IndexedRandom, Rng};

use crate::{
    bitboard::{Bitboard, PieceType, Team},
    board::BoardState,
    r#move::{Move, MoveError},
};

const SCORES: [(PieceType, i32); 7] = [
    (PieceType::None, 0),
    (PieceType::Pawn, 100),
    (PieceType::Knight, 300),
    (PieceType::Bishop, 300),
    (PieceType::Rook, 500),
    (PieceType::Queen, 900),
    (PieceType::King, 1000000),
];
const SAC_SCORES: [(PieceType, i32); 7] = [
    (PieceType::None, 0),
    (PieceType::Pawn, 50),
    (PieceType::Knight, 150),
    (PieceType::Bishop, 150),
    (PieceType::Rook, 350),
    (PieceType::Queen, 1100),
    (PieceType::King, 1000000),
];
#[derive(Debug, Copy, Clone)]
struct NegamaxEval {
    eval: i32,
    legal_move: Move,
}
impl Display for NegamaxEval {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}e)", self.legal_move, self.eval)?;
        Ok(())
    }
}
#[derive(Debug, Clone)]
struct EvaluationList(Vec<NegamaxEval>);
impl Display for EvaluationList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "EvaluationList {{")?;
        for v in &self.0 {
            write!(f, "{}, ", v)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
#[derive(Debug, Copy, Clone)]
pub enum ChessOpponent {
    Randy,
    Matt(i32),
    Ada(Duration),
}

fn pick_random_move(board: BoardState) -> Option<Move> {
    let legals = board
        .get_legal_moves()
        .to_vec()
        .convert_to_moves_and_mask_atk_squares(board.get_team_coverage(board.active_team), &board);
    legals.choose(&mut rand::rng()).copied()
}

fn handle_move_result(
    result_type: &str,
    result: Result<(), MoveError>,
    ava_move: Move,
    search_budget: i32,
    virtual_board: &BoardState,
) {
    if let Err(vm_err) = result {
        tracing::info!(
            "RECURSIVE {result_type} at search budget {search_budget}: {vm_err:?}; MOVE: {ava_move}"
        );
        BoardState::render_piece_list(virtual_board.piece_list.to_vec());
        tracing::info!("{}", virtual_board.get_team_coverage(Team::White))
    }
}

fn evaluate_move(
    board: &mut BoardState,
    ava_move: Move,
    search_budget: i32,
    mut best_self: i32,
    best_opponent: i32,
) -> i32 {
    // SUPER EXPENSIVE to recurse over it
    let virtual_board = board;
    let who_to_play = if virtual_board.active_team == Team::White {
        1
    } else {
        1
    };

    let risky = virtual_board.opponent_attacking_square(ava_move.target);

    let piece_score = {
        let score_pt = SCORES.iter().position(|(piece_type, _scre)| {
            piece_type == &virtual_board.piece_list[ava_move.start]
        });
        SCORES[score_pt.unwrap()].1 * who_to_play
    };

    let good_trade = false; //capture_score - piece_score > 0;

    let sacrifice_score = {
        let score_pt = SAC_SCORES.iter().position(|(piece_type, _scre)| {
            piece_type == &virtual_board.piece_list[ava_move.start]
        });
        SAC_SCORES[score_pt.unwrap()].1 * who_to_play
    };

    let mut eval_score = 0;
    let active_team = virtual_board.active_team;
    let active_team_mate = virtual_board.active_team_mate;
    handle_move_result(
        "MOVE",
        virtual_board.make_move(ava_move),
        ava_move,
        search_budget,
        virtual_board,
    );
    let legals = virtual_board
        .get_legal_moves()
        .to_vec()
        .convert_to_moves_and_mask_atk_squares(
            virtual_board.get_team_coverage(active_team),
            virtual_board,
        );
    eval_score += evaluate(virtual_board, vec![]);

    if ava_move.is_castle {
        eval_score += 200 * who_to_play
    }

    if virtual_board.active_team_mate && virtual_board.is_team_checked(virtual_board.active_team) {
        // This is the other team!
        eval_score += 100000000 * who_to_play;
    }
    if virtual_board.active_team_mate && !virtual_board.is_team_checked(virtual_board.active_team) {
        // FUCK draws
        // This is the other team!
        eval_score -= 100000000 * who_to_play;
    }
    let center_control_bits = Bitboard {
        state: 0x1818000000,
    };
    let center_control = virtual_board.get_team_coverage(active_team) & center_control_bits;
    if center_control.state > 0 {
        eval_score += (center_control.state.count_ones() as i32) * 3;
    }

    if search_budget == 0 {
        handle_move_result(
            "UNMOVE",
            virtual_board.unmake_move(ava_move),
            ava_move,
            search_budget,
            virtual_board,
        );
        return eval_score + 0; //+ jiggle;
    }
    let mut best = i32::MIN;

    for legal_move in legals {
        let move_score = -evaluate_move(
            virtual_board,
            legal_move,
            search_budget - 1,
            -best_opponent.checked_neg().unwrap_or(i32::MAX),
            best_self.checked_neg().unwrap_or(i32::MIN),
        );
        best = best.max(move_score);
        best_self = best_self.max(best);

        if move_score >= best_opponent {
            // Opponent would never let this move happen
            break;
        }
    }

    handle_move_result(
        "UNMOVE",
        virtual_board.unmake_move(ava_move),
        ava_move,
        search_budget,
        virtual_board,
    );

    best
}
fn evaluate_team(board: &BoardState, team: Team, available_moves: Vec<Move>) -> i32 {
    let mut material = 0;
    for (idx, piece) in board.piece_list.iter().enumerate() {
        if board.get_square_team(idx) == team {
            let score_pt = SCORES
                .iter()
                .position(|(piece_type, _scre)| piece_type == piece);

            material += SCORES[score_pt.unwrap()].1;
        }
    }

    // Rewards mobility, but kind of expensive
    material
}
fn evaluate(board: &BoardState, _all_moves: Vec<(Bitboard, Vec<Move>)>) -> i32 {
    let wl = Vec::new();
    let bl = Vec::new();
    let white_eval = evaluate_team(board, Team::White, wl);
    let black_eval = evaluate_team(board, Team::Black, bl);

    white_eval - black_eval
}
impl fmt::Display for ChessOpponent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub trait ConvertToMoves {
    fn convert_to_moves_and_mask_atk_squares(
        &self,
        bitboard: Bitboard,
        board: &BoardState,
    ) -> Vec<Move>;
    fn convert_to_moves_with_changes<F: FnMut(&Bitboard, &BoardState, usize) -> Bitboard>(
        &self,
        board: &BoardState,
        f: F,
    ) -> Vec<Move>;
    fn convert_to_moves(&self, board: &BoardState) -> Vec<Move>;
    fn convert_to_moves_and_bitboard(&self, bitboard: Bitboard, board: &BoardState) -> Vec<Move>;
}

impl ConvertToMoves for Vec<Bitboard> {
    fn convert_to_moves_with_changes<F>(&self, board: &BoardState, mut f: F) -> Vec<Move>
    where
        F: FnMut(&Bitboard, &BoardState, usize) -> Bitboard,
    {
        self.to_vec()
            .iter()
            .enumerate()
            .map(|(attacker_square, bitboard_to_map_to_vec)| {
                let final_board = f(bitboard_to_map_to_vec, board, attacker_square);
                let moves = board
                    .get_piece_at_pos(attacker_square)
                    .and_then(|attacking_piece| {
                        Some(final_board.as_moves(&attacking_piece, attacking_piece.team, &board))
                    })
                    .unwrap_or(vec![Move::default(); 0]);

                moves
            })
            .flatten()
            .collect()
    }
    fn convert_to_moves(&self, board: &BoardState) -> Vec<Move> {
        self.convert_to_moves_with_changes(board, |bb, _, _| *bb)
    }
    fn convert_to_moves_and_mask_atk_squares(
        &self,
        bitboard: Bitboard,
        board: &BoardState,
    ) -> Vec<Move> {
        self.convert_to_moves_with_changes(board, |main_bitboard, _, atk_square| {
            if bitboard.get_bit::<Lsb0>(atk_square) {
                return *main_bitboard;
            } else {
                return Bitboard { state: 0 };
            }
        })
    }
    fn convert_to_moves_and_bitboard(&self, bitboard: Bitboard, board: &BoardState) -> Vec<Move> {
        self.convert_to_moves_with_changes(board, |bb, _, _| *bb & bitboard)
    }
}
pub trait MoveComputer {
    fn get_move(&mut self, board: BoardState) -> Option<Move>;
}

impl MoveComputer for ChessOpponent {
    fn get_move(&mut self, board: BoardState) -> Option<Move> {
        let mut board = board;
        let legal_moves = board.get_legal_moves();

        let result = match self {
            ChessOpponent::Randy => pick_random_move(board),
            ChessOpponent::Ada(time_limit) => {
                let legals = legal_moves.to_vec().convert_to_moves_and_mask_atk_squares(
                    board.get_team_coverage(board.active_team),
                    &board,
                );
                let mut current_best: Option<NegamaxEval> = None;
                let start_time = Instant::now();

                if board.active_team_mate || legals.len() == 0 {
                    return None;
                }
                if legals.len() == 1 {
                    return Some(legals[0]);
                }
                let mut search_budget = 0;
                let time_actually_spent: Duration;
                let mut mapped_legals: EvaluationList = EvaluationList(Vec::new());

                'eval: loop {
                    let mut evals: EvaluationList = EvaluationList(Vec::new());
                    let (best_white, best_black) = (i32::MAX, i32::MIN);
                    for legal_move in &legals {
                        let eval = evaluate_move(
                            &mut board,
                            *legal_move,
                            search_budget,
                            best_white,
                            best_black,
                        );

                        evals.0.push(NegamaxEval {
                            eval: eval + rand::rng().random_range(-1..=1),
                            legal_move: *legal_move,
                        });

                        let elapsed = Instant::now().duration_since(start_time);

                        if elapsed >= *time_limit {
                            time_actually_spent = elapsed;
                            break 'eval;
                        };
                    }

                    mapped_legals = evals;

                    search_budget += 1;
                }

                mapped_legals.0.sort_by(|a, b| b.eval.cmp(&a.eval));
                if !mapped_legals.0.is_empty() {
                    if let Some(current_best_move) = current_best {
                        current_best = if current_best_move.eval < mapped_legals.0[0].eval {
                            Some(mapped_legals.0[0])
                        } else {
                            current_best
                        };
                    } else {
                        current_best = Some(mapped_legals.0[0]);
                    }

                    tracing::warn!(
                        "\n Best move ply {search_budget}: {:?}",
                        mapped_legals.0[0].eval
                    );
                    tracing::warn!("Mapped legals ply {search_budget}: {}", mapped_legals);
                } else if let Some(current_best_move) = current_best {
                    mapped_legals.0.push(current_best_move);
                }
                if let Some(best_move) = current_best {
                    tracing::warn!(
                        "Within limit of {:?} (actual time {:?}) Ada got to ply {search_budget} eval: {}",
                        time_limit,
                        time_actually_spent,
                        best_move
                    );
                    Some(best_move.legal_move)
                } else {
                    None
                }
            }
            ChessOpponent::Matt(search_budget) => {
                let legals = board
                    .get_legal_moves()
                    .to_vec()
                    .convert_to_moves_and_mask_atk_squares(
                        board.get_team_coverage(board.active_team),
                        &board,
                    );

                let mut mapped_legals: EvaluationList = EvaluationList(Vec::new());
                if legals.len() == 1 {
                    return Some(legals[0]);
                }
                // expensive...
                let (best_white, best_black) = (i32::MIN, i32::MAX);

                for legal_move in legals {
                    let eval = evaluate_move(
                        &mut board.clone(),
                        legal_move,
                        *search_budget - 1,
                        best_white,
                        best_black,
                    ) * if board.active_team == Team::White {
                        1
                    } else {
                        -1
                    };

                    mapped_legals.0.push(NegamaxEval { eval, legal_move })
                }

                mapped_legals.0.sort_by(|a, b| b.eval.cmp(&a.eval));
                tracing::debug!(
                    "Evaled to: {} and {}",
                    mapped_legals.0[0].eval,
                    mapped_legals.0[mapped_legals.0.len() - 1].eval
                );

                if !mapped_legals.0.is_empty() {
                    Some(mapped_legals.0[0].legal_move)
                } else {
                    None
                }
            }
        };

        if result.is_some() {
            result
        } else {
            None
        }
    }
}
