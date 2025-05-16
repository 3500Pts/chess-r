// TODO: Add an opponent module that just seeks a UCI protocol response from a websocket
// TODO: Add a timer that is passed to the opponent

use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    time::{Duration, Instant},
};

use rand::{seq::IndexedRandom, Rng};

use crate::{
    bitboard::{Bitboard, PieceType, Team},
    board::BoardState,
    r#move::{self, Move, MoveError, Piece},
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
    let legals = board.prune_moves_for_team(board.get_legal_moves(), board.active_team);
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
fn eval_max(
    board: &mut BoardState,
    ava_move: Move,
    search_budget: i32,
    mut best_white: i32,
    mut best_black: i32,
) {
}
fn evaluate_move(
    board: &mut BoardState,
    ava_move: Move,
    search_budget: i32,
    mut best_white: i32,
    mut best_black: i32,
) -> i32 {
    // SUPER EXPENSIVE to recurse over it
    let virtual_board = board;
    let who_to_play = if virtual_board.active_team == Team::White {
        1
    } else {
        -1
    };

    let risky = virtual_board.opponent_attacking_square(ava_move.target);

    let cap_score_idx = SCORES.iter().position(|(piece_type, _scre)| {
        piece_type
            == &ava_move
                .captures
                .unwrap_or(Piece {
                    piece_type: PieceType::None,
                    position: ava_move.target,
                    team: virtual_board.active_team.opponent(),
                })
                .piece_type
    });

    let capture_score = SCORES[cap_score_idx.unwrap()].1;
    let piece_score = {
        let score_pt = SCORES.iter().position(|(piece_type, _scre)| {
            piece_type == &virtual_board.piece_list[ava_move.start]
        });
        SCORES[score_pt.unwrap()].1 * who_to_play
    };

    let good_trade = capture_score - piece_score > 0;

    let sacrifice_score = {
        let score_pt = SAC_SCORES.iter().position(|(piece_type, _scre)| {
            piece_type == &virtual_board.piece_list[ava_move.start]
        });
        SAC_SCORES[score_pt.unwrap()].1 * who_to_play
    };

    let mut eval_score = 0;

    handle_move_result(
        "MOVE",
        virtual_board.make_move(ava_move),
        ava_move,
        search_budget,
        virtual_board,
    );
    let legals_all = virtual_board.get_legal_moves();
    let legals = virtual_board.prune_moves_for_team(legals_all.clone(), virtual_board.active_team);

    eval_score += evaluate(virtual_board, legals_all);

    if risky && !good_trade {
        //eval_score -= sacrifice_score
    }

    if ava_move.is_castle {
        eval_score += 1200 * who_to_play
    }

    if virtual_board.active_team_checkmate {
        eval_score -= 100000000 * who_to_play;
    }
    let center_control_bits = Bitboard {
        state: 0x1818000000,
    };
    let center_control =
        virtual_board.get_team_coverage(virtual_board.active_team) & center_control_bits;
    if center_control.state > 0 {
        // For some reason negatively attributing it makes it focus on the center
        //  eval_score -= (center_control.state.count_ones() as i32) * who_to_play * 3;
    }

    let forking = virtual_board.capture_bitboard[virtual_board.active_team as usize]
        & virtual_board.get_team_coverage(virtual_board.active_team.opponent());

    if forking.state.count_ones() > 1 {
        eval_score += 50 * (forking.state.count_ones() as i32) * who_to_play;
    }

    let jiggle = 0; //rand::rng().random_range(-1..1);

    if virtual_board.ply_clock > 6 {
        //jiggle = rand::rng().random_range(-70..70);
    }
    if search_budget == 0 {
        handle_move_result(
            "UNMOVE",
            virtual_board.unmake_move(ava_move),
            ava_move,
            search_budget,
            virtual_board,
        );
        return eval_score + jiggle;
    }

    if virtual_board.active_team == Team::White {
        let mut max = i32::MIN;

        for legal_move in legals {
            let move_score = evaluate_move(
                virtual_board,
                legal_move,
                search_budget - 1,
                best_white,
                best_black,
            );
            max = max.max(best_white);
            //println!("W{best_black}, {best_white} {search_budget}");
            if move_score >= best_black {
                break;
            }
            best_white = best_white.max(move_score);
        }
        handle_move_result(
            "UNMOVE",
            virtual_board.unmake_move(ava_move),
            ava_move,
            search_budget,
            virtual_board,
        );
        max
    } else {
        let mut min = i32::MAX;
        for legal_move in legals {
            let move_score = evaluate_move(
                virtual_board,
                legal_move,
                search_budget - 1,
                best_white,
                best_black,
            );
            min = min.min(best_black);
            // println!("B{best_white}, {best_black} {search_budget}");
            if move_score <= best_white {
                break;
            }
            best_black = best_black.min(move_score);
        }
        handle_move_result(
            "UNMOVE",
            virtual_board.unmake_move(ava_move),
            ava_move,
            search_budget,
            virtual_board,
        );
        min
    }
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
fn evaluate(board: &BoardState, all_moves: Vec<(Bitboard, Vec<Move>)>) -> i32 {
    let wl = board.prune_moves_for_team(all_moves.clone(), Team::White);
    let bl = board.prune_moves_for_team(all_moves, Team::Black);
    let white_eval = evaluate_team(board, Team::White, wl);
    let black_eval = evaluate_team(board, Team::Black, bl);

    white_eval - black_eval
}
impl fmt::Display for ChessOpponent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
pub trait MoveComputer {
    fn get_move(&mut self, board: BoardState) -> Option<Move>;
}

impl MoveComputer for ChessOpponent {
    fn get_move(&mut self, board: BoardState) -> Option<Move> {
        let mut board = board;
        let result = match self {
            ChessOpponent::Randy => pick_random_move(board),
            ChessOpponent::Ada(time_limit) => {
                let mut legals =
                    board.prune_moves_for_team_mut(board.get_legal_moves(), board.active_team);
                let mut current_best: Option<NegamaxEval> = None;
                let current_worst: Option<NegamaxEval> = None;
                let start_time = Instant::now();

                if board.active_team_checkmate {
                    return None;
                }
                if legals.len() == 1 {
                    return Some(legals[0]);
                }
                let mut search_budget = 0;
                let mut mapped_legals = EvaluationList(Vec::new());
                loop {
                    let mut evals: EvaluationList = EvaluationList(Vec::new());

                    let mut will_break = false;
                    /* Check the current best first
                    legals.sort_by(|c_a, c_b| {
                        if let Some(cb) = current_best {
                            if cb.legal_move == *c_a && cb.legal_move != *c_b {
                                Ordering::Less
                            } else {
                                Ordering::Equal
                            }
                        } else {
                            Ordering::Equal
                        }
                    });*/
                    let (best_white, best_black) = (i32::MIN, i32::MAX);
                    'legal_check: for legal_move in &legals {
                        // Preset the AB pruning with the eval we already have

                        if Instant::now().duration_since(start_time) > *time_limit {
                            will_break = true;
                            break 'legal_check;
                        };

                        let eval = evaluate_move(
                            &mut board,
                            *legal_move,
                            search_budget,
                            best_white,
                            best_black,
                        ) * if board.active_team == Team::White {
                            1
                        } else {
                            -1
                        };

                        evals.0.push(NegamaxEval {
                            eval: eval + rand::rng().random_range(-2..=2),
                            legal_move: *legal_move,
                        })
                    }
                    if will_break {
                        break;
                    };
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

                    /*if let Some(current_worst_move) = current_worst {
                        current_worst =
                            if current_worst_move.eval > mapped_legals.0.last().unwrap().eval {
                                mapped_legals.0.last().copied()
                            } else {
                                current_best
                            };
                    } else {
                        current_worst = mapped_legals.0.last().copied();
                    }*/

                    tracing::warn!(
                        "\n Best move ply {search_budget}: {:?}",
                        mapped_legals.0[0].eval
                    );
                    tracing::warn!("Mapped legals ply {search_budget}: {}", mapped_legals);
                } else if let Some(current_best_move) = current_best {
                    mapped_legals.0.push(current_best_move);
                }

                if current_best.is_some() {
                    tracing::warn!(
                        "Within limit of {:?} Ada got to ply {search_budget} eval: {}",
                        time_limit,
                        current_best.unwrap()
                    );
                    Some(current_best.unwrap().legal_move)
                } else {
                    None
                }
            }
            ChessOpponent::Matt(search_budget) => {
                let legals = board.prune_moves_for_team(board.get_legal_moves(), board.active_team);
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
