use std::collections::HashMap;
use std::sync::mpsc::Receiver;

use bitvec::order::Lsb0;
use bitvec::view::BitView;
use ggez::GameError;
use ggez::audio::SoundSource;
use ggez::audio::Source;
use ggez::event;
use ggez::glam::*;
use ggez::graphics::Canvas;
use ggez::graphics::DrawParam;
use ggez::graphics::Image;
use ggez::graphics::Rect;
use ggez::graphics::Text;
use ggez::graphics::Transform;
use ggez::graphics::{self, Color};
use ggez::mint::Point2;
use ggez::mint::Vector2;
use ggez::{Context, GameResult};

use crate::bitboard::Bitboard;
use crate::bitboard::PIECE_TYPE_ARRAY;
use crate::bitboard::PieceType;
use crate::bitboard::Team;
use crate::board::BoardState;
use crate::r#move::Move;
use crate::opponents::*;
use chrono::prelude::*;

pub type ColorRGBA = [f32; 4];

const BLACK: ColorRGBA = [0.2, 0.2, 0.2, 1.0];
const SELECTED_SQUARE_COLOR: ColorRGBA = [1.0, 1.0, 1.0, 1.0];
const OLD_MOVE_COLOR: ColorRGBA = [1.0, 0.8, 0.25, 1.0];
const LEGAL_MOVE_COLOR_LERP: f32 = 0.3;
const LIGHT_SQUARE_COLOR: ColorRGBA = [0.941, 0.467, 0.467, 1.0];
const DARK_SQUARE_COLOR: ColorRGBA = [0.651, 0.141, 0.141, 1.0];
const WIDTH: f32 = 600.0;
const SQUARE_SIZE: f32 = WIDTH / 8.0;
const FLAG_DEBUG_UI_COORDS: bool = false;

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
pub fn color_lerp(left: Color, right: Color, t: f32) -> Color {
    Color::from([
        lerp(left.r, right.r, t),
        lerp(left.g, right.g, t),
        lerp(left.b, right.b, t),
        lerp(left.a, right.a, t),
    ])
}

#[derive(Clone, Copy)]
pub struct MoveHistoryEntry {
    piece_type: PieceType,
    team: Team,
    captures: bool,
    checks: bool,
    mate: bool,
    target: usize,
    start: usize,
    castle: bool,
}
impl MoveHistoryEntry {
    pub fn to_string(self) -> String {
        // TODO: Piece disambiguation

        let piece_id = match self.piece_type {
            PieceType::None => "",
            PieceType::Pawn => "",
            PieceType::Knight => "n",
            PieceType::Queen => "q",
            PieceType::King => "k",
            PieceType::Rook => "r",
            PieceType::Bishop => "b",
        }
        .to_uppercase();

        let capture_string = if self.captures { "x" } else { "" };
        let file_array = ["a", "b", "c", "d", "e", "f", "g", "h"];
        let target_file = file_array[self.target % 8];
        let target_rank = (((self.target / 8) as i32) + 1).to_string();
        let append_string = if self.mate {
            "#"
        } else if self.checks {
            "+"
        } else {
            ""
        };

        if self.castle {
            let diff = self.target as i32 - self.start as i32;
            if diff < 0 {
                return String::from("O-O-O");
            } else {
                return String::from("O-O");
            }
        }
        format!("{piece_id}{capture_string}{target_file}{target_rank}{append_string}")
    }
}

pub struct MainState {
    pub board: BoardState,
    pub piece_imgs: HashMap<String, Image>,
    pub sound_sources: HashMap<String, Source>,
    pub selected_square: Option<usize>,
    pub queued_move: Option<Move>, // Moves are queued to the draw queue so nothing changes during drawing
    pub drag_x: Option<f32>,
    pub drag_y: Option<f32>,
    pub board_legal_moves: Option<Vec<(Bitboard, Vec<Move>)>>,
    pub last_move_origin: Option<usize>,
    pub last_move_end: Option<usize>,
    pub player_team: Team,
    pub opp_thread: Option<Receiver<Option<Move>>>,
    pub opponent: ChessOpponent,
    pub move_history: Vec<MoveHistoryEntry>, // for PGN
    pub start_board: BoardState,
}

