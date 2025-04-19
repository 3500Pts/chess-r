use std::collections::HashMap;

use bitvec::order::Lsb0;
use bitvec::order::Msb0;
use bitvec::view::BitView;
use ggez::audio::SoundSource;
use ggez::audio::Source;
use ggez::GameError;
use ggez::conf::WindowSetup;
use ggez::event;
use ggez::glam::*;
use ggez::graphics::Canvas;
use ggez::graphics::DrawParam;
use ggez::graphics::Image;
use ggez::graphics::Rect;
use ggez::graphics::Transform;
use ggez::graphics::{self, Color};
use ggez::mint::Point2;
use ggez::mint::Vector2;
use ggez::{Context, GameResult};

use crate::bitboard::PIECE_TYPE_ARRAY;
use crate::bitboard::PieceType;
use crate::bitboard::Team;
use crate::board::BoardState;
use crate::r#move::Move;

pub type ColorRGBA = [f32; 4];

const BLACK: ColorRGBA = [0.2, 0.2, 0.2, 1.0];
const SELECTED_SQUARE_COLOR: ColorRGBA = [1.0, 1.0, 1.0, 1.0];
const LEGAL_MOVE_COLOR: ColorRGBA = [0.5, 0.5, 0.5, 1.0];
const LEGAL_CAP_COLOR: ColorRGBA = [0.25, 0.25, 0.25, 1.0];
const ORIGIN_COLOR: ColorRGBA = [0.0, 0.0, 1.0, 1.0];
const ANTI_ORIGIN_COLOR: ColorRGBA = [0.0, 1.0, 1.0, 1.0];
const LIGHT_SQUARE_COLOR: ColorRGBA = [0.941, 0.467, 0.467, 1.0];
const DARK_SQUARE_COLOR: ColorRGBA = [0.651, 0.141, 0.141, 1.0];
const WIDTH: f32 = 600.0;
const HEIGHT: f32 = 800.0;
const SQUARE_SIZE: f32 = WIDTH / 8.0;
const FLAG_DEBUG_UI_COORDS: bool = false;

pub struct MainState {
    board: BoardState,
    piece_imgs: HashMap<String, Image>,
    sound_sources: HashMap<String, Source>,
    selected_square: Option<usize>,
    queued_move: Option<Move>, // Moves are queued to the draw queue so nothing changes during drawing
    drag_x: Option<f32>,
    drag_y: Option<f32>,
    
}

impl MainState {
    pub fn new(board_state: BoardState, ctx: &mut Context) -> GameResult<MainState> {
        let mut s = MainState {
            board: board_state,
            piece_imgs: HashMap::new(),
            sound_sources: HashMap::new(),
            selected_square: None,
            queued_move: None,
            drag_x: None,
            drag_y: None
        };

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
        let sound_paths = vec!["bass_intro".to_string(), "piece_move".to_string(), "capture".to_string()];

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
        for rank in (0..8) {
        for file in (0..8) {
                let square_number = 63 - (((7 - rank) * 8) + file) as usize;
                let color = 
                if square_number == 0 && FLAG_DEBUG_UI_COORDS {
                    Color::from(ORIGIN_COLOR)
                } else if square_number == 6 && FLAG_DEBUG_UI_COORDS {
                    Color::from(ANTI_ORIGIN_COLOR)
                } else if Some(square_number) == self.selected_square {
                    Color::from(SELECTED_SQUARE_COLOR)
                } else if (rank + file) % 2 == 0 {
                    Color::from(LIGHT_SQUARE_COLOR)
                } else {
                    Color::from(DARK_SQUARE_COLOR)
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

                canvas.draw(&square_mesh, DrawParam::default());
            }
        }
        Ok(())
    }
    fn draw_pieces(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult<()> {
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
                    let image = self.piece_imgs.get(&square_piece_id).expect(&format!("Couldn't find piece png for {square_piece_id}"));
                    
                    let piece_x = file as f32 * SQUARE_SIZE;
                    let piece_y = rank as f32 * SQUARE_SIZE;
                    let piece_x = if Some(square_bit_idx) == self.selected_square { self.drag_x.unwrap_or(piece_x) } else { piece_x };
                    let piece_y = if Some(square_bit_idx) == self.selected_square { self.drag_y.unwrap_or(piece_y) } else { piece_y };
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
        if let Some(start_square) = self.selected_square {
            let start_square = start_square as f32;
            let start_x = ((start_square % 8.0) * SQUARE_SIZE);
            let start_y = ((start_square / 8.0) * SQUARE_SIZE);

            self.drag_x = Some(x - (0.5 * SQUARE_SIZE));
            self.drag_y = Some(y - (0.5 * SQUARE_SIZE));
        }
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
            // Attempt a move here
            if let Some(start_square) = self.selected_square {
                self.queued_move = Some(Move {
                    start: start_square,
                    target: target_square_idx,
                    captures: None,
                });
            }
            // Drop the square
            self.selected_square = None;
            self.drag_x = None;
            self.drag_y = None;
        }

        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Some(graphics::Color::from(BLACK)));

        if let Some(c_move) = self.queued_move {
            let bit_pre = self.board.piece_list.clone();

            if let Ok(()) = self.board.make_move(c_move) {
                self.play_sound(ctx, "piece_move", 0.1)?;
            }

            tracing::debug!("White bitboard after move: {}", self.board.get_team_coverage(Team::White));
            tracing::debug!("Black bitboard after move: {}", self.board.get_team_coverage(Team::Black));
            // Pull the move from queue
            self.queued_move = None;
        }
        self.draw_board(ctx, &mut canvas)?;
        self.draw_pieces(ctx, &mut canvas)?;
        canvas.finish(ctx)?;
        Ok(())
    }
}
