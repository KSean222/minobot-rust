use ggez;
use ggez::event;
use ggez::graphics::{ self, Image, Rect, DrawParam };
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::Context;
use enumset::EnumSet;
use std::boxed::Box;
use minotetris::*;

mod tetris;
use tetris::{ Tetris, TetrisSettings, TetrisEvent };
mod input;
use input::*;

pub struct MainState {
    res: Resources,
    tetris: Tetris,
    controller: Box<dyn TetrisController>,
    event_buffer: Vec<TetrisEvent>
}

impl MainState {
    fn new(ctx: &mut Context) -> ggez::GameResult<MainState> {
        let state = MainState {
            res: Resources::new(ctx),
            tetris: Tetris::new(TetrisSettings::default()),
            controller: Box::new(BotController::new()),
            event_buffer: Vec::new()
        };
        Ok(state)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        self.controller.update(ctx, &mut self.tetris, &self.event_buffer);
        self.event_buffer = self.tetris.update(ggez::timer::delta(ctx), self.controller.inputs());
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(ctx, graphics::BLACK);
        self.tetris.draw(ctx, &mut self.res, DrawParam::default())?;
        graphics::present(ctx)?;
        Ok(())
    }
}

pub struct Resources {
    pub skin: SpriteBatch,
    pub skin_size: i32
}

impl Resources {
    pub fn new(ctx: &mut ggez::Context) -> Resources {
        let skin = Image::new(ctx, "/skin.png").unwrap();
        Resources {
            skin_size: skin.height() as i32,
            skin: SpriteBatch::new(skin)
        }
    }
    fn cell_draw_params(&self, cell: CellType, ghost: bool, size: f32) -> DrawParam {
        let mut index = match cell {
            CellType::Solid => 0,
            CellType::Garbage => 1,
            CellType::Z => 2,
            CellType::L => 3,
            CellType::O => 4,
            CellType::S => 5,
            CellType::I => 6,
            CellType::J => 7,
            CellType::T => 8,
            CellType::Empty => 16
        };
        if ghost {
            index += 7;
        }
        const CELLS: f32 = 17f32;
        let rect = Rect::new((index as f32) / CELLS, 0f32, 1f32 / CELLS, 1f32);
        let scale = size / (self.skin_size as f32);
        DrawParam::new()
            .src(rect)
            .scale([scale, scale])
    }
}

pub fn main() -> ggez::GameResult {
    let (mut ctx, mut event_loop) = ggez::ContextBuilder::new("minobot", "KSean222")
        .window_setup(ggez::conf::WindowSetup {
            title: "MinoBot".to_owned(),
            samples: ggez::conf::NumSamples::Zero,
            vsync: true,
            icon: "".to_owned(),
            srgb: true,
        })
        .window_mode(ggez::conf::WindowMode {
            width: 32.0 * 20.0,
            height: 32.0 * 20.0,
            ..Default::default()
        })
        .add_resource_path("./res")
        .build()?;
    let state = &mut MainState::new(&mut ctx)?;
    event::run(&mut ctx, &mut event_loop, state)
}
