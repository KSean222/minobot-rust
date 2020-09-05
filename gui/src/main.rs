use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::time::{Instant, Duration};
use std::collections::VecDeque;

use ggez;
use ggez::event;
use ggez::graphics;
use ggez::nalgebra as na;
use minotetris::*;
use minobot::pathfinder::PathfinderMove;
use minobot::evaluator::StandardEvaluator;
use minobot::bot::BotSettings;
use serde::{Serialize, Deserialize};

mod bot_handle;
use bot_handle::BotHandle;

struct MainState {
    board: Board<ColoredRow>,
    queue: PieceQueue,
    piece: Piece,
    bot: BotHandle,
    state: State,
    think_time: Duration,
    move_time: Duration
}

enum State {
    Thinking(Instant),
    Moving(VecDeque<PathfinderMove>, Instant),
}

const OPTIONS_PATH: &'static str = "minobot_options.yaml";

#[derive(Serialize, Deserialize)]
struct Options {
    evaluator: StandardEvaluator,
    settings: BotSettings,
    think_time: u64,
    move_time: u64,
    queue: u32
}

impl Default for Options {
    fn default() -> Self {
        Self {
            evaluator: StandardEvaluator::default(),
            settings: BotSettings::default(),
            think_time: 100,
            move_time: 50,
            queue: 5
        }
    }
}

impl Options {
    pub fn read() -> Result<Self, OptionsReadingError> {
        match File::open(&OPTIONS_PATH) {
            Ok(file) => Ok(serde_yaml::from_reader(BufReader::new(file))?),
            Err(e) => if e.kind() == std::io::ErrorKind::NotFound {
                let options = Options::default();
                let file = BufWriter::new(File::create(OPTIONS_PATH)?);
                serde_yaml::to_writer(file, &options)?;
                Ok(options)
            } else {
                Err(e.into())
            }
        }
    }
}

#[derive(Debug)]
enum OptionsReadingError {
    FileError(std::io::Error),
    YamlParsingError(serde_yaml::Error)
}

impl From<std::io::Error> for OptionsReadingError {
    fn from(err: std::io::Error) -> Self {
        Self::FileError(err)
    }
}

impl From<serde_yaml::Error> for OptionsReadingError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::YamlParsingError(err)
    }
}

const HOLD_WIDTH: i32 = 4;
const HOLD_PADDING: i32 = 1;
const BOARD_WIDTH: i32 = 10;
const QUEUE_PADDING: i32 = 1;
const QUEUE_WIDTH: i32 = 4;
const WIDTH: i32 = HOLD_WIDTH + HOLD_PADDING + BOARD_WIDTH + QUEUE_PADDING + QUEUE_WIDTH;
const HEIGHT: i32 = 20;

