use ggez::conf::WindowSetup;
use ggez::event;
use ggez::glam::*;
use ggez::graphics::Canvas;
use ggez::graphics::DrawParam;
use ggez::graphics::Rect;
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};

pub type ColorRGBA = [f32; 4];

const BLACK: ColorRGBA = [0.2, 0.2, 0.2, 1.0];
const LIGHT_SQUARE_COLOR: ColorRGBA = [0.941, 0.467, 0.467, 1.0];
const DARK_SQUARE_COLOR: ColorRGBA = [0.651, 0.141, 0.141, 1.0];
const WIDTH: f32 = 600.0;
const HEIGHT: f32 = 800.0;
const SQUARE_SIZE: f32 = WIDTH / 8.0;

pub struct MainState {}

impl MainState {
    pub fn new() -> GameResult<MainState> {
        let s = MainState {};
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
                println!(
                    "{:?}",
                    Rect {
                        x: rank as f32 * SQUARE_SIZE,
                        y: file as f32 * SQUARE_SIZE,
                        h: SQUARE_SIZE,
                        w: SQUARE_SIZE
                    }
                );

                canvas.draw(&square_mesh, DrawParam::default());
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

        println!("drawboard incoming");
        self.draw_board(ctx, &mut canvas).unwrap();

        canvas.finish(ctx)?;
        Ok(())
    }
}
