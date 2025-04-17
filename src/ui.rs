use std::collections::HashMap;

use bitvec::order::Lsb0;
use bitvec::view::BitView;
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
use ggez::GameError;
use ggez::{Context, GameResult};

use crate::bitboard::Team;
use crate::bitboard::PIECE_TYPE_ARRAY;
use crate::board::BoardState;
use crate::bitboard::PieceType;

pub type ColorRGBA = [f32; 4];

const BLACK: ColorRGBA = [0.2, 0.2, 0.2, 1.0];
const LIGHT_SQUARE_COLOR: ColorRGBA = [0.941, 0.467, 0.467, 1.0];
const DARK_SQUARE_COLOR: ColorRGBA = [0.651, 0.141, 0.141, 1.0];
const WIDTH: f32 = 600.0;
const HEIGHT: f32 = 800.0;
const SQUARE_SIZE: f32 = WIDTH / 8.0;

pub struct MainState {
    board: BoardState,
    piece_imgs: HashMap<String, Image>
}

impl MainState {
    pub fn new(board_state: BoardState, ctx: &mut Context) -> GameResult<MainState> {
        let mut s = MainState {
            board: board_state,
            piece_imgs: HashMap::new()
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
                PieceType::None => "-"
            };

            if piece_id != "-" {
                piece_ids.push(String::from("w") + piece_id);
                piece_ids.push(String::from("b") + piece_id);
            }
        });

        piece_ids.iter().for_each(|id| {
            let file_path = format!("/horsey/{}.png", id);
            let image_res = graphics::Image::from_path(ctx, file_path);

            if let Ok(image) = image_res {
                s.piece_imgs.insert(id.to_owned(), image);
            } else {
                println!("{image_res:?}");
            }
        });

        Ok(s)
    }
    fn draw_board(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult<()> {
        for rank in (0..8).rev() {
            for file in (0..8) {
                let color = if (rank + file) % 2 == 0 {
                    Color::from(LIGHT_SQUARE_COLOR)
                } else {
                    Color::from(DARK_SQUARE_COLOR)
                };

                let square_mesh = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    Rect {
                        x: rank as f32 * SQUARE_SIZE,
                        y: file as f32 * SQUARE_SIZE,
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
            for file in (0..8) {
                let black_bits = self.board.get_team_coverage(Team::Black);
                let white_bits = self.board.get_team_coverage(Team::White);
                let square_bit_idx = ((rank * 8) + file) as usize;
                
                let square_team = {
                    let white_bitcheck = white_bits.state.view_bits::<Lsb0>().get(square_bit_idx).expect("Index was not within bitboard").then(|| {Team::White});

                    if white_bitcheck.is_none() {
                        let black_bitcheck = black_bits.state.view_bits::<Lsb0>().get(square_bit_idx).expect("Index was not within bitboard").then(|| {Team::Black});

                        if let Some(bbc) = black_bitcheck {
                            bbc
                        } else {
                            // There is not a piece here.
                            continue
                        }
                    } else {
                        white_bitcheck.unwrap()
                    }
                };
                 
                // We use the team id to compose the team part of the file name
                let file_team = String::from(
                    if square_team == Team::White {
                        "w"
                    } else {
                        "b"
                    }
                );

                // So we know there is a piece, we can just match its type now
                let square_piece = match self.board.piece_list[square_bit_idx] {
                    PieceType::Pawn => "p",
                    PieceType::Knight => "n",
                    PieceType::Rook => "r",
                    PieceType::Queen => "q",
                    PieceType::King => "k",
                    PieceType::Bishop => "b",
                    PieceType::None => {
                        return Err(GameError::RenderError(String::from("FIX - Attempted to draw an empty bit")));
                    }
                };

                let square_piece_id = file_team + square_piece;
                //file_team + square_piece
                let image = self.piece_imgs.get(&   square_piece_id).unwrap();
               
                canvas.draw(image, DrawParam::default().transform(
                    Transform::Values {
                        dest: Point2 {
                            x: file as f32 * SQUARE_SIZE,
                            y: rank as f32 * SQUARE_SIZE,
                        },
                        rotation: 0.0,
                        scale: Vector2 {
                            x: SQUARE_SIZE / image.width() as f32,
                            y:  SQUARE_SIZE / image.height() as f32
                        },
                        offset: Point2 {
                            x: 0.5,
                            y: 0.5,
                        },
                    }.to_bare_matrix()
                ));
            }
        }

        Ok(())
    }
}
impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Some(graphics::Color::from(BLACK)));

        self.draw_board(ctx, &mut canvas)?;
        self.draw_pieces(ctx, &mut canvas)?;
        canvas.finish(ctx)?;
        Ok(())
    }
}
