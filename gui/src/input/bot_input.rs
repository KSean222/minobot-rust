use std::collections::VecDeque;
use std::time::Duration;
use minobot::pathfinder::{ Moves, PathfinderMove };
use minobot::bot::{ Bot, BotSettings };
use minobot::evaluator::Evaluator;
use std::sync::mpsc::{ self, Sender, Receiver, TryRecvError };
use minotetris::*;
use crate::input::*;

const DURATION_ZERO: Duration = Duration::from_millis(0);

pub struct BotController {
    queue: Vec<PathfinderMove>,
    mino_queue_buffer: Vec<PieceType>,
    state: BotControllerState,
    tx: Sender<BotCommand>,
    rx: Receiver<BotResult>,
    inputs: EnumSet<TetrisInput>,
    send_inputs: bool,
    thinking_time: Duration,
    timed_out: bool
}

#[derive(Debug)]
enum BotCommand {
    Update(Vec<PieceType>),
    Reset(Board, Vec<PieceType>),
    Think,
    NextMove,
}

#[derive(Debug)]
enum BotResult {
    Move(Vec<PathfinderMove>, bool, MoveDiagnostics)
}

#[derive(Debug)]
struct MoveDiagnostics {
    thinks: u32,
    mv: Piece,
    moves: u32,
    visits: Vec<u32>
}

#[derive(Debug)]
enum BotControllerState {
    Update,
    Reset,
    Thinking,
    Move(PathfinderMove),
    HardDrop,
    Hold
}

impl BotController {
    pub fn new(evaluator: impl Evaluator + 'static, settings: BotSettings) -> Self {
        let (tx, bot_rx) = mpsc::channel();
        let (bot_tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let board = Board::new();
            let mut bot = Bot::new(board, evaluator, settings);
            let mut moves = 0;
            'handler: loop {
                if let Ok(command) = bot_rx.recv() {
                    match command {
                        BotCommand::Update(new_minos) => {
                            for mino in new_minos {
                                bot.update_queue(mino);
                            }
                        },
                        BotCommand::Reset(board, queue) => {
                            bot.reset(board, queue);
                        },
                        BotCommand::Think => {
                            let mut done = false;
                            let mut thinks = 0;
                            loop {
                                if done {
                                    break;
                                } else {
                                    done = bot.think();
                                    thinks += 1;
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
                            let root = &bot.root;
                            let board = root.board.clone();
                            let visits = root.children
                                .iter()
                                .map(|n| n.visits)
                                .collect();
                            let mut mv = Piece {
                                kind: PieceType::O,
                                x: 0,
                                y: 0,
                                r: 0,
                                tspin: TspinType::None
                            };
                            let mut uses_hold = false;
                            let path = if let Some(node) = bot.next_move() {
                                mv = node.mv;
                                uses_hold = node.uses_hold;
                                Moves::moves(&board, Piece::spawn(&board, mv.kind)).path(mv)
                            } else {
                                Vec::new()
                            };
                            moves += 1;
                            let diagnostics = MoveDiagnostics {
                                thinks,
                                mv,
                                moves,
                                visits
                            };
                            if bot_tx.send(BotResult::Move(path, uses_hold, diagnostics)).is_err() {
                                break 'handler;
                            }
                        },
                        BotCommand::NextMove => unreachable!("Received NextMove command while not thinking")                    }
                } else {
                    break 'handler;
                }
            }
        });
        BotController {
            queue: Vec::new(),
            state: BotControllerState::Reset,
            mino_queue_buffer: Vec::new(),
            tx,
            rx,
            inputs: EnumSet::empty(),
            send_inputs: false,
            thinking_time: DURATION_ZERO,
            timed_out: false
        }
    }
    fn update_state_from_queue(&mut self) {
        self.state = if self.queue.is_empty() {
            BotControllerState::HardDrop
        } else {
            BotControllerState::Move(self.queue.remove(0))
        };
    }
}

impl TetrisController for BotController {
    fn update(&mut self, ctx: &Context, tetris: &mut Tetris, events: &[TetrisEvent]) {
        for event in events {
            match event {
                TetrisEvent::PieceQueued(mino) => {
                    self.mino_queue_buffer.push(*mino);
                },
                _ => {}
            }
        }
        match self.state {
            BotControllerState::Move(mv) => {
                let mut finished = false;
                for event in events {
                    match event {
                        TetrisEvent::PieceMove(prev) => {
                            if tetris.piece.x != prev.x || tetris.piece.r != prev.r  {
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
                let mut pieces = Vec::with_capacity(tetris.queue.max_previews() as usize + 1);
                pieces.push(tetris.piece.kind);
                for i in 0..tetris.queue.max_previews() {
                    pieces.push(tetris.queue.get(i));
                }
                self.tx.send(BotCommand::Reset(tetris.board.compress(), pieces)).unwrap();
                self.tx.send(BotCommand::Think).unwrap();
                self.state = BotControllerState::Thinking;
            },
            BotControllerState::Update => {
                self.tx.send(BotCommand::Update(self.mino_queue_buffer.clone())).unwrap();
                self.mino_queue_buffer.clear();
                self.tx.send(BotCommand::Think).unwrap();
                self.state = BotControllerState::Thinking;
            }
            BotControllerState::Thinking => {
                if !self.timed_out {
                    self.thinking_time += ggez::timer::delta(ctx);
                }
                if let Ok(command) = self.rx.try_recv() {
                    if let BotResult::Move(path, uses_hold, diagnostics) = command {
                        println!("Move {}", diagnostics.moves);
                        println!("ms/think: {}", 100.0 / (diagnostics.thinks as f64));
                        println!("Uses hold: {}", uses_hold);
                        println!("Visits: {:?}", diagnostics.visits);
                        println!();
                        self.thinking_time = DURATION_ZERO;
                        self.timed_out = false;
                        self.queue = path;
                        if uses_hold {
                            self.state = BotControllerState::Hold;
                        } else {
                            self.update_state_from_queue();
                        }
                        // println!("{:?}, {:?}", self.state, self.queue);
                        // tetris.debug_ghost = diagnostics.mv;
                        // tetris.debug_mino = tetris.board.current;
                    }
                } else if self.thinking_time.as_millis() >= 100 {
                    self.tx.send(BotCommand::NextMove).unwrap();
                    self.timed_out = true;
                    self.thinking_time = DURATION_ZERO;
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
                // tetris.debug_mino = tetris.board.current;
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