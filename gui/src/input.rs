use ggez::input::keyboard;
use ggez::event::KeyCode;
use ggez::Context;
use enumset::{ EnumSet, EnumSetType };
use crate::tetris::{ Tetris, TetrisEvent };
use std::collections::VecDeque;
use minobot::pathfinder::{ Pathfinder, PathfinderMove };

#[derive(EnumSetType)]
pub enum TetrisInput {
    Hold,
    Left,
    Right,
    RotLeft,
    RotRight,
    HardDrop,
    SoftDrop
}

pub trait TetrisController {
    fn update(&mut self, ctx: &Context, tetris: &mut Tetris, events: &Vec<TetrisEvent>);
    fn inputs(&mut self) -> EnumSet<TetrisInput>;
}

pub struct HumanController {
    pub inputs: EnumSet<TetrisInput>,
    pub hold: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub rot_left: KeyCode,
    pub rot_right: KeyCode,
    pub hard_drop: KeyCode,
    pub soft_drop: KeyCode
}

impl Default for HumanController {
    fn default() -> Self {
        HumanController {
            inputs: EnumSet::empty(),
            hold: KeyCode::C,
            left: KeyCode::Left,
            right: KeyCode::Right,
            rot_left: KeyCode::Z,
            rot_right: KeyCode::Up,
            hard_drop: KeyCode::Space,
            soft_drop: KeyCode::Down
        }
    }
}

impl TetrisController for HumanController {
    fn update(&mut self, ctx: &Context, _tetris: &mut Tetris, _events: &Vec<TetrisEvent>) {
        self.inputs.clear();
        if keyboard::is_key_pressed(ctx, self.hold) {
            self.inputs.insert(TetrisInput::Hold);
        }
        if keyboard::is_key_pressed(ctx, self.left) {
            self.inputs.insert(TetrisInput::Left);
        }
        if keyboard::is_key_pressed(ctx, self.right) {
            self.inputs.insert(TetrisInput::Right);
        }
        if keyboard::is_key_pressed(ctx, self.rot_left) {
            self.inputs.insert(TetrisInput::RotLeft);
        }
        if keyboard::is_key_pressed(ctx, self.rot_right) {
            self.inputs.insert(TetrisInput::RotRight);
        }
        if keyboard::is_key_pressed(ctx, self.hard_drop) {
            self.inputs.insert(TetrisInput::HardDrop);
        }
        if keyboard::is_key_pressed(ctx, self.soft_drop) {
            self.inputs.insert(TetrisInput::SoftDrop);
        }
    }
    fn inputs(&mut self) -> EnumSet<TetrisInput> {
        self.inputs
    }
}

pub struct BotController {
    queue: VecDeque<PathfinderMove>,
    state: BotControllerState,
    bot: Pathfinder,
    inputs: EnumSet<TetrisInput>,
    send_inputs: bool
}

#[derive(Debug)]
enum BotControllerState {
    Think,
    Move(PathfinderMove),
    HardDrop,
    Hold
}

impl BotController {
    pub fn new() -> Self {
        BotController {
            queue: VecDeque::new(),
            state: BotControllerState::Think,
            bot: Pathfinder::new(),
            inputs: EnumSet::empty(),
            send_inputs: false
        }
    }
}

impl BotController {
    fn update_state_from_queue(&mut self) {
        self.state = if let Some(mv) = self.queue.pop_front() {
            BotControllerState::Move(mv)
        } else {
            BotControllerState::HardDrop
        };
    }
}

impl TetrisController for BotController {
    fn update(&mut self, _ctx: &Context, tetris: &mut Tetris, events: &Vec<TetrisEvent>) {
        match self.state {
            BotControllerState::Move(mv) => {
                let mut finished = false;
                for event in events {
                    match event {
                        TetrisEvent::PieceMove(prev) => {
                            if tetris.board.state.x != prev.x || tetris.board.state.r != prev.r  {
                                finished = true;
                            }
                        },
                        TetrisEvent::StackTouched => {
                            if mv == PathfinderMove::SonicDrop {
                                finished = true;
                            }
                        },
                        _ => {}
                    }
                }
                if finished {
                    self.update_state_from_queue();
                }
            },
            BotControllerState::Hold => {
                for event in events {
                    if let TetrisEvent::PieceHeld = event {
                        self.update_state_from_queue();
                    }
                }
            },
            BotControllerState::HardDrop => {
                for event in events {
                    if let TetrisEvent::PieceLocked(_) = event {
                        self.state = BotControllerState::Think;
                    }
                }
            },
            BotControllerState::Think => {
                let moves = self.bot.get_moves(&mut tetris.board.compress());
                let index = (rand::random::<f64>() * (moves.len() as f64)) as usize;
                for (i, pos) in moves.iter().enumerate() {
                    if i == index {
                        tetris.debug_ghost = *pos;
                        self.queue = self.bot.path_to(pos.x, pos.y, pos.r);
                    }
                }
                self.update_state_from_queue();
            }
        }
        self.inputs.clear();
        match self.state {
            BotControllerState::Move(mv) => self.inputs.insert(match mv {
                PathfinderMove::Left => TetrisInput::Left,
                PathfinderMove::Right => TetrisInput::Right,
                PathfinderMove::RotLeft => TetrisInput::RotLeft,
                PathfinderMove::RotRight => TetrisInput::RotRight,
                PathfinderMove::SonicDrop => TetrisInput::SoftDrop
            }),
            BotControllerState::Hold => self.inputs.insert(TetrisInput::Hold),
            BotControllerState::HardDrop => self.inputs.insert(TetrisInput::HardDrop),
            BotControllerState::Think => false
        };
    }
    fn inputs(&mut self) -> EnumSet<TetrisInput> {
        if self.send_inputs {
            self.send_inputs = false;
            self.inputs
        } else {
            self.send_inputs = true;
            EnumSet::empty()
        }
    }
}