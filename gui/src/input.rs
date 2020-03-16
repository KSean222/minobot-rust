use ggez::input::keyboard;
use ggez::event::KeyCode;
use ggez::Context;
use enumset::{ EnumSet, EnumSetType };
use crate::tetris::{ Tetris, TetrisEvent };
use std::collections::VecDeque;
use minobot::pathfinder::{ Pathfinder, PathfinderMove };
use minobot::bot::Bot;
use minobot::evaluator::StandardEvaluator;
use std::sync::mpsc::{ self, Sender, Receiver, TryRecvError };
use minotetris::*;
use std::time::Duration;

const DURATION_ZERO: Duration = Duration::from_millis(0);

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
    tx: Sender<BotCommand>,
    rx: Receiver<BotCommand>,
    inputs: EnumSet<TetrisInput>,
    send_inputs: bool
}

#[derive(Debug)]
enum BotCommand {
    //tx
    Update(Tetrimino),
    Reset(Board, Vec<Tetrimino>),
    Think,
    NextMove,
    //rx
    Move(VecDeque<PathfinderMove>, PieceState)
}

#[derive(Debug)]
enum BotControllerState {
    Update,
    Reset,
    Thinking(Duration),
    Move(PathfinderMove),
    HardDrop,
    Hold
}

impl BotController {
    pub fn new() -> Self {
        let (tx, bot_rx) = mpsc::channel();
        let (bot_tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut bot = Bot::new(StandardEvaluator::default());
            let mut pathfinder = Pathfinder::new();
            'handler: loop {
                if let Ok(command) = bot_rx.recv() {
                    match command {
                        BotCommand::Update(mino) => {
                            bot.update(mino);
                        },
                        BotCommand::Reset(board, queue) => {
                            bot.reset(board, queue);
                        },
                        BotCommand::Think => {
                            let mut done = false;
                            loop {
                                if !done {
                                    done = bot.think();
                                }
                                match bot_rx.try_recv() {
                                    Ok(command) => {
                                        match command {
                                            BotCommand::NextMove => {
                                                break;
                                            }
                                            _ => unreachable!("Received command other than NextMove while thinking")
                                        }
                                    },
                                    Err(err) => if err == TryRecvError::Disconnected {
                                        break 'handler;
                                    }
                                }
                            }
                            let mut board = bot.root.as_ref().unwrap().board.clone();
                            let root = bot.root.as_ref().unwrap();
                            println!("score: {}", root.score);
                            println!("sims: {}", root.sims);
                            let mut mv = PieceState {
                                x: 0,
                                y: 0,
                                r: 0
                            };
                            let path = if let Some(node) = bot.next_move() {
                                mv = node.mv;
                                pathfinder.get_moves(&mut board);
                                if let Some(path) = pathfinder.path_to(node.mv.x, node.mv.y, node.mv.r) {
                                    path
                                } else {
                                    VecDeque::with_capacity(0)
                                }
                            } else {
                                VecDeque::with_capacity(0)
                            };
                            if let Err(_) = bot_tx.send(BotCommand::Move(path, mv)) {
                                break 'handler;
                            }
                        },
                        BotCommand::NextMove => unreachable!("Received NextMove command while not thinking"),
                        BotCommand::Move(_, _) => unreachable!("Received Move command (That's my job!)")
                    }
                } else {
                    break 'handler;
                }
            }
        });
        BotController {
            queue: VecDeque::new(),
            state: BotControllerState::Reset,
            tx,
            rx,
            inputs: EnumSet::empty(),
            send_inputs: false
        }
    }
    fn update_state_from_queue(&mut self) {
        self.state = if let Some(mv) = self.queue.pop_front() {
            BotControllerState::Move(mv)
        } else {
            BotControllerState::HardDrop
        };
    }
}

impl TetrisController for BotController {
    fn update(&mut self, ctx: &Context, tetris: &mut Tetris, events: &Vec<TetrisEvent>) {
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
                        self.state = BotControllerState::Update;
                    }
                }
            },
            BotControllerState::Reset => {
                let mut pieces = Vec::with_capacity(tetris.queue.max_previews() as usize);
                for i in 0..tetris.queue.max_previews() {
                    pieces.push(tetris.queue.get(i));
                }
                self.tx.send(BotCommand::Reset(tetris.board.compress(), pieces)).unwrap();
                self.tx.send(BotCommand::Think).unwrap();
                self.state = BotControllerState::Thinking(DURATION_ZERO);
            },
            BotControllerState::Update => {
                self.tx.send(BotCommand::Update(tetris.queue.get(tetris.queue.max_previews() - 1))).unwrap();
                self.tx.send(BotCommand::Think).unwrap();
                self.state = BotControllerState::Thinking(DURATION_ZERO);
            }
            BotControllerState::Thinking(duration) => {
                //TODO de-hackify
                self.state = BotControllerState::Thinking(duration + ggez::timer::delta(ctx));
                if let Ok(command) = self.rx.try_recv() {
                    if let BotCommand::Move(path, mv) = command {
                        self.queue = path;
                        tetris.debug_ghost = mv;
                        tetris.debug_mino = tetris.board.current;
                        self.update_state_from_queue();
                    }
                } else if duration > Duration::from_millis(250) {
                    self.state = BotControllerState::Thinking(DURATION_ZERO);
                    self.tx.send(BotCommand::NextMove).unwrap();
                }
            }
        }
        self.inputs.clear();
        match self.state {
            BotControllerState::Move(mv) => {
                self.inputs.insert(match mv {
                    PathfinderMove::Left => TetrisInput::Left,
                    PathfinderMove::Right => TetrisInput::Right,
                    PathfinderMove::RotLeft => TetrisInput::RotLeft,
                    PathfinderMove::RotRight => TetrisInput::RotRight,
                    PathfinderMove::SonicDrop => TetrisInput::SoftDrop
                });
            },
            BotControllerState::Hold => {
                self.inputs.insert(TetrisInput::Hold); 
            },
            BotControllerState::HardDrop => {
                self.inputs.insert(TetrisInput::HardDrop);
            },
            _ => {}
        }
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