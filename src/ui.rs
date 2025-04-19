use std::collections::HashMap;

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
use crate::opponents;

pub type ColorRGBA = [f32; 4];

const BLACK: ColorRGBA = [0.2, 0.2, 0.2, 1.0];
const SELECTED_SQUARE_COLOR: ColorRGBA = [1.0, 1.0, 1.0, 1.0];
const OLD_MOVE_COLOR: ColorRGBA = [1.0, 0.8, 0.25, 1.0];
const LEGAL_MOVE_COLOR_LERP: f32 = 0.3;
const LIGHT_SQUARE_COLOR: ColorRGBA = [0.941, 0.467, 0.467, 1.0];
const DARK_SQUARE_COLOR: ColorRGBA = [0.651, 0.141, 0.141, 1.0];
const WIDTH: f32 = 600.0;
const SQUARE_SIZE: f32 = WIDTH / 8.0;
const FLAG_DEBUG_UI_COORDS: bool = true;

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
pub fn color_lerp(left: Color, right: Color, t: f32) -> Color {
    return Color::from([
        lerp(left.r, right.r, t),
        lerp(left.g, right.g, t),
        lerp(left.b, right.b, t),
        lerp(left.a, right.a, t),
    ]);
}
pub struct MainState {
    board: BoardState,
    piece_imgs: HashMap<String, Image>,
    sound_sources: HashMap<String, Source>,
    selected_square: Option<usize>,
    queued_move: Option<Move>, // Moves are queued to the draw queue so nothing changes during drawing
    drag_x: Option<f32>,
    drag_y: Option<f32>,
    board_legal_moves: Option<Vec<(Bitboard, Vec<Move>)>>,
    last_move_origin: Option<usize>,
    last_move_end: Option<usize>,
    player_team: Team,
}

impl MainState {
    pub fn new(
        board_state: BoardState,
        ctx: &mut Context,
        plr_team: Team,
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
        };
        s.board_legal_moves = Some(s.board.get_psuedolegal_moves());
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
            } else {
                println!("{image_res:?}");
            }
        });

        // Preload sounds
        let sound_paths = vec![
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
    fn draw_board(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult<()> {
        for rank in 0..8 {
            for file in 0..8 {
                let square_number = 63 - (((7 - rank) * 8) + file) as usize;
                // What an unholy if statement. TODO: Make it neater maybe
                let default_color = if (rank + file) % 2 == 0 {
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

                        if status_on_bitboard.unwrap().then_some(true).is_some() {
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
                let square_bit_idx = 63 - ((rank * 8) + file) as usize;

                let square_team_opt = self.board.get_square_team(square_bit_idx);

                if let Some(square_team) = square_team_opt {
                    // We use the team id to compose the team part of the file name
                    let file_team =
                        String::from(if square_team == Team::White { "w" } else { "b" });

                    if square_bit_idx == 0 {
                        //println!("{square_team:?}")
                    }
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
                        .expect(&format!("Couldn't find piece png for {square_piece_id}"));

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

        return 63.0 - ((rank * 8.0) + file);
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
        if button == event::MouseButton::Left && self.queued_move == None {
            let target_square_idx = MainState::get_square_idx_from_pixel(x, y) as usize;
            tracing::debug!("Mouse up at square {}", target_square_idx);
            // Attempt a move here if it's on the bitboard

            if let Some(selected_square) = self.selected_square {
                if let Some(pl_moves) = &self.board_legal_moves {
                    self.queued_move = if self.player_team == self.board.active_team {
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
            if let Ok(()) = self.board.make_move(c_move) {
                self.play_sound(ctx, "piece_move", 0.1)?;
                self.last_move_origin = Some(c_move.start);
                self.last_move_end = Some(c_move.target);
                // Regenerate moves
                self.board_legal_moves = Some(self.board.get_psuedolegal_moves())
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
            self.queued_move = None;
        }
        self.draw_board(ctx, &mut canvas)?;
        self.draw_pieces(ctx, &mut canvas)?;
        canvas.finish(ctx)?;
        Ok(())
    }
}
