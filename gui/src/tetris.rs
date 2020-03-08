use std::time::Duration;
use minotetris::*;
use crate::*;
use ggez::graphics::{ Drawable, DrawParam };
use ggez::GameResult;

pub struct Tetris {
    queue: RandomPieceQueue,
    board: Board<ColoredRow>,
    cell_size: f32,
    ARR_timer: Duration,
    DAS_timer: Duration,
    SDD_timer: Duration,
    prev_inputs: EnumSet<TetrisInput>,
    settings: TetrisSettings,
    ghost_y: i32
}

pub struct TetrisSettings {
    pub DAS: Duration,
    pub ARR: Duration,
    pub SDD: Duration
}

const DURATION_ZERO: Duration = Duration::from_millis(0);
impl Tetris {
    pub fn new(settings: TetrisSettings) -> Tetris {
        let mut queue = RandomPieceQueue::new([1u8; 16], 5);
        let mut tetris = Tetris {
            board: Board::<ColoredRow>::new(queue.take()),
            queue,
            cell_size: 32.0,
            DAS_timer: DURATION_ZERO,
            SDD_timer: DURATION_ZERO,
            ARR_timer: DURATION_ZERO,
            prev_inputs: EnumSet::empty(),
            settings,
            ghost_y: 0
        };
        tetris.update_ghost_y();
        tetris
    }
    fn update_ghost_y(&mut self) {
        let orig_y = self.board.state.y;
        while self.board.soft_drop() { }
        self.ghost_y = self.board.state.y;
        self.board.state.y = orig_y;
    }
    pub fn update(&mut self, delta: Duration, inputs: EnumSet<TetrisInput>) {
        let mut orig_x = self.board.state.x;
        let orig_r = self.board.state.r;
        if inputs.contains(TetrisInput::Hold) {
            if self.board.hold.is_none() {
                self.board.hold_piece(self.queue.take());
                orig_x = -1;
            } else if self.board.hold_piece(self.queue.get(0)) {
                orig_x = -1;
            }
        }
        if inputs.contains(TetrisInput::Left) != inputs.contains(TetrisInput::Right) {
            if inputs.contains(TetrisInput::Left) != self.prev_inputs.contains(TetrisInput::Left) ||
            self.prev_inputs.contains(TetrisInput::Right) != inputs.contains(TetrisInput::Right) {
                self.DAS_timer = DURATION_ZERO;
                self.ARR_timer = DURATION_ZERO;
            }
            if self.DAS_timer == DURATION_ZERO {
                if inputs.contains(TetrisInput::Left) {
                    self.board.move_left();
                } else {
                    self.board.move_right();
                }
            }
            if self.DAS_timer < self.settings.DAS {
                self.DAS_timer += delta;
            }
            if self.DAS_timer >= self.settings.DAS {
                if self.ARR_timer > self.settings.ARR {
                    self.ARR_timer = DURATION_ZERO;
                }
                if self.ARR_timer == DURATION_ZERO {
                    if inputs.contains(TetrisInput::Left) {
                        self.board.move_left();
                    } else {
                        self.board.move_right();
                    }
                }
                self.ARR_timer += delta;
            }
        }
        if inputs.contains(TetrisInput::RotLeft) != inputs.contains(TetrisInput::RotRight) {
            if inputs.contains(TetrisInput::RotLeft) {
                if !self.prev_inputs.contains(TetrisInput::RotLeft) {
                    self.board.turn_left();
                }
            } else {
                if !self.prev_inputs.contains(TetrisInput::RotRight) {
                    self.board.turn_right();
                }
            }
        }
        if inputs.contains(TetrisInput::SoftDrop) {
            if !self.prev_inputs.contains(TetrisInput::SoftDrop) || self.SDD_timer >= self.settings.SDD {
                self.SDD_timer = DURATION_ZERO;
            }
            if self.SDD_timer == DURATION_ZERO {
                self.board.soft_drop();
            }
            self.SDD_timer += delta;
        }
        if inputs.contains(TetrisInput::HardDrop) && !self.prev_inputs.contains(TetrisInput::HardDrop) {
            self.board.hard_drop(self.queue.take());
            orig_x = -1;
        }
        if self.board.state.x != orig_x || self.board.state.r != orig_r {
            self.update_ghost_y();
        }
        self.prev_inputs = inputs;
    }
    pub fn draw(&mut self, ctx: &mut ggez::Context, res: &mut Resources, param: DrawParam) -> GameResult {
        res.skin.clear();
        for x in 0..10 {
            for y in 20..40 {
                let cell = self.board.get_cell(x, y);
                self.draw_cell(res, x + 5, y - 20, cell, false);
            }
        }
        self.draw_tetrimino(
            res,
            self.board.state.x + 5,
            self.ghost_y - 20,
            self.board.state.r,
            self.board.current,
            true
        );
        self.draw_tetrimino(
            res,
            self.board.state.x + 5,
            self.board.state.y - 20,
            self.board.state.r,
            self.board.current,
            false
        );
        if let Some(mino) = self.board.hold {
            self.draw_tetrimino(res, 1, 2, 0, mino, false);
        }
        for i in 0..self.queue.max_previews() {
            self.draw_tetrimino(res, 17, i * 3 + 2, 0, self.queue.get(i), false);
        }
        res.skin.draw(ctx, param)
    }
    fn draw_cell(&mut self, res: &mut Resources, x: i32, y: i32, cell: CellType, ghost: bool) {
        let param = res.cell_draw_params(cell, ghost, self.cell_size)
            .dest([
                (x as f32) * self.cell_size,
                (y as f32) * self.cell_size
            ]);
        res.skin.add(param);
    }
    fn draw_tetrimino(&mut self, res: &mut Resources, x: i32, y: i32, r: u8, mino: Tetrimino, ghost: bool) {
        let cell = mino.cell();
        for (cell_x, cell_y) in &mino.cells(r) {
            self.draw_cell(res, cell_x + x, cell_y + y, cell, ghost);
        }
    }
}