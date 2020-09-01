use serde::{ Serialize, Deserialize };

use crate::pathfinder::Moves;
use crate::evaluator::{ Evaluator, StandardEvaluator };
use minotetris::*;

pub struct Bot<E=StandardEvaluator> {
    pub data: BotData<E>,
    pub root: Node,
}

pub struct BotData<E> {
    pub queue: Vec<PieceType>,
    pub settings: BotSettings,
    evaluator: E,
}

#[derive(Serialize, Deserialize)]
pub struct BotSettings {
    pub use_hold: bool,
    pub exploration_exploitation_constant: f64
}

impl Default for BotSettings {
    fn default() -> Self {
        BotSettings {
            use_hold: true,
            exploration_exploitation_constant: std::f64::consts::SQRT_2
        }
    }
}

impl<E: Evaluator> Bot<E> {
    pub fn new(board: Board, evaluator: E, settings: BotSettings) -> Self {
        Bot {
            data: BotData {
                queue: Vec::new(),
                evaluator,
                settings
            },
            root: Node::root(board),
        }
    }
    pub fn update_queue(&mut self, mino: PieceType) {
        self.data.queue.push(mino);
    }
    pub fn reset(&mut self, board: Board, queue: Vec<PieceType>) {
        self.root = Node::root(board);
        self.data.queue = queue;
    }
    pub fn think(&mut self) -> bool {
        self.root.update(&mut self.data);
        self.root.finished
    }
    pub fn next_move(&mut self) -> Option<&Node> {
        self.data.queue.remove(0);
        let root = self.root.children
            .drain(..)
            .max_by(|a, b| {
                (a.value + a.reward + a.max_child_reward)
                    .partial_cmp(&(b.value + b.reward + b.max_child_reward))
                    .unwrap()
            });
        if let Some(root) = root {
            self.root = root;
            self.root.advance();
            // for &row in self.root.board.rows.iter().skip(20) {
            //     for x in 0..10 {
            //         print!("{}", if row.get(x) {
            //             "[]"
            //         } else {
            //             ".."
            //         });
            //     }
            //     println!()
            // }
            // println!("Hold: {:?}, Queue: {:?}", self.root.board.hold, self.data.queue);
            Some(&self.root)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Node {
    pub board: Board,
    pub mv: Piece,
    pub lock: LockResult,
    pub uses_hold: bool,

    pub children: Vec<Node>,
    pub value: f64,
    pub reward: f64,
    pub max_child_reward: f64,
    pub visits: u32,
    pub finished: bool,
    pub depth: u32
}

impl Node {
    fn root(board: Board) -> Self {
        Self {
            board,
            children: Vec::new(),
            mv: Piece {
                kind: PieceType::O,
                x: 0,
                y: 0,
                r: 0,
                tspin: TspinType::None
            },
            lock: LockResult {
                lines_cleared: 0
            },
            value: std::f64::NEG_INFINITY,
            reward: 0.0,
            max_child_reward: 0.0,
            visits: 1,
            uses_hold: false,
            finished: false,
            depth: 0
        }
    }
    fn update<E: Evaluator>(&mut self, data: &mut BotData<E>) -> ((f64, f64), u32) {
        let mut child_index = None;
        let mut score = std::f64::NEG_INFINITY;
        for (i, c) in self.children.iter().enumerate() {
            if !c.finished {
                // let child_score = c.score / (c.sims as f64) + 1.0 * SQRT_2
                //     * ((node.sims as f64).ln() / (c.sims as f64)).sqrt();
                let child_score =
                    (i as f64) / (self.children.len() as f64) +
                    data.settings.exploration_exploitation_constant * 
                    ((self.visits as f64).ln() / (c.visits as f64)).sqrt();
                if child_score > score {
                    child_index = Some(i);
                    score = child_score;
                }
            }
        }
        if let Some(child_index) = child_index {
            let ((value, reward), visits) = self.children[child_index].update(data);
            let child = self.children.remove(child_index);
            let child_index = self.children
                .iter()
                .position(|c| {
                    c.value + c.reward + c.max_child_reward >
                    child.value + child.reward + child.max_child_reward
                })
                .unwrap_or(self.children.len());
            self.children.insert(child_index, child);
            if value + reward > self.value + self.max_child_reward {
                self.value = value;
                self.max_child_reward = reward;
            }
            self.visits += visits;
            ((value, self.reward + reward), visits)
        } else if self.children.is_empty() {
            self.expand(data)
        } else {
            self.finished = true;
            ((std::f64::NEG_INFINITY, 0.0), 0)
        }
    }
    fn expand<E: Evaluator>(&mut self, data: &mut BotData<E>) -> ((f64, f64), u32) {
        let piece = Piece::spawn(&self.board, data.queue[self.depth as usize]);
        for mv in Moves::moves(&self.board, piece).moves {
            self.create_child(data, mv, false);
        }
        if data.settings.use_hold {
            let mut hold_board = self.board.clone();
            let piece_type = hold_board.hold
                .replace(data.queue[self.depth as usize])
                .or(data.queue.get((self.depth + 1) as usize).copied());
            if let Some(piece_type) = piece_type {
                let piece = Piece::spawn(&self.board, piece_type);
                for mv in Moves::moves(&hold_board, piece).moves {
                    self.create_child(data, mv, true);
                }
            }
        }
        if self.children.is_empty() {
            self.finished = true;
            ((std::f64::NEG_INFINITY, 0.0), 0)
        } else {
            self.children.sort_unstable_by(|a, b| {
                (a.value + a.reward).partial_cmp(&(b.value + b.reward)).unwrap()
            });
            let best = self.children.last().unwrap();
            let visits = self.children.len() as u32;
            self.value = best.value;
            self.max_child_reward = best.reward;
            self.visits += visits;
            ((best.value, self.reward + best.reward), visits)
        }
    }
    fn create_child<E: Evaluator>(&mut self, data: &mut BotData<E>, mv: Piece, uses_hold: bool) {
        let mut board = self.board.clone();
        let mut child_depth = self.depth;
        if uses_hold {
            if board.hold.replace(data.queue[child_depth as usize]).is_none() {
                child_depth += 1;
            }
        }
        let lock = board.lock_piece(mv);
        child_depth += 1;
        let mut child = Node {
            board,
            mv,
            lock,
            children: Vec::new(),
            depth: child_depth,
            value: 0.0,
            reward: 0.0,
            max_child_reward: 0.0,
            visits: 1,
            uses_hold,
            finished: child_depth as usize >= data.queue.len()
        };
        let (value, reward) = data.evaluator.evaluate(&child, self, &data.queue);
        child.value = value;
        child.reward = reward;
        self.children.push(child);
    }
    fn advance(&mut self){
        self.finished = false;
        self.depth -= 1;
        for c in self.children.iter_mut() {
            c.advance();
        }
    }
}