impl MainState {
    fn new() -> ggez::GameResult<MainState> {
        let options = match Options::read() {
            Ok(options) => options,
            Err(err) => {
                println!("Error reading options file: {:?}", err);
                Options::default()
            }
        };
        let mut rng = rand::thread_rng();

        let board = Board::<ColoredRow>::new();
        let mut queue = PieceQueue::new(options.queue as usize, &mut rng);
        let piece = Piece::spawn(&board, queue.next(&mut rng));

        let bot = BotHandle::new(board.compress(), options.evaluator, options.settings);
        bot.add_piece(piece.kind);
        for &piece in queue.get_queue() {
            bot.add_piece(piece);
        }
        bot.begin_thinking();
        
        Ok(MainState {
            board,
            queue,
            piece,
            bot,
            state: State::Thinking(Instant::now()),
            think_time: Duration::from_millis(options.think_time),
            move_time: Duration::from_millis(options.move_time),
        })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult {
        match self.state {
            State::Thinking(start) => {
                if start.elapsed() > self.think_time {
                    let mv = self.bot.next_move().unwrap();
                    println!("Thinks: {}", mv.thinks);
                    println!("ms/think: {}", mv.think_time.as_millis() as f32 / mv.thinks as f32);
                    println!();
                    if mv.uses_hold {
                        let piece = self.board.hold
                            .replace(self.piece.kind)
                            .unwrap_or_else(|| {
                                let piece = self.queue.next(&mut rand::thread_rng());
                                self.bot.add_piece(*self.queue.get_queue().back().unwrap());
                                piece
                            });
                        self.piece = Piece::spawn(&self.board, piece);
                    }
                    self.state = State::Moving(mv.path, Instant::now());
                }
            }
            State::Moving(ref mut path, ref mut instant) => {
                if instant.elapsed() > self.move_time {
                    if let Some(mv) = path.pop_front() {
                        match mv {
                            PathfinderMove::Left => { self.piece.move_left(&self.board); },
                            PathfinderMove::Right => { self.piece.move_right(&self.board); },
                            PathfinderMove::RotLeft => { self.piece.turn_left(&self.board); },
                            PathfinderMove::RotRight => { self.piece.turn_right(&self.board); },
                            PathfinderMove::SonicDrop => while self.piece.soft_drop(&self.board) {},
                        }
                        *instant = Instant::now();
                    } else {
                        while self.piece.soft_drop(&self.board) {}
                        self.board.lock_piece(self.piece);
                        self.piece = Piece::spawn(&self.board, self.queue.next(&mut rand::thread_rng()));
                        self.bot.add_piece(*self.queue.get_queue().back().unwrap());
                        self.bot.begin_thinking();
                        self.state = State::Thinking(Instant::now());
                    }
                }
            }
        }
        Ok(())
    }

    fn resize_event(&mut self, ctx: &mut ggez::Context, width: f32, height: f32) {
        let rect = graphics::Rect::new(0.0, 0.0, width, height);
        graphics::set_screen_coordinates(ctx, rect).unwrap();
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(ctx, graphics::BLACK);
        let mut mesh = graphics::MeshBuilder::new();

        for y in 20..40 {
            for x in 0..10 {
                let cell = self.board.rows[y as usize].cell_type(x as usize);
                self.draw_cell(ctx, &mut mesh, cell, false, HOLD_WIDTH + HOLD_PADDING + x, y - 20)?;
            }
        }
        let mut piece = self.piece;
        piece.x += HOLD_WIDTH + HOLD_PADDING;
        piece.y -= 20;
        self.draw_piece(ctx, &mut mesh, piece, false)?;
        
        let mut ghost = self.piece;
        while ghost.soft_drop(&self.board) {}
        ghost.x += HOLD_WIDTH + HOLD_PADDING;
        ghost.y -= 20;
        self.draw_piece(ctx, &mut mesh, ghost, true)?;

        if let Some(piece) = self.board.hold {
            let piece = Piece {
                kind: piece,
                x: 1,
                y: 2,
                r: 0,
                tspin: TspinType::None
            };
            self.draw_piece(ctx, &mut mesh, piece, false)?;
        }

        for (i, piece) in self.queue.get_queue().clone().into_iter().enumerate() {
            let piece = Piece {
                kind: piece,
                x: HOLD_WIDTH + HOLD_PADDING + BOARD_WIDTH + QUEUE_PADDING + 1,
                y: 2 + i as i32 * 4,
                r: 0,
                tspin: TspinType::None
            };
            self.draw_piece(ctx, &mut mesh, piece, false)?;
        }

        let mesh = mesh.build(ctx)?;
        graphics::draw(ctx, &mesh, (na::Point2::new(0.0, 0.0),))?;
        graphics::present(ctx)?;
        Ok(())
    }
}

impl MainState {
    fn draw_piece(&mut self, ctx: &mut ggez::Context, mesh: &mut graphics::MeshBuilder, piece: Piece, ghost: bool) -> ggez::GameResult {
        for &(x, y) in &piece.cells() {
            self.draw_cell(ctx, mesh, piece.kind.cell(), ghost, x, y)?;
        }
        Ok(())
    }

    fn draw_cell(&mut self, ctx: &mut ggez::Context, mesh: &mut graphics::MeshBuilder, cell: CellType, ghost: bool, x: i32, y: i32) -> ggez::GameResult {
        let (width, height) = graphics::drawable_size(ctx);
        let cell_size = (width / WIDTH as f32).min(height / HEIGHT as f32);
        let start_x = (width - cell_size * WIDTH as f32) / 2.0;
        let start_y = (height - cell_size * HEIGHT as f32) / 2.0;
        let bounds = graphics::Rect::new(
            start_x + x as f32 * cell_size,
            start_y + y as f32 * cell_size,
            cell_size,
            cell_size
        );
        let color = match cell {
            CellType::Empty => (112, 128, 144),//(112, 128, 144),
            CellType::Garbage => (112, 128, 144),
            CellType::Solid => (105, 105, 105),
            CellType::J => (0, 0, 255),
            CellType::L => (255, 69, 0),
            CellType::S => (0, 255, 0),
            CellType::T => (138, 43, 226),
            CellType::Z => (255, 0, 0),
            CellType::I => (0, 255, 255),
            CellType::O => (255, 215, 0)
        };
        let mut color: graphics::Color = color.into();
        if ghost {
            color.a = 0.5;
        }
        let mode = graphics::DrawMode::fill();
        mesh.rectangle(mode, bounds, color);
        if cell == CellType::Empty {
            const BORDER: f32 = 0.1;
            let mut bounds = bounds.clone();
            let border = cell_size * BORDER;
            bounds.translate([border / 2.0, border / 2.0]);
            bounds.scale(1.0 - BORDER, 1.0 - BORDER);
            mesh.rectangle(mode, bounds, graphics::BLACK);
        }
        Ok(())
    }
}

pub fn main() -> ggez::GameResult { 
    let context_builder = ggez::ContextBuilder::new("Minobot GUI", "KSean222")
        .window_setup(ggez::conf::WindowSetup {
            title: "MinoBot GUI".to_owned(),
            ..Default::default()
        })    
        .window_mode(ggez::conf::WindowMode {
            resizable: true,
            ..Default::default()
        });
    let (mut ctx, mut events_loop) = context_builder.build()?;
    let mut state = MainState::new()?;
    event::run(&mut ctx, &mut events_loop, &mut state)
}
