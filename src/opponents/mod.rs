// TODO: Add an opponent module that just seeks a UCI protocol response from a websocket
// TODO: Add a timer that is passed to the opponent

use std::{
    cmp::Ordering,
    collections::HashMap,
    time::{Duration, Instant},
};

use rand::{Rng, rng, seq::IndexedRandom};

use crate::{
    bitboard::{Bitboard, PieceType, Team},
    board::{self, BoardState},
    r#move::{self, Move},
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

#[derive(Debug, Copy, Clone)]
pub struct NegamaxEval {
    eval: i32,
    legal_move: Move,
}
#[derive(Debug, Copy, Clone)]
pub enum ChessOpponent {
    Randy,
    Matt(i32),
    Ada(Duration),
}

fn pick_random_move(mut board: BoardState) -> Option<Move> {
    let legals = board.prune_moves_for_team(board.get_legal_moves(), board.active_team);
    legals.choose(&mut rand::rng()).copied()
}

fn evaluate_move(
    board: &mut BoardState,
    ava_move: Move,
    search_budget: i32,
    mut best_white: i32,
    mut best_black: i32,
) -> i32 {
    // SUPER EXPENSIVE to recurse over it
    let virtual_board = &mut board.clone();
    let who_to_play = if virtual_board.active_team == Team::White {
        1
    } else {
        -1
    };

    let risky = virtual_board.opponent_attacking_square(ava_move.target);

    let mut eval_score = 0;
    if risky {
        let score_pt = SCORES.iter().position(|(piece_type, _scre)| {
            piece_type == &virtual_board.piece_list[ava_move.start]
        });

        eval_score -= SCORES[score_pt.unwrap()].1 * who_to_play;
    }

    let _ = virtual_board.make_move(ava_move);
    let legals_all = virtual_board.get_legal_moves();
    let legals = virtual_board
    .prune_moves_for_team(virtual_board.get_legal_moves(), virtual_board.active_team);


    eval_score += evaluate(virtual_board, legals_all);

    if ava_move.is_castle {
        eval_score += 580 * who_to_play
    }

    let mut jiggle = rand::rng().random_range(-30..30);

    if virtual_board.ply_clock > 6 {
        jiggle = rand::rng().random_range(-70..70);
    }
    if search_budget == 0 {
        return eval_score + jiggle;
    }

    if virtual_board.active_team == Team::White {
        let mut max = i32::MIN;

        for legal_move in legals {
            let move_score = evaluate_move(
                &mut virtual_board.clone(),
                legal_move,
                search_budget - 1,
                best_white,
                best_black,
            );

            max = max.max(move_score);
            best_white = best_white.max(move_score);

            if best_white >= best_black {
                break;
            }
        }
        return max;
    } else {
        let mut min = i32::MAX;
        for legal_move in legals {
            let move_score = evaluate_move(
                &mut virtual_board.clone(),
                legal_move,
                search_budget - 1,
                best_white,
                best_black,
            );
            min = min.min(move_score);
            best_black = best_black.min(move_score);

            if best_white <= best_black {
                break;
            }
        }
        return min;
    }
}
fn evaluate_team(board: &BoardState, team: Team, legal_moves: Vec<Move>) -> i32 {
    let mut material = 0;
    for (idx, piece) in board.piece_list.iter().enumerate() {
        if board.get_square_team(idx).unwrap_or(Team::None) == team {
            let score_pt = SCORES
                .iter()
                .position(|(piece_type, _scre)| piece_type == piece);

            material += SCORES[score_pt.unwrap()].1;
        }
    }

    // Rewards mobility, but kind of expensive
    let available_moves = board.clone().prune_moves_for_team(board.get_legal_moves(), team);
    material += available_moves.len() as i32 * 10;

    material
        + (if board.active_team_checkmate && board.active_team != team {
            100000000
        } else {
            0
        })
}
fn evaluate(board: &BoardState, all_moves: Vec<(Bitboard, Vec<Move>)>) -> i32 {
    let wl = board.prune_moves_for_team(all_moves.clone(), Team::White);
    let bl = board.prune_moves_for_team(all_moves, Team::Black);
    let white_eval = evaluate_team(board, Team::White, wl);
    let black_eval = evaluate_team(board, Team::Black, bl);

    return white_eval - black_eval;
}

pub trait MoveComputer {
    fn get_move(&mut self, board: BoardState) -> Option<Move>;
}

impl MoveComputer for ChessOpponent {
    fn get_move(&mut self, board: BoardState) -> Option<Move> {
        let mut board = board.clone();
        let result = match self {
            ChessOpponent::Randy => pick_random_move(board),
            ChessOpponent::Ada(time_limit) => {
                let mut legals =
                    board.prune_moves_for_team_mut(board.get_legal_moves(), board.active_team);
                let mut current_best: Option<NegamaxEval> = None;
                let mut current_worst: Option<NegamaxEval> = None;
                let start_time = Instant::now();

                if board.active_team_checkmate {
                    return None;
                }
                if legals.len() == 1 {
                    return Some(legals[0]);
                }
                let mut search_budget = 1;
                loop {
                    let mut mapped_legals = Vec::new();
                    let mut will_break = false;
                    // Check the current best first

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
                    });
                    'legal_check: for legal_move in &legals {
                        if Instant::now().duration_since(start_time) > *time_limit {
                            will_break = true;
                            break 'legal_check;
                        };

                        // Preset the AB pruning with the eval we already have
                        let (mut best_white, mut best_black) = (i32::MIN, i32::MAX);

                        if let Some(cb) = current_best {
                            if board.active_team == Team::White {
                                best_white = cb.eval;
                            } else if board.active_team == Team::Black {
                                best_black = -cb.eval;
                            }
                        }

                        if let Some(cb) = current_worst {
                            if board.active_team == Team::White {
                                best_white = -cb.eval;
                            } else if board.active_team == Team::Black {
                                best_black = cb.eval;
                            }
                        }
                        let eval = evaluate_move(
                            &mut board.clone(),
                            *legal_move,
                            search_budget - 1,
                            best_white,
                            best_black,
                        ) * if board.active_team == Team::White {
                            1
                        } else {
                            -1
                        };

                        mapped_legals.push(NegamaxEval {
                            eval: eval,
                            legal_move: *legal_move,
                        })
                    }

                    mapped_legals.sort_by(|a, b| b.eval.cmp(&a.eval));
                    if mapped_legals.len() > 0 {
                        if let Some(current_best_move) = current_best {
                            current_best = if current_best_move.eval < mapped_legals[0].eval {
                                Some(mapped_legals[0])
                            } else {
                                current_best
                            };
                        } else {
                            current_best = Some(mapped_legals[0]);
                        }

                        if let Some(current_worst_move) = current_worst {
                            current_worst =
                                if current_worst_move.eval > mapped_legals.last().unwrap().eval {
                                    mapped_legals.last().copied()
                                } else {
                                    current_best
                                };
                        } else {
                            current_worst = mapped_legals.last().copied();
                        }

                        println!(
                            "\n Best move ply {search_budget}: {:?}",
                            mapped_legals[0].eval
                        );
                        println!("Mapped legals ply {search_budget}: {:?}", mapped_legals);
                    } else {
                        if let Some(current_best_move) = current_best {
                            mapped_legals.push(current_best_move);
                        }
                    }
                    if will_break {
                        break;
                    };
                    search_budget += 1;
                }

                if current_best.is_some() {
                    println!(
                        "Within limit of {:?} Ada got to ply {search_budget} eval: {}",
                        time_limit,
                        current_best.unwrap().eval
                    );
                    Some(current_best.unwrap().legal_move)
                } else {
                    None
                }
            }
            ChessOpponent::Matt(search_budget) => {
                let legals = board.prune_moves_for_team(board.get_legal_moves(), board.active_team);
                let mut mapped_legals = Vec::new();
                if legals.len() == 1 {
                    return Some(legals[0]);
                }
                // expensive...
                for legal_move in legals {
                    let eval = evaluate_move(
                        &mut board.clone(),
                        legal_move,
                        *search_budget - 1,
                        i32::MIN,
                        i32::MAX,
                    ) * if board.active_team == Team::White {
                        1
                    } else {
                        -1
                    };

                    mapped_legals.push(NegamaxEval {
                        eval: eval,
                        legal_move: legal_move,
                    })
                }

                mapped_legals.sort_by(|a, b| b.eval.cmp(&a.eval));
                println!(
                    "Evaled to: {} and {}",
                    mapped_legals[0].eval,
                    mapped_legals[mapped_legals.len() - 1].eval
                );

                if mapped_legals.len() > 0 {
                    Some(mapped_legals[0].legal_move)
                } else {
                    None
                }
            }
        };

        if result.is_some() { result } else { None }
    }
}
