use std::collections::VecDeque;

use minotetris::*;
use rand::prelude::*;
use enumset::{EnumSet, EnumSetType};

const ATTACK_TABLE: [u32; 4] = [0, 1, 2, 4];
const TSPIN_MULTIPLIER: u32 = 2;
const B2B_BONUS: u32 = 1;

#[derive(Debug, EnumSetType)]
pub enum TetrisInput {
    HardDrop,
    SoftDrop,
    Left,
    Right,
    RotLeft,
    RotRight,
    Hold
}

pub enum TetrisGameState {
    SpawnDelay(u32),
    PieceFalling(Piece),
    LineClearDelay(LockResult, TspinType, u32),
    GameOver
}

pub struct TetrisGame {
    state: TetrisGameState,
    board: Board,
    held: bool,
    queue: PieceQueue,
    garbage_pending: u32,
    prev_inputs: EnumSet<TetrisInput>,
    das_timer: u32,
    arr_timer: u32,
    config: TetrisGameConfig
}

pub struct TetrisGameConfig {
    queue: u32,
    spawn_delay: u32,
    line_clear_delay: u32,
    das: u32,
    arr: u32
}

pub enum TetrisGameEvent {
    PieceSpawned {
        queued_piece: PieceType
    },
    PieceLocked(LockResult),
    GameOver,
    GarbageSent(u32),
    GarbageAdded(u32)
}

impl TetrisGame {
    pub fn new(config: TetrisGameConfig, rng: &mut (impl Rng + ?Sized)) -> Self {
        Self {
            state: TetrisGameState::SpawnDelay(0),
            board: Board::new(),
            held: false,
            queue: PieceQueue::new(config.queue as usize, rng),
            garbage_pending: 0,
            prev_inputs: EnumSet::new(),
            das_timer: 0,
            arr_timer: 0,
            config
        }
    }
    
