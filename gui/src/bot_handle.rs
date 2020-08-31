use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use std::time::{Instant, Duration};
use std::collections::VecDeque;

use minotetris::*;
use minobot::bot::{Bot, BotSettings};
use minobot::evaluator::Evaluator;
use minobot::pathfinder::{Moves, PathfinderMove};

enum BotCommand {
    NewPiece(PieceType),
    BeginThinking,
    NextMove,
}

pub struct BotMove {
    pub mv: Piece,
    pub uses_hold: bool,
    pub path: VecDeque<PathfinderMove>,
    pub thinks: u32,
    pub think_time: Duration
}

pub struct BotHandle {
    tx: Sender<BotCommand>,
    rx: Receiver<Option<BotMove>>
}

impl BotHandle {
    pub fn new(board: Board, evaluator: impl Evaluator + Send + 'static, settings: BotSettings) -> Self {
        let (tx, bot_rx) = channel::<BotCommand>();
        let (bot_tx, rx) = channel();
        std::thread::spawn(move || {
            let mut thinking = false;
            let mut thinking_start = Instant::now();
            let mut thinks = 0;
            let mut bot = Bot::new(board, evaluator, settings);
            loop {
                let command = if thinking {
                    thinking = !bot.think();
                    thinks += 1;
                    match bot_rx.try_recv() {
                        Ok(command) => command,
                        Err(TryRecvError::Empty) => continue,
                        _ => return
                    }
                } else if let Ok(command) = bot_rx.recv() {
                    command
                } else {
                    return
                };
                match command {
                    BotCommand::BeginThinking => {
                        thinking_start = Instant::now();
                        thinking = true;
                    }
                    BotCommand::NewPiece(piece) => bot.update_queue(piece),
                    BotCommand::NextMove => {
                        thinking = false;
                        let board = bot.root.board.clone();
                        let mv = bot.next_move().map(|node| {
                            let piece = Piece::spawn(&board, node.mv.kind);
                            BotMove {
                                mv: node.mv,
                                uses_hold: node.uses_hold,
                                path: Moves::moves(&board, piece).path(node.mv),
                                think_time: thinking_start.elapsed(),
                                thinks
                            }
                        });
                        thinks = 0;
                        bot_tx.send(mv).unwrap()
                    },
                }
            }
        });
        Self {
            tx,
            rx
        }
    }

    pub fn add_piece(&self, piece: PieceType) {
        self.tx.send(BotCommand::NewPiece(piece)).unwrap();
    }

    pub fn begin_thinking(&self) {
        self.tx.send(BotCommand::BeginThinking).unwrap();
    }

    pub fn next_move(&self) -> Option<BotMove> {
        self.tx.send(BotCommand::NextMove).unwrap();
        self.rx.recv().unwrap()
    }
}
