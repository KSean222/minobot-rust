use std::time::Duration;
use minotetris::*;
use crate::*;
use ggez::graphics::{ Drawable, DrawParam };
use ggez::GameResult;

pub struct Tetris {
    pub queue: RandomPieceQueue,
    pub board: Board<ColoredRow>,
    pub piece: Piece,
    held: bool,
    cell_size: f32,
    arr_timer: Duration,
    das_timer: Duration,
    sdd_timer: Duration,
    prev_inputs: EnumSet<TetrisInput>,
    settings: TetrisSettings,
    ghost_y: i32,
    pub debug_ghost: Piece
}

pub struct TetrisSettings {
    effective_frame: Duration,
    pub das: Duration,
    pub arr: Duration,
    pub sdd: Duration
}

impl Default for TetrisSettings {
    fn default() -> Self {
        TetrisSettings {
            effective_frame: Duration::from_micros(16667),
            arr: Duration::from_micros(33333),
            das: Duration::from_millis(150),
            sdd: Duration::from_micros(33333)
        }
    }
}

const DURATION_ZERO: Duration = Duration::from_millis(0);
impl Tetris {
    pub fn new(settings: TetrisSettings) -> Tetris {
        let mut queue = RandomPieceQueue::new([1u8; 16], 5);
        let board = Board::new();
        let mut tetris = Tetris {
            piece: Piece::spawn(&board, queue.take()),
            board,
            held: false,
            queue,
            cell_size: 32.0,
            das_timer: DURATION_ZERO,
            sdd_timer: DURATION_ZERO,
            arr_timer: DURATION_ZERO,
            prev_inputs: EnumSet::empty(),
            settings,
            ghost_y: 0,
            debug_ghost: Piece {
                kind: PieceType::O,
                x: 0,
                y: 0,
                r: 0,
                tspin: TspinType::None
            },
        };
        tetris.update_ghost_y();
        tetris
    }
    fn update_ghost_y(&mut self) {
        let mut piece = self.piece;
        while piece.soft_drop(&self.board) { }
        self.ghost_y = piece.y;
    }
    fn take_mino(&mut self, events: &mut Vec<TetrisEvent>) -> PieceType {
        let mino = self.queue.take();
        events.push(TetrisEvent::PieceQueued(self.queue.get(self.queue.max_previews() - 1)));
        mino
    }
    pub fn update(&mut self, delta: Duration, inputs: EnumSet<TetrisInput>) -> Vec<TetrisEvent> {
        let mut events = Vec::new();
        let orig_state = self.piece;
        if inputs.contains(TetrisInput::Hold) && !self.held {
            let piece_type = self.board.hold
                .replace(self.piece.kind)
                .unwrap_or_else(|| self.take_mino(&mut events));
            self.piece = Piece::spawn(&self.board, piece_type);
            events.push(TetrisEvent::PieceHeld);
        }
        if inputs.contains(TetrisInput::Left) != inputs.contains(TetrisInput::Right) {
            if inputs.contains(TetrisInput::Left) != self.prev_inputs.contains(TetrisInput::Left) ||
            self.prev_inputs.contains(TetrisInput::Right) != inputs.contains(TetrisInput::Right) {
                self.das_timer = DURATION_ZERO;
                self.arr_timer = DURATION_ZERO;
            }
            if self.das_timer == DURATION_ZERO {
                if inputs.contains(TetrisInput::Left) {
                    self.piece.move_left(&self.board);
                } else {
                    self.piece.move_right(&self.board);
                }
            }
            if self.das_timer < self.settings.das {
                self.das_timer += delta;
            }
            if self.das_timer >= self.settings.das {
                if self.arr_timer > self.settings.arr {
                    self.arr_timer = DURATION_ZERO;
                }
                if self.arr_timer == DURATION_ZERO {
                    if inputs.contains(TetrisInput::Left) {
                        self.piece.move_left(&self.board);
                    } else {
                        self.piece.move_right(&self.board);
                    }
                }
                self.arr_timer += delta;
            }
        }
        if inputs.contains(TetrisInput::RotLeft) != inputs.contains(TetrisInput::RotRight) {
            if inputs.contains(TetrisInput::RotLeft) {
                if !self.prev_inputs.contains(TetrisInput::RotLeft) {
                    self.piece.turn_left(&self.board);
                }
            } else {
                if !self.prev_inputs.contains(TetrisInput::RotRight) {
                    self.piece.turn_right(&self.board);
                }
            }
        }
        if inputs.contains(TetrisInput::SoftDrop) {
            if !self.prev_inputs.contains(TetrisInput::SoftDrop) || self.sdd_timer >= self.settings.sdd {
                self.sdd_timer = DURATION_ZERO;
            }
            if self.sdd_timer == DURATION_ZERO {
                self.piece.soft_drop(&self.board);
            }
            self.sdd_timer += delta;
        }
        if inputs.contains(TetrisInput::HardDrop) && !self.prev_inputs.contains(TetrisInput::HardDrop) {
            while self.piece.soft_drop(&self.board) {}
            let result = self.board.lock_piece(self.piece);
            let piece = self.take_mino(&mut events);
            self.piece = Piece::spawn(&self.board, piece);
            events.push(TetrisEvent::PieceLocked(result));
        }
        if self.piece.x != orig_state.x || self.piece.r != orig_state.r || self.piece.kind != orig_state.kind {
            self.update_ghost_y();
        }
        if self.piece != orig_state {
            events.push(TetrisEvent::PieceMove(orig_state));
            if self.piece.y == self.ghost_y {
                events.push(TetrisEvent::StackTouched);
            }
        }
        self.prev_inputs = inputs;
        events
    }
    pub fn draw(&mut self, ctx: &mut ggez::Context, res: &mut Resources, param: DrawParam) -> GameResult {
        res.skin.clear();
        for x in 0..10 {
            for y in 20..40 {
                let cell = self.board.rows[y as usize].cell_type(x as usize);
                self.draw_cell(res, x + 5, y - 20, cell, false);
            }
        }
        self.draw_tetrimino(
            res,
            self.piece.x + 5,
            self.ghost_y - 20,
            self.piece.r,
            self.piece.kind,
            true
        );
        self.draw_tetrimino(
            res,
            self.piece.x + 5,
            self.piece.y - 20,
            self.piece.r,
            self.piece.kind,
            false
        );
        self.draw_tetrimino(
            res,
            self.debug_ghost.x + 5,
            self.debug_ghost.y - 20,
            self.debug_ghost.r,
            self.debug_ghost.kind,
            true
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
    fn draw_tetrimino(&mut self, res: &mut Resources, x: i32, y: i32, r: u8, mino: PieceType, ghost: bool) {
        let cell = mino.cell();
        for (cell_x, cell_y) in &mino.cells(r) {
            self.draw_cell(res, cell_x + x, cell_y + y, cell, ghost);
        }
    }
}

#[derive(Debug)]
pub enum TetrisEvent {
    PieceMove(Piece),
    StackTouched,
    PieceLocked(LockResult),
    PieceHeld,
    PieceQueued(PieceType)
}