impl MainState {
    pub fn new(
        board_state: BoardState,
        ctx: &mut Context,
        plr_team: Team,
        opponent: ChessOpponent,
    ) -> GameResult<MainState> {
        let mut s = MainState {
            board: board_state,
            piece_imgs: HashMap::new(),
            sound_sources: HashMap::new(),
            selected_square: None,
            queued_move: None,
            drag_x: None,
            drag_y: None,
            board_legal_moves: None,
            last_move_origin: None,
            last_move_end: None,
            player_team: plr_team,
            opponent,
            opp_thread: None,
            move_history: Vec::new(),
            start_board: board_state,
        };
        s.board_legal_moves = Some(s.board.get_legal_moves());
        // Preload piece data for speed - pulling it every frame is slow as I learned the hard way

        let mut piece_ids: Vec<String> = Vec::new();

        PIECE_TYPE_ARRAY.iter().for_each(|p| {
            // So we know there is a piece, we can just match its type now
            let piece_id = match p {
                PieceType::Pawn => "p",
                PieceType::Knight => "n",
                PieceType::Rook => "r",
                PieceType::Queen => "q",
                PieceType::King => "k",
                PieceType::Bishop => "b",
                PieceType::None => "-",
            };

            if piece_id != "-" {
                piece_ids.push(String::from("w") + piece_id);
                piece_ids.push(String::from("b") + piece_id);
            }
        });

        piece_ids.iter().for_each(|id| {
            let file_path = format!("/alila/{}.png", id);
            let image_res = graphics::Image::from_path(ctx, file_path);

            if let Ok(image) = image_res {
                s.piece_imgs.insert(id.to_owned(), image);
            }
        });

        // Preload sounds
        let sound_paths = [
            "bass_intro".to_string(),
            "piece_move".to_string(),
            "capture".to_string(),
        ];

        sound_paths.iter().for_each(|id| {
            let sound_source_r = Source::new(ctx, format!("/sounds/{}.ogg", id));

            if let Ok(sound_source) = sound_source_r {
                s.sound_sources.insert(id.to_owned(), sound_source);
            } else {
                sound_source_r.unwrap();
            }
        });
        Ok(s)
    }
    pub fn to_pgn(&self) {
        let current_date = Utc::now().format("%Y-%m-%d");
        let bot_name = format!("Bot {}", self.opponent);

        let white_name = if self.player_team == Team::White {
            "Player"
        } else {
            &bot_name
        };
        let black_name = if self.player_team != Team::White {
            "Player"
        } else {
            &bot_name
        };

        let result = "1-0";

        let mut pgn_header = format!(
            "[Event \"chess-r match\"]\n[Site \"chess-r\"]\n[Date \"{current_date}\"]\n[Round \"1\"]\n[White \"{white_name}\"]\n[Black \"{black_name}\"]\n[Result \"{result}\"]\n\n"
        );

        for (ply, move_data) in self.move_history.iter().enumerate() {
            let turn_string = if ply % 2 == 0 {
                format!("{}.", (ply / 2) + 1)
            } else {
                String::from("")
            };

            pgn_header.push_str(&format!("{turn_string}{} ", move_data.to_string()));
        }

        println!("{pgn_header}");
    }
    fn end_game(&self) {
        let opponent = if self.board.active_team == Team::White {
            Team::Black
        } else {
            Team::White
        };

        if self.board.is_team_checked(self.board.active_team) {
            println!("Checkmate - {opponent:?} wins");
        } else {
            println!("Stalemate");
        }
    }
    fn draw_board(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult<()> {
        for rank in 0..8 {
            for file in 0..8 {
                let square_number = 63 - (((7 - rank) * 8) + 7 - file) as usize;
                // What an unholy if statement. TODO: Make it neater maybe
                let default_color = if (rank + file) % 2 != 0 {
                    Color::from(LIGHT_SQUARE_COLOR)
                } else {
                    Color::from(DARK_SQUARE_COLOR)
                };
                let color = if Some(square_number) == self.selected_square {
                    Color::from(SELECTED_SQUARE_COLOR)
                } else if let Some(selected_square) = self.selected_square {
                    if let Some(pl_moves) = &self.board_legal_moves {
                        let status_on_bitboard = pl_moves[selected_square]
                            .0
                            .state
                            .view_bits::<Lsb0>()
                            .get(square_number);

                        let board_team = self.board.get_square_team(selected_square);
                        if status_on_bitboard.unwrap().then_some(true).is_some()
                            && self.player_team == board_team
                        {
                            color_lerp(
                                Color::from(SELECTED_SQUARE_COLOR),
                                default_color,
                                LEGAL_MOVE_COLOR_LERP,
                            )
                        } else {
                            default_color
                        }
                    } else {
                        default_color
                    }
                } else if Some(square_number) == self.last_move_origin {
                    color_lerp(Color::from(OLD_MOVE_COLOR), default_color, 0.7)
                } else if Some(square_number) == self.last_move_end {
                    color_lerp(Color::from(OLD_MOVE_COLOR), default_color, 0.3)
                } else {
                    default_color
                };

                let square_mesh = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    Rect {
                        x: file as f32 * SQUARE_SIZE,
                        y: (7 - rank) as f32 * SQUARE_SIZE,
                        h: SQUARE_SIZE,
                        w: SQUARE_SIZE,
                    },
                    color,
                )?;

                let sqr_txt = square_number.to_string();

                canvas.draw(&square_mesh, DrawParam::default());

                // DRAW DEBUG SQUARE ID TEXT
                if FLAG_DEBUG_UI_COORDS {
                    let mut text_mesh = Text::new(sqr_txt);
                    text_mesh.set_bounds(Vector2 {
                        x: SQUARE_SIZE,
                        y: SQUARE_SIZE,
                    });
                    canvas.draw(
                        &text_mesh,
                        DrawParam::default().transform({
                            Transform::Values {
                                dest: Point2 {
                                    x: file as f32 * SQUARE_SIZE,
                                    y: (7 - rank) as f32 * SQUARE_SIZE,
                                },
                                rotation: 0.0,
                                scale: Vector2 { x: 1.0, y: 1.0 },
                                offset: Point2 { x: 0.5, y: 0.5 },
                            }
                            .to_bare_matrix()
                        }),
                    )
                } else if square_number <= 7 || square_number % 8 == 0 {
                    let file_array = ["a", "b", "c", "d", "e", "f", "g", "h"];
                    let text_frag = if square_number <= 7 {
                        file_array[square_number]
                    } else {
                        &((square_number / 8) + 1).to_string()
                    };
                    let mut text_frag_str = String::from(text_frag);
                    if square_number == 0 {
                        text_frag_str.push('1');
                    }

                    let mut text_mesh = Text::new(text_frag_str);
                    text_mesh.set_bounds(Vector2 {
                        x: SQUARE_SIZE,
                        y: SQUARE_SIZE,
                    });
                    canvas.draw(
                        &text_mesh,
                        DrawParam::default().transform({
                            Transform::Values {
                                dest: Point2 {
                                    x: file as f32 * SQUARE_SIZE,
                                    y: (7 - rank) as f32 * SQUARE_SIZE,
                                },
                                rotation: 0.0,
                                scale: Vector2 { x: 1.0, y: 1.0 },
                                offset: Point2 { x: 0.5, y: 0.5 },
                            }
                            .to_bare_matrix()
                        }),
                    )
                }
            }
        }
        Ok(())
    }
    fn draw_pieces(&mut self, _ctx: &mut Context, canvas: &mut Canvas) -> GameResult<()> {
        // Map each piece and team in the game state to the image.
        // To do this, use the team bitboard to check the square's team
        // then the piece list to check the square's type

        for rank in (0..8).rev() {
            for file in 0..8 {
                let square_bit_idx = 63 - ((rank * 8) + (7 - file)) as usize;

                let square_team = self.board.get_square_team(square_bit_idx);

                if square_team != Team::None {
                    // We use the team id to compose the team part of the file name
                    let file_team =
                        String::from(if square_team == Team::White { "w" } else { "b" });
                        
                    // So we know there is a piece, we can just match its type now
                    let team_bitboard = self.board.get_team_coverage(square_team);
                    let square_piece = match self.board.piece_list[square_bit_idx] {
                        PieceType::Pawn => "p",
                        PieceType::Knight => "n",
                        PieceType::Rook => "r",
                        PieceType::Queen => "q",
                        PieceType::King => "k",
                        PieceType::Bishop => "b",
                        PieceType::None => {
                            // Should be unreachable
                            return Err(GameError::RenderError(format!(
                                "Attempted to draw a piece that does not exist for team {square_team:?}. Bitboard: {team_bitboard}",
                            )));
                        }
                    };

                    let square_piece_id = file_team + square_piece;
                    //file_team + square_piece
                    let image = self
                        .piece_imgs
                        .get(&square_piece_id)
                        .unwrap_or_else(|| panic!("Couldn't find piece png for {square_piece_id}"));

                    let piece_x = file as f32 * SQUARE_SIZE;
                    let piece_y = rank as f32 * SQUARE_SIZE;
                    let piece_x = if Some(square_bit_idx) == self.selected_square {
                        self.drag_x.unwrap_or(piece_x)
                    } else {
                        piece_x
                    };
                    let piece_y = if Some(square_bit_idx) == self.selected_square {
                        self.drag_y.unwrap_or(piece_y)
                    } else {
                        piece_y
                    };
                    canvas.draw(
                        image,
                        DrawParam::default().transform(
                            Transform::Values {
                                dest: Point2 {
                                    x: piece_x,
                                    y: piece_y,
                                },
                                rotation: 0.0,
                                scale: Vector2 {
                                    x: SQUARE_SIZE / image.width() as f32,
                                    y: SQUARE_SIZE / image.height() as f32,
                                },
                                offset: Point2 { x: 0.5, y: 0.5 },
                            }
                            .to_bare_matrix(),
                        ),
                    );
                }
            }
        }
        Ok(())
    }
    fn get_square_idx_from_pixel(x: f32, y: f32) -> f32 {
        let file = (x / SQUARE_SIZE).floor();
        let rank = (y / SQUARE_SIZE).floor();

        63.0 - ((rank * 8.0) + (7.0 - file))
    }
    fn play_sound(&mut self, ctx: &mut Context, id: &str, volume: f32) -> GameResult<()> {
        let sound = self.sound_sources.get_mut(id).unwrap();
        sound.set_volume(volume);
        sound.play(ctx)?;

        Ok(())
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.opp_thread.is_none()
            && self.player_team != self.board.active_team
            && !self.board.active_team_checkmate
        {
            let (mv_tx, mv_rx) = std::sync::mpsc::channel();
            let mut opponent_clone = self.opponent;
            let board_clone = self.board.clone();

            tokio::spawn(async move {
                let legal = opponent_clone.get_move(board_clone);
                mv_tx.send(legal).unwrap();
            });
            self.opp_thread = Some(mv_rx);
        }
        self.queued_move = if self.player_team != self.board.active_team {
            if let Some(ot) = &self.opp_thread {
                let legal = ot.try_recv();

                if let Ok(legal_move) = legal {
                    if legal_move.is_none() {
                        self.end_game();
                        self.to_pgn();
                    }
                    legal_move
                } else {
                    self.queued_move
                }
            } else {
                self.queued_move
            }
        } else {
            self.queued_move
        };

        Ok(())
    }
    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: event::MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), ggez::GameError> {
        if button == event::MouseButton::Left {
            let square_idx = MainState::get_square_idx_from_pixel(x, y) as usize;
            tracing::debug!("Mouse down on square {}", square_idx);

            // If there's a piece here, "select" the piece at this index to drag
            self.selected_square = Some(square_idx);
        }

        Ok(())
    }
    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        x: f32,
        y: f32,
        _dx: f32,
        _dy: f32,
    ) -> Result<(), ggez::GameError> {
        // Do drag effect on the piece at the currently selected square

        self.drag_x = Some(x - (0.5 * SQUARE_SIZE));
        self.drag_y = Some(y - (0.5 * SQUARE_SIZE));

        Ok(())
    }
    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: event::MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), ggez::GameError> {
        if button == event::MouseButton::Left && self.queued_move.is_none() {
            let target_square_idx = MainState::get_square_idx_from_pixel(x, y) as usize;
            tracing::debug!("Mouse up at square {}", target_square_idx);
            // Attempt a move here if it's on the bitboard

            if let Some(selected_square) = self.selected_square {
                let ss_team = self.board.get_square_team(selected_square);

                if let Some(pl_moves) = &self.board_legal_moves {
                    self.queued_move = if self.player_team == self.board.active_team
                        && ss_team == self.player_team
                    {
                        pl_moves[selected_square]
                            .1
                            .iter()
                            .find(|fmove| fmove.target == target_square_idx)
                            .copied()
                    } else {
                        self.queued_move
                    };
                }
            }
            // Drop the square if there is one
            self.selected_square = None;
            self.drag_x = None;
            self.drag_y = None;
        }

        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Some(graphics::Color::from(BLACK)));

        if let Some(c_move) = self.queued_move {
            if c_move.is_castle {
                println!("Castling!");
            }
            if let Ok(()) = self.board.make_move(c_move) {
                let moving_piece_type = self.board.piece_list[c_move.target];
                let moving_piece_team = self.board.get_square_team(c_move.target);
                self.play_sound(ctx, "piece_move", 0.1)?;
                self.last_move_origin = Some(c_move.start);
                self.last_move_end = Some(c_move.target);
                // Regenerate moves
                self.board_legal_moves = Some(self.board.get_legal_moves());
                let team_legal_moves_active = self.board.prune_moves_for_team(self.board_legal_moves.clone().unwrap(), self.board.active_team);

                self.move_history.push(MoveHistoryEntry {
                    piece_type: moving_piece_type,
                    team: moving_piece_team,
                    checks: self.board.is_team_checked(self.board.active_team),
                    mate: self.board.active_team_checkmate,
                    captures: c_move.captures.is_some(),
                    target: c_move.target,
                    start: c_move.start,
                    castle: c_move.is_castle,
                })
            }

            tracing::debug!(
                "White bitboard after move: {}",
                self.board.get_team_coverage(Team::White)
            );
            tracing::debug!(
                "Black bitboard after move: {}",
                self.board.get_team_coverage(Team::Black)
            );
            // Pull the move from queue
            if self.player_team == self.board.active_team {
                self.opp_thread = None;
            }
            self.queued_move = None;
        }
        self.draw_board(ctx, &mut canvas)?;
        self.draw_pieces(ctx, &mut canvas)?;

        //};
        canvas.finish(ctx)?;
        Ok(())
    }
}