    pub fn update(
        &mut self, inputs: EnumSet<TetrisInput>,
        rng: &mut (impl Rng + ?Sized), garbage_rng: &mut (impl Rng + ?Sized)
    ) -> Vec<TetrisGameEvent> {
        let mut events = Vec::new();
        
        match &mut self.state {
            TetrisGameState::PieceFalling(piece) => {
                if inputs.contains(TetrisInput::Hold) && !self.held {
                    if let Some(kind) = self.board.hold.replace(piece.kind) {
                        *piece = Piece::spawn(&self.board, kind);
                        if !self.board.piece_fits(*piece) {
                            self.state = TetrisGameState::GameOver;
                            events.push(TetrisGameEvent::GameOver);
                        }
                    } else {
                        self.state = TetrisGameState::SpawnDelay(0);
                    }
                    self.held = true;
                } else {
                    if inputs.contains(TetrisInput::Left) != inputs.contains(TetrisInput::Right) {
                        let dir = if inputs.contains(TetrisInput::Left) {
                            TetrisInput::Left
                        } else {
                            TetrisInput::Right
                        };
                        let prev_dir = if self.prev_inputs.contains(TetrisInput::Left) != self.prev_inputs.contains(TetrisInput::Right) {
                            Some(if self.prev_inputs.contains(TetrisInput::Left) {
                                TetrisInput::Left
                            } else {
                                TetrisInput::Right
                            })
                        } else {
                            None
                        };
                        if Some(dir) != prev_dir {
                            self.das_timer = 0;
                        }
            
                        if self.das_timer == 0 {
                            if dir == TetrisInput::Left {
                                piece.move_left(&self.board);
                            } else {
                                piece.move_right(&self.board);
                            }
                        }
                        if self.das_timer == self.config.das {
                            loop {
                                if self.arr_timer == 0 {
                                    let success = if dir == TetrisInput::Left {
                                        piece.move_left(&self.board)
                                    } else {
                                        piece.move_right(&self.board)
                                    };
                                    if !success {
                                        break;
                                    }
                                }
                                if self.arr_timer < self.config.arr {
                                    break;
                                }
                                self.arr_timer -= self.config.arr;
                            }
                            self.arr_timer += 1;
                        } else {
                            self.das_timer += 1;
                        }
                    }
            
                    if inputs.contains(TetrisInput::RotLeft) != inputs.contains(TetrisInput::RotRight) {
                        if inputs.contains(TetrisInput::RotLeft) {
                            piece.turn_left(&self.board);
                        } else {
                            piece.turn_right(&self.board);
                        }
                    }
            
                    if inputs.contains(TetrisInput::SoftDrop) {
                        piece.soft_drop(&self.board);
                    }
            
                    if inputs.contains(TetrisInput::HardDrop) {
                        piece.sonic_drop(&self.board);
                        let result = self.board.lock_piece(*piece);
                        events.push(TetrisGameEvent::PieceLocked(result));

                        if result.lines_cleared > 0 {
                            self.state = TetrisGameState::LineClearDelay(result, piece.tspin, 0);
                        } else {
                            if self.apply_garbage(garbage_rng, &mut events) && result.block_out {
                                self.state = TetrisGameState::GameOver;
                                events.push(TetrisGameEvent::GameOver);
                            }
                        }
                    }
                }                
            }
            TetrisGameState::LineClearDelay(result, tspin , elapsed) => {
                *elapsed += 1;
                if *elapsed >= self.config.line_clear_delay {
                    let mut attack = ATTACK_TABLE[result.lines_cleared as usize - 1];
                    if *tspin == TspinType::Full {
                        attack *= TSPIN_MULTIPLIER;
                    }
                    if result.b2b_bonus {
                        attack += B2B_BONUS;
                    }
                    if attack > self.garbage_pending {
                        events.push(TetrisGameEvent::GarbageSent(attack - self.garbage_pending));
                    }
                    self.garbage_pending = self.garbage_pending.saturating_sub(attack);
                    if !self.apply_garbage(garbage_rng, &mut events) {
                        self.state = TetrisGameState::SpawnDelay(0);
                    }
                }
            }
            TetrisGameState::SpawnDelay(elapsed) => {
                *elapsed += 1;
                if *elapsed >= self.config.spawn_delay {
                    let piece = self.queue.next(rng);
                    events.push(TetrisGameEvent::PieceSpawned {
                        queued_piece: piece
                    });
                    let piece = Piece::spawn(&self.board, piece);
                    if !self.board.piece_fits(piece) {
                        self.state = TetrisGameState::GameOver;
                        events.push(TetrisGameEvent::GameOver);
                    } else {
                        self.state = TetrisGameState::PieceFalling(piece);
                    }
                }
            }
            TetrisGameState::GameOver => {}
        }
        
        events
    }

    fn apply_garbage(&mut self, garbage_rng: &mut (impl Rng + ?Sized), events: &mut Vec<TetrisGameEvent>) -> bool {
        if self.garbage_pending > 0 {
            events.push(TetrisGameEvent::GarbageAdded(self.garbage_pending));
            let mut holes = Vec::with_capacity(self.garbage_pending as usize);
            for _ in 0..self.garbage_pending {
                let hole = if let Some(&prev) = holes.last() {
                    if garbage_rng.gen_ratio(3, 10) {
                        let mut hole = prev;
                        while hole != prev {
                            hole = garbage_rng.gen_range(0, 10);
                        }
                        hole
                    } else {
                        prev
                    }
                } else {
                    garbage_rng.gen_range(0, 10)
                };
                holes.push(hole);
            }
            self.garbage_pending = 0;
            if self.board.add_garbage(&holes) {
                self.state = TetrisGameState::GameOver;
                events.push(TetrisGameEvent::GameOver);
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    
    pub fn get_state(&self) -> &TetrisGameState {
        &self.state
    }

    pub fn get_board(&self) -> &Board {
        &self.board
    }

    pub fn get_queue(&self) -> &VecDeque<PieceType> {
        self.queue.get_queue()
    }

    pub fn get_config(&self) -> &TetrisGameConfig {
        &self.config
    }

    pub fn add_garbage(&mut self, garbage: u32) {
        self.garbage_pending += garbage;
    }

    pub fn get_pending_garbage(&self) -> u32 {
        self.garbage_pending
    }
}
